//! A client picks a transport from a string, then talks to an in-memory server.
//!
//! `infer_transport` only classifies. The connection this chapter opens is a
//! `tokio` duplex, the same pipe stdio uses, with no child process. A group
//! lists every member at once and prefixes each tool with the server name.

use std::collections::HashSet;
use std::fmt;

use rmcp::model::{ClientConfig, ClientRequest, PingRequest, ProtocolVersion, ServerResult};
use rmcp::service::{ClientInitializeError, RoleClient, RunningService, ServiceError};
use rmcp::{ClientHandler, ClientLifecycleMode, ClientServiceExt, ServiceExt};
use tokio::task::JoinSet;

use crate::server::FastMcp;

/// What a connection string points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Stdio,
    Http,
    Sse,
    Memory,
    Config,
}

impl fmt::Display for TransportKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TransportKind::Stdio => "stdio",
            TransportKind::Http => "http",
            TransportKind::Sse => "sse",
            TransportKind::Memory => "memory",
            TransportKind::Config => "config",
        };
        formatter.write_str(name)
    }
}

/// Classify `input`. Nothing is opened.
///
/// # Errors
///
/// Returns a message when the string matches none of the kinds.
pub fn infer_transport(input: &str) -> Result<TransportKind, String> {
    let input = input.trim();
    if input == "memory" {
        return Ok(TransportKind::Memory);
    }
    if input.ends_with(".py") || input.ends_with(".js") {
        return Ok(TransportKind::Stdio);
    }
    if input.contains("mcpServers") {
        return Ok(TransportKind::Config);
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        if input.ends_with("/sse") {
            return Ok(TransportKind::Sse);
        }
        return Ok(TransportKind::Http);
    }
    Err(format!("could not infer a transport from {input}"))
}

/// A connected client. Dropping it drops the connection.
pub struct McpClient {
    running: RunningService<RoleClient, QuietClient>,
    /// `None` when the session came from `server/discover`.
    handshake: Option<ProtocolVersion>,
}

struct QuietClient {
    version: ProtocolVersion,
}

impl ClientHandler for QuietClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = self.version.clone();
        info
    }
}

impl McpClient {
    /// Handshake with `server` over an in-memory duplex.
    ///
    /// # Errors
    ///
    /// Returns the client error `rmcp` reports when the handshake fails.
    pub async fn connect_memory(server: FastMcp) -> Result<Self, ClientError> {
        Self::connect(server, false).await
    }

    /// `server/discover` for `2026-07-28`. There is no `InitializeResult`.
    ///
    /// # Errors
    ///
    /// Returns the client error `rmcp` reports when discover fails.
    pub async fn connect_discover(server: FastMcp) -> Result<Self, ClientError> {
        Self::connect(server, true).await
    }

    async fn connect(server: FastMcp, discover: bool) -> Result<Self, ClientError> {
        let (server_io, client_io) = tokio::io::duplex(64 * 1024);
        tokio::spawn(async move {
            let running = server.serve(server_io).await.expect("server");
            let _ = running.waiting().await;
        });
        let handler = QuietClient {
            version: ProtocolVersion::V_2026_07_28,
        };
        let running = if discover {
            handler
                .serve_with_lifecycle(
                    client_io,
                    ClientLifecycleMode::Discover {
                        preferred_versions: vec![ProtocolVersion::V_2026_07_28],
                    },
                )
                .await
                .map_err(ClientError::from_initialize)?
        } else {
            QuietClient {
                version: ProtocolVersion::V_2025_06_18,
            }
            .serve(client_io)
            .await
            .map_err(ClientError::from_initialize)?
        };
        let handshake = if discover {
            None
        } else {
            Some(
                running
                    .peer_info()
                    .expect("handshake stores peer info")
                    .protocol_version
                    .clone(),
            )
        };
        Ok(Self { running, handshake })
    }

