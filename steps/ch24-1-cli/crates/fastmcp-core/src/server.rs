//! A small MCP server on top of `rmcp`, speaking stdio.
//!
//! Chapter 2 chose a protocol version. This server also keeps a `HashMap` of
//! tools and calls the one the client names.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, CompleteRequestParams, CompleteResult,
    CompletionInfo, ContentBlock, ErrorData, GetPromptRequestParams, GetPromptResponse,
    GetPromptResult, Implementation, InitializeRequestParams, InitializeResult, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    Prompt, PromptArgument, PromptMessage, ProtocolVersion, ReadResourceRequestParams,
    ReadResourceResponse, ReadResourceResult, Reference, Resource, ResourceContents,
    ResourceTemplate, Role, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::app_error::AppError;
use crate::context::{Context, LogNote, ProgressNote, SharedNotes};
use crate::elicit::{Desk, ElicitError};
use crate::inject::{Extract, InjectCtx};
use crate::lifespan::AppState;
use crate::negotiate::{choose_version, era_note, era_of, SUPPORTED_VERSIONS};
use crate::pagination::{paginate_sequence, Page};
use crate::prompts::{complete_values, RegisteredPrompt};
use crate::resources::{StaticResource, UriTemplate};
use crate::tools::{
    empty_object_schema, register, register_app, register_ctx, register_inject, RegisteredTool,
    ToolDecl, ToolError,
};

const DEFAULT_PAGE_SIZE: usize = 50;

/// The server this chapter runs.
///
/// `tool` takes `self` and returns `Self`, so registrations chain.
#[derive(Clone)]
pub struct FastMcp {
    name: String,
    version: String,
    tools: HashMap<String, RegisteredTool>,
    resources: HashMap<String, StaticResource>,
    templates: Vec<UriTemplate>,
    prompts: HashMap<String, RegisteredPrompt>,
    completions: HashMap<(String, String), Vec<String>>,
    page_size: usize,
    notes: Arc<SharedNotes>,
    deps: InjectCtx,
    state: AppState,
    elicits: HashMap<String, String>,
    desk: Desk,
    pub(crate) hooks: Vec<std::sync::Arc<dyn crate::middleware::ToolHook>>,
}

