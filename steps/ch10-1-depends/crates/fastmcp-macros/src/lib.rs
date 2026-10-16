//! `#[tool]` turns a function into a registration the `FastMcp` builder can take.
//!
//! The macro runs while the program compiles. It leaves the function in place
//! and emits a unit struct that knows the name, the description, the argument
//! struct, and how to call the function.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, LitStr, Pat, Token, Type};

struct ToolAttr {
    name: Option<String>,
    description: Option<String>,
}

impl Parse for ToolAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            if key == "name" {
                name = Some(value.value());
            } else if key == "description" {
                description = Some(value.value());
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "expected `name = \"...\"` or `description = \"...\"`",
                ));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { name, description })
    }
}

/// Register `fn` as a tool. The function stays callable. A sibling struct
/// `<Pascal>Tool` implements [`fastmcp_core::ToolDecl`].
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as ToolAttr);
    let function = parse_macro_input!(item as ItemFn);
    expand(attrs, function)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(attrs: ToolAttr, function: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    if function.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            function.sig.fn_token,
            "#[tool] functions in this chapter are synchronous",
        ));
    }
    let fn_name = &function.sig.ident;
    let tool_name = attrs.name.unwrap_or_else(|| fn_name.to_string());
    let description = attrs
        .description
        .or_else(|| doc_comment(&function))
        .unwrap_or_else(|| tool_name.clone());
    let struct_name = Ident::new(
        &format!("{}Tool", pascal(&fn_name.to_string())),
        fn_name.span(),
    );
    let args_name = Ident::new(&format!("__{fn_name}_args"), fn_name.span());

    let mut fields = Vec::new();
    let mut field_names = Vec::new();
    for arg in &function.sig.inputs {
        match arg {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new_spanned(
                    receiver,
                    "#[tool] takes named arguments, not self",
                ));
            }
            FnArg::Typed(pat_type) => {
                let name = match pat_type.pat.as_ref() {
                    Pat::Ident(ident) => ident.ident.clone(),
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "#[tool] arguments are `name: Type`",
                        ));
                    }
                };
                let ty = pat_type.ty.clone();
                field_names.push(name.clone());
                fields.push(quote! { #name: #ty });
            }
        }
    }

    let output = match &function.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &function.sig,
                "#[tool] needs a return type",
            ));
        }
        syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
    };
    let (ok_ty, call) = if let Some(inner) = result_ok_type(&output) {
        let names = &field_names;
        (inner, quote! { #fn_name(#(args.#names),*) })
    } else {
        let names = &field_names;
        (output, quote! { Ok(#fn_name(#(args.#names),*)) })
    };

    let tool_name_lit = LitStr::new(&tool_name, fn_name.span());
    let description_lit = LitStr::new(&description, fn_name.span());

    Ok(quote! {
        #function

        #[derive(serde::Deserialize, schemars::JsonSchema)]
        #[allow(dead_code, non_camel_case_types)]
        pub struct #args_name {
            #(#fields,)*
        }

        #[allow(non_camel_case_types)]
        pub struct #struct_name;

        impl fastmcp_core::ToolDecl for #struct_name {
            type Params = #args_name;
            type Output = #ok_ty;

            fn name() -> &'static str {
                #tool_name_lit
            }

            fn description() -> &'static str {
                #description_lit
            }

            fn invoke(args: Self::Params) -> Result<Self::Output, String> {
                #call
            }
        }
    })
}

fn result_ok_type(ty: &Type) -> Option<Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    }
}

fn doc_comment(function: &ItemFn) -> Option<String> {
    let lines: Vec<String> = function
        .attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            let syn::Meta::NameValue(name_value) = &attr.meta else {
                return None;
            };
            let syn::Expr::Lit(expr) = &name_value.value else {
                return None;
            };
            let syn::Lit::Str(text) = &expr.lit else {
                return None;
            };
            let line = text.value();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

fn pascal(name: &str) -> String {
    let mut out = String::new();
    for part in name.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        out.push_str("Tool");
    }
    out
}