    /// The version stored at connect. A second call returns the same value.
    ///
    /// # Errors
    ///
    /// [`ClientError::NoInitializeResult`] when the session was discovered.
    pub fn initialize(&self) -> Result<ProtocolVersion, ClientError> {
        self.handshake
            .clone()
            .ok_or(ClientError::NoInitializeResult)
    }

    /// Send `ping`. An empty result is success.
    ///
    /// # Errors
    ///
    /// Returns the transport or protocol error from the peer.
    pub async fn ping(&self) -> Result<bool, ClientError> {
        let result = self
            .running
            .send_request(ClientRequest::PingRequest(PingRequest {
                method: Default::default(),
                extensions: Default::default(),
            }))
            .await?;
        Ok(matches!(result, ServerResult::EmptyResult(_)))
    }

    /// Tool names on this one server, in list order.
    ///
    /// # Errors
    ///
    /// Returns the peer error from `tools/list`.
    pub async fn tool_names(&self) -> Result<Vec<String>, ClientError> {
        let listed = self.running.list_tools(Default::default()).await?;
        Ok(listed
            .tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect())
    }
}

/// One client per server. Tool names are `{server}_{tool}`.
pub struct ClientGroup {
    members: Vec<(String, McpClient)>,
}

impl ClientGroup {
    /// # Errors
    ///
    /// Returns [`ClientError::Empty`] when `members` is empty.
    pub fn new(members: Vec<(String, McpClient)>) -> Result<Self, ClientError> {
        if members.is_empty() {
            return Err(ClientError::Empty);
        }
        Ok(Self { members })
    }

    /// List every member at once. Names are sorted.
    ///
    /// # Errors
    ///
    /// [`ClientError::Collision`] when two servers would publish the same public name.
    /// A peer error from any member fails the whole list.
    pub async fn list_tools(&self) -> Result<Vec<String>, ClientError> {
        let mut set = JoinSet::new();
        for (name, client) in &self.members {
            let name = name.clone();
            let peer = client.running.peer().clone();
            set.spawn(async move {
                let listed = peer.list_tools(Default::default()).await?;
                let names = listed
                    .tools
                    .into_iter()
                    .map(|tool| format!("{name}_{}", tool.name))
                    .collect::<Vec<_>>();
                Ok::<_, ServiceError>(names)
            });
        }
        let mut public = Vec::new();
        while let Some(joined) = set.join_next().await {
            let names = joined.expect("list task")?;
            public.extend(names);
        }
        let mut seen = HashSet::new();
        for name in &public {
            if !seen.insert(name.clone()) {
                return Err(ClientError::Collision(name.clone()));
            }
        }
        public.sort();
        Ok(public)
    }
}

/// Failures a client or a group can report.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("client group requires at least one client")]
    Empty,
    #[error("tool name collision: {0}")]
    Collision(String),
    #[error("discover session has no InitializeResult")]
    NoInitializeResult,
    #[error("{0}")]
    Peer(String),
}

impl From<ServiceError> for ClientError {
    fn from(error: ServiceError) -> Self {
        ClientError::Peer(error.to_string())
    }
}

impl ClientError {
    fn from_initialize(error: ClientInitializeError) -> Self {
        ClientError::Peer(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_url_ending_in_sse_is_the_older_transport() {
        assert_eq!(
            infer_transport("http://127.0.0.1:9/sse").expect("sse"),
            TransportKind::Sse
        );
        assert_eq!(
            infer_transport("https://example.com/mcp").expect("http"),
            TransportKind::Http
        );
        assert_eq!(
            infer_transport("tool.py").expect("stdio"),
            TransportKind::Stdio
        );
        assert_eq!(
            infer_transport(r#"{"mcpServers":{"weather":{}}}"#).expect("config"),
            TransportKind::Config
        );
        assert_eq!(
            infer_transport("memory").expect("memory"),
            TransportKind::Memory
        );
    }
}