impl FastMcp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".into(),
            tools: HashMap::new(),
            resources: HashMap::new(),
            templates: Vec::new(),
            prompts: HashMap::new(),
            completions: HashMap::new(),
            page_size: DEFAULT_PAGE_SIZE,
            notes: Arc::new(SharedNotes::default()),
            deps: InjectCtx::default(),
            state: AppState::default(),
            elicits: HashMap::new(),
            desk: Desk::default(),
            hooks: Vec::new(),
        }
    }

    /// Register a tool that waits for [`Desk::submit`] before it returns.
    pub fn elicit(mut self, name: impl Into<String>, message: impl Into<String>) -> Self {
        self.elicits.insert(name.into(), message.into());
        self
    }

    /// The desk a second task uses to answer an in-flight tool.
    pub fn desk(&self) -> Desk {
        self.desk.clone()
    }

    pub fn app_state(&self) -> AppState {
        self.state.clone()
    }

    /// Store `value` where `Dep<T>` can find it. The client never sends `T`.
    pub fn provide<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        self.deps.insert(value);
        self
    }

    pub fn logs(&self) -> Vec<LogNote> {
        self.notes.logs()
    }

    pub fn progress_notes(&self) -> Vec<ProgressNote> {
        self.notes.progress()
    }

    /// How many items one `list_*` response holds. The default is 50.
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.max(1);
        self
    }

    /// Store a resource whose URI is exact. A second call with the same URI replaces the first.
    pub fn resource(
        mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let uri = uri.into();
        self.resources.insert(
            uri.clone(),
            StaticResource {
                uri,
                name: name.into(),
                mime: mime.into(),
                body: body.into(),
            },
        );
        self
    }

    /// Store a URI template. `{param}` captures one path segment.
    pub fn template<F>(
        mut self,
        pattern: impl Into<String>,
        name: impl Into<String>,
        mime: impl Into<String>,
        render: F,
    ) -> Self
    where
        F: Fn(&[(String, String)]) -> Result<String, String> + Send + Sync + 'static,
    {
        self.templates
            .push(UriTemplate::new(pattern, name, mime, render));
        self
    }

    /// Register a prompt. A second call with the same name replaces the first.
    ///
    /// `arguments` is `(name, required)`. The closure receives the argument
    /// map the client sent and returns the message text.
    pub fn prompt<F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        arguments: &[(&str, bool)],
        render: F,
    ) -> Self
    where
        F: Fn(&HashMap<String, String>) -> Result<String, String> + Send + Sync + 'static,
    {
        let name = name.into();
        let arguments = arguments
            .iter()
            .map(|(argument, required)| PromptArgument::new(*argument).with_required(*required))
            .collect();
        self.prompts.insert(
            name.clone(),
            RegisteredPrompt::new(name, description, arguments, render),
        );
        self
    }

    /// Candidates for one prompt argument. Completion filters these by prefix.
    pub fn completion(
        mut self,
        prompt: impl Into<String>,
        argument: impl Into<String>,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.completions.insert(
            (prompt.into(), argument.into()),
            values.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Render `name` with `arguments`.
    pub fn render_prompt(
        &self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<String, ToolError> {
        let prompt = self.prompts.get(name).ok_or(ToolError::Unknown)?;
        prompt.render(arguments).map_err(ToolError::InvalidParams)
    }

    /// Completion values for a prompt argument, capped at 100.
    pub fn complete_argument(
        &self,
        prompt: &str,
        argument: &str,
        prefix: &str,
    ) -> (Vec<String>, u32, bool) {
        let empty = Vec::new();
        let candidates = self
            .completions
            .get(&(prompt.to_string(), argument.to_string()))
            .unwrap_or(&empty);
        complete_values(candidates, prefix)
    }

    /// Read a static resource, or the first template that matches.
    ///
    /// The `Ok` pair is `(body, mime)`. An exact URI wins over every template.
    pub fn read(&self, uri: &str) -> Result<(String, String), ToolError> {
        if let Some(resource) = self.resources.get(uri) {
            return Ok((resource.body.clone(), resource.mime.clone()));
        }
        for template in &self.templates {
            if let Some(rendered) = template.render(uri) {
                return rendered
                    .map(|body| (body, template.mime.clone()))
                    .map_err(ToolError::Failed);
            }
        }
        Err(ToolError::Unknown)
    }

    /// Register `call` under `name`.
    ///
    /// `P` is the argument struct. `R` is the value the closure returns.
    /// A second call with the same name replaces the first. The closure runs
    /// only after `P` deserializes. Both schemas are built here, once.
    pub fn tool<P, R, F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        call: F,
    ) -> Self
    where
        P: DeserializeOwned + JsonSchema + Send + 'static,
        R: Serialize + JsonSchema,
        F: Fn(P) -> Result<R, String> + Send + Sync + 'static,
    {
        self.tools
            .insert(name.into(), register::<P, R, F>(description, call));
        self
    }

    /// Register a tool whose `Err` is an [`AppError`].
    ///
    /// `ToolFailed` is still a tool result with `is_error`. `NotFound`,
    /// `Disabled`, and `Validation` become `invalid_params`. Anything else
    /// becomes `internal_error`, unless the error already chose a code.
    pub fn tool_app<P, R, F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        call: F,
    ) -> Self
    where
        P: DeserializeOwned + JsonSchema + Send + 'static,
        R: Serialize + JsonSchema,
        F: Fn(P) -> Result<R, AppError> + Send + Sync + 'static,
    {
        self.tools
            .insert(name.into(), register_app::<P, R, F>(description, call));
        self
    }

    /// Register a tool that receives a [`Context`].
    ///
    /// The schema is still `P`. `Context` is not a parameter the client sends.
    /// The context sees the static resources registered before this call.
    pub fn tool_ctx<P, R, F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        call: F,
    ) -> Self
    where
        P: DeserializeOwned + JsonSchema + Send + 'static,
        R: Serialize + JsonSchema,
        F: Fn(Context, P) -> Result<R, String> + Send + Sync + 'static,
    {
        let notes = Arc::clone(&self.notes);
        let resources = self.resources.clone();
        self.tools.insert(
            name.into(),
            register_ctx(
                description,
                move || Context::new(Arc::clone(&notes), resources.clone()),
                call,
            ),
        );
        self
    }

    /// Register a tool whose first argument is extracted, not deserialized.
    ///
    /// The schema is `P`. `Db` and `Token` stay out of `required`.
    pub fn tool_inject<P, R, E, F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        call: F,
    ) -> Self
    where
        P: DeserializeOwned + JsonSchema + Send + 'static,
        R: Serialize + JsonSchema,
        E: Extract + 'static,
        F: Fn(E, P) -> Result<R, String> + Send + Sync + 'static,
    {
        self.tools.insert(
            name.into(),
            register_inject(description, self.deps.clone(), call),
        );
        self
    }

    /// Register the struct a `#[tool]` macro emitted.
    ///
    /// This calls [`Self::tool`]. The macro's work was naming the struct and
    /// the argument fields. The schema is still built here, once, at registration.
    pub fn tool_decl<T: ToolDecl + 'static>(self) -> Self {
        self.tool(T::name(), T::description(), T::invoke)
    }

    /// `true` when `name` was registered with [`Self::tool`].
    ///
    /// `status` is not in the map. [`Self::call_tool`] answers it on its own.
    /// Names a client would see, `status` first, then registered tools in order.
    pub fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .tools
            .keys()
            .cloned()
            .chain(self.elicits.keys().cloned())
            .collect();
        names.sort();
        names.dedup();
        let mut listed = Vec::with_capacity(names.len() + 1);
        listed.push("status".to_string());
        listed.extend(names);
        listed
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// The description stored for `name`, when the tool was registered.
    pub fn tool_description(&self, name: &str) -> Option<&str> {
        self.tools.get(name).map(RegisteredTool::description)
    }

    /// The input schema stored for `name`.
    pub fn tool_schema(
        &self,
        name: &str,
    ) -> Option<std::sync::Arc<serde_json::Map<String, Value>>> {
        self.tools.get(name).map(RegisteredTool::schema)
    }

    /// Look up `name` and run it.
    ///
    /// # Errors
    ///
    /// [`ToolError::Unknown`] if the name is missing, [`ToolError::InvalidParams`]
    /// if `arguments` is not a `P`, [`ToolError::Failed`] if the closure returns `Err`.
    pub fn dispatch_tool(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        let tool = self.tools.get(name).ok_or(ToolError::Unknown)?;
        tool.call(arguments)
    }

    /// Serve one connection on the process's stdin and stdout.
    ///
    /// # Errors
    ///
    /// Returns the transport or handshake error `rmcp` reports. A client that
    /// closes stdin ends the function.
    pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let running = self.serve(rmcp::transport::stdio()).await?;
        running.waiting().await?;
        Ok(())
    }

    fn info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .enable_completions()
                .build(),
        )
        .with_server_info(Implementation::new(self.name.clone(), self.version.clone()))
        .with_protocol_version(ProtocolVersion::V_2025_11_25)
    }

    fn listed_tools(&self, version: &ProtocolVersion) -> Vec<Tool> {
        let mut names: Vec<String> = self
            .tools
            .keys()
            .cloned()
            .chain(self.elicits.keys().cloned())
            .collect();
        names.sort();
        names.dedup();
        let mut listed = Vec::with_capacity(names.len() + 1);
        listed.push(status_tool(version));
        for name in names {
            if let Some(registered) = self.tools.get(&name) {
                listed.push(
                    Tool::new(
                        name,
                        registered.description().to_string(),
                        registered.schema(),
                    )
                    .with_raw_output_schema(registered.output_schema()),
                );
            } else {
                let message = self.elicits[&name].clone();
                listed.push(Tool::new(name, message, empty_object_schema()));
            }
        }
        listed
    }
}

fn status_tool(version: &ProtocolVersion) -> Tool {
    Tool::new("status", era_note(era_of(version)), empty_object_schema())
}

impl ServerHandler for FastMcp {
    fn get_info(&self) -> ServerConfig {
        self.info()
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_VERSIONS)
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        context.peer.set_peer_info(request.clone());
        let version = choose_version(&request.protocol_version, SUPPORTED_VERSIONS)?;
        let mut info = self.info();
        info.protocol_version = version.clone();
        info.instructions = Some(era_note(era_of(&version)).to_string());
        Ok(info)
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let version = context
            .protocol_version()
            .unwrap_or(ProtocolVersion::V_2025_06_18);
        let (tools, next_cursor) =
            turn_page(self.listed_tools(&version), &request, self.page_size)?;
        let mut listed = ListToolsResult::with_all_items(tools);
        listed.next_cursor = next_cursor;
        Ok(listed)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if request.name == "status" {
            let version = context
                .protocol_version()
                .unwrap_or(ProtocolVersion::V_2025_06_18);
            let note = era_note(era_of(&version));
            return Ok(CallToolResult::success(vec![ContentBlock::text(note)]).into());
        }
        if let Some(message) = self.elicits.get(request.name.as_ref()).cloned() {
            return match self.desk.ask(message).await {
                Ok(value) => {
                    Ok(CallToolResult::structured(serde_json::json!({"answer": value})).into())
                }
                Err(ElicitError::Declined) => {
                    Ok(CallToolResult::error(vec![ContentBlock::text("declined")]).into())
                }
                Err(ElicitError::Cancelled) => {
                    Ok(CallToolResult::error(vec![ContentBlock::text("cancelled")]).into())
                }
            };
        }
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        match self.invoke(request.name.as_ref(), arguments) {
            Ok(value) => Ok(CallToolResult::structured(value).into()),
            Err(ToolError::Unknown) => Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            )),
            Err(ToolError::InvalidParams(message)) => Err(ErrorData::invalid_params(message, None)),
            Err(ToolError::Failed(message)) => {
                Ok(CallToolResult::error(vec![ContentBlock::text(message)]).into())
            }
            Err(ToolError::Wire(data)) => Err(data),
        }
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let mut items: Vec<Resource> = self
            .resources
            .values()
            .map(|resource| {
                Resource::new(resource.uri.clone(), resource.name.clone())
                    .with_mime_type(resource.mime.clone())
            })
            .collect();
        items.sort_by(|left, right| left.uri.cmp(&right.uri));
        let (resources, next_cursor) = turn_page(items, &request, self.page_size)?;
        let mut listed = ListResourcesResult::with_all_items(resources);
        listed.next_cursor = next_cursor;
        Ok(listed)
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        let mut items: Vec<ResourceTemplate> = self
            .templates
            .iter()
            .map(|template| {
                ResourceTemplate::new(template.pattern.clone(), template.name.clone())
                    .with_mime_type(template.mime.clone())
            })
            .collect();
        items.sort_by(|left, right| left.uri_template.cmp(&right.uri_template));
        let (templates, next_cursor) = turn_page(items, &request, self.page_size)?;
        let mut listed = ListResourceTemplatesResult::with_all_items(templates);
        listed.next_cursor = next_cursor;
        Ok(listed)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        match self.read(&request.uri) {
            Ok((body, mime)) => {
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(body, request.uri).with_mime_type(mime)
                ])
                .into())
            }
            Err(ToolError::Unknown) => Err(ErrorData::invalid_params(
                format!("unknown resource: {}", request.uri),
                None,
            )),
            Err(ToolError::InvalidParams(message)) => Err(ErrorData::invalid_params(message, None)),
            Err(ToolError::Failed(message)) => Err(ErrorData::internal_error(message, None)),
            Err(ToolError::Wire(data)) => Err(data),
        }
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let mut names: Vec<&String> = self.prompts.keys().collect();
        names.sort();
        let items = names
            .into_iter()
            .map(|name| {
                let prompt = &self.prompts[name];
                Prompt::new(
                    prompt.name.clone(),
                    Some(prompt.description.clone()),
                    Some(prompt.arguments.clone()),
                )
            })
            .collect();
        let (prompts, next_cursor) = turn_page(items, &request, self.page_size)?;
        let mut listed = ListPromptsResult::with_all_items(prompts);
        listed.next_cursor = next_cursor;
        Ok(listed)
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        let arguments = request
            .arguments
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(key, value)| value.as_str().map(|text| (key, text.to_string())))
            .collect();
        match self.render_prompt(&request.name, &arguments) {
            Ok(text) => Ok(
                GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)])
                    .with_description(
                        self.prompts
                            .get(request.name.as_str())
                            .map(|prompt| prompt.description.clone())
                            .unwrap_or_default(),
                    )
                    .into(),
            ),
            Err(ToolError::Unknown) => Err(ErrorData::invalid_params(
                format!("unknown prompt: {}", request.name),
                None,
            )),
            Err(ToolError::InvalidParams(message)) => Err(ErrorData::invalid_params(message, None)),
            Err(ToolError::Failed(message)) => Err(ErrorData::internal_error(message, None)),
            Err(ToolError::Wire(data)) => Err(data),
        }
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, ErrorData> {
        let reference = match request.r#ref {
            Reference::Prompt(prompt) => prompt.name,
            Reference::Resource(template) => template.uri,
            _ => String::new(),
        };
        let (values, total, has_more) =
            self.complete_argument(&reference, &request.argument.name, &request.argument.value);
        let info = CompletionInfo::with_pagination(values, Some(total), has_more)
            .map_err(|message| ErrorData::internal_error(message, None))?;
        Ok(CompleteResult::new(info))
    }
}

fn turn_page<T: Clone>(
    items: Vec<T>,
    request: &Option<PaginatedRequestParams>,
    page_size: usize,
) -> Result<(Vec<T>, Option<String>), ErrorData> {
    let cursor = request.as_ref().and_then(|params| params.cursor.as_deref());
    let Page { items, next_cursor } = paginate_sequence(&items, cursor, page_size)
        .map_err(|message| ErrorData::invalid_params(message, None))?;
    Ok((items, next_cursor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, JsonSchema)]
    struct Add {
        left: i64,
        right: i64,
        label: Option<String>,
    }

    #[derive(Serialize, JsonSchema)]
    struct Sum {
        total: i64,
    }

    fn add_server() -> FastMcp {
        FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
            let _ = args.label;
            Ok(Sum {
                total: args.left + args.right,
            })
        })
    }

    #[test]
    fn calls_a_registered_tool() {
        let server = add_server();
        let value = server
            .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
            .expect("add");
        assert_eq!(value, serde_json::json!({"total": 5}));
        assert!(server.has_tool("add"));
        assert!(!server.has_tool("missing"));
    }

    #[test]
    fn unknown_tool_is_not_a_panic() {
        let error = add_server()
            .dispatch_tool("nope", Value::Object(serde_json::Map::new()))
            .expect_err("missing");
        assert_eq!(error, ToolError::Unknown);
    }

    #[test]
    fn missing_field_does_not_run_the_tool() {
        let error = add_server()
            .dispatch_tool("add", serde_json::json!({"left": 1}))
            .expect_err("right is required");
        assert!(matches!(error, ToolError::InvalidParams(_)));
    }

    #[test]
    fn tool_err_is_a_failure_not_a_panic() {
        let server = FastMcp::new("fastmcp-rs").tool(
            "nope",
            "always fails",
            |args: Add| -> Result<Sum, String> {
                let _ = args.label;
                Err("nope".to_string())
            },
        );
        let error = server
            .dispatch_tool("nope", serde_json::json!({"left": 1, "right": 2}))
            .expect_err("tool err");
        assert_eq!(error, ToolError::Failed("nope".to_string()));
    }

    #[test]
    fn a_second_registration_replaces_the_name() {
        let server = FastMcp::new("fastmcp-rs")
            .tool("add", "first", |args: Add| {
                let _ = (args.label, args.right);
                Ok(Sum { total: args.left })
            })
            .tool("add", "second", |args: Add| {
                let _ = (args.label, args.left);
                Ok(Sum { total: args.right })
            });
        let value = server
            .dispatch_tool("add", serde_json::json!({"left": 2, "right": 9}))
            .expect("replaced");
        assert_eq!(value, serde_json::json!({"total": 9}));
        assert_eq!(
            server.tools.get("add").expect("add").description(),
            "second"
        );
    }

    #[test]
    fn omitted_label_runs_and_stays_out_of_required() {
        let server = add_server();
        let value = server
            .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
            .expect("label is optional");
        assert_eq!(value, serde_json::json!({"total": 5}));
        let schema = server.tools.get("add").expect("add").schema();
        let required = schema
            .get("required")
            .and_then(|value| value.as_array())
            .expect("required");
        assert!(required.iter().any(|name| name == "left"));
        assert!(required.iter().all(|name| name != "label"));
        let output = server.tools.get("add").expect("add").output_schema();
        assert_eq!(
            output
                .get("properties")
                .and_then(|value| value.get("total"))
                .and_then(|value| value.get("type"))
                .and_then(|value| value.as_str()),
            Some("integer")
        );
    }
}
