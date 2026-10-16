//! Streamable HTTP on `127.0.0.1`.
//!
//! A handshake `initialize` opens a session. The reply is an SSE event and
//! the response carries `Mcp-Session-Id`. A `2026-07-28` request carries its
//! version on the call and does not open a session. `rmcp` makes that split.
//! This module binds the listener and posts the bytes the tests read back.

use std::io::{self, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

use crate::server::FastMcp;

const ACCEPT: &str = "application/json, text/event-stream";

/// One loopback listener. [`HttpServer::shutdown`] stops accepting.
pub struct HttpServer {
    url: String,
    cancel: CancellationToken,
}

impl HttpServer {
    /// `http://127.0.0.1:<port>/mcp`
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Ask the listener to stop. In-flight replies finish.
    pub fn shutdown(self) {
        self.cancel.cancel();
    }
}

impl FastMcp {
    /// Bind `127.0.0.1:0` and serve `/mcp`.
    ///
    /// JSON is preferred when a reply is a single message. A handshake
    /// `initialize` still answers with SSE, because that is how the session
    /// is created. Keep-alives are off so that one reply ends the body.
    ///
    /// # Errors
    ///
    /// Returns the bind error from the operating system.
    pub async fn serve_http(self) -> io::Result<HttpServer> {
        let server = self;
        let cancel = CancellationToken::new();
        let config = StreamableHttpServerConfig::default()
            .with_json_response(true)
            .with_legacy_session_mode(true)
            .with_sse_keep_alive(None)
            .with_sse_retry(None)
            .with_cancellation_token(cancel.clone());
        let service: StreamableHttpService<FastMcp, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(server.clone()),
                Arc::new(LocalSessionManager::default()),
                config,
            );
        let router = axum::Router::new().nest_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let cancel_for_serve = cancel.clone();
        tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    cancel_for_serve.cancelled().await;
                })
                .await;
        });
        Ok(HttpServer {
            url: format!("http://{addr}/mcp"),
            cancel,
        })
    }
}

/// Status, headers, and the decoded body of one HTTP reply.
#[derive(Debug)]
pub struct HttpReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpReply {
    /// The header value, compared without case.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The JSON-RPC object. An SSE body contributes its first `data:` line.
    ///
    /// # Errors
    ///
    /// Returns the `serde_json` error when the payload is not JSON.
    pub fn rpc(&self) -> Result<Value, serde_json::Error> {
        let payload = if self
            .header("content-type")
            .is_some_and(|value| value.contains("text/event-stream"))
        {
            sse_data(&self.body).unwrap_or("")
        } else {
            self.body.as_str()
        };
        serde_json::from_str(payload)
    }
}

/// `POST` JSON to a Streamable HTTP URL.
///
/// `Accept` and `Content-Type` are set here. `extra` is appended after them,
/// so a session id and `MCP-Protocol-Version` ride along.
///
/// # Errors
///
/// Returns an I/O error when the socket fails, the reply is truncated, or
/// five seconds pass.
pub async fn post_mcp(url: &str, extra: &[(&str, &str)], body: &str) -> io::Result<HttpReply> {
    exchange("POST", url, extra, body).await
}

/// `DELETE` a handshake session.
///
/// # Errors
///
/// Returns an I/O error when the socket fails.
pub async fn delete_mcp(url: &str, extra: &[(&str, &str)]) -> io::Result<HttpReply> {
    exchange("DELETE", url, extra, "").await
}

async fn exchange(
    method: &str,
    url: &str,
    extra: &[(&str, &str)],
    body: &str,
) -> io::Result<HttpReply> {
    let (host, path) = split_http_url(url)?;
    tokio::time::timeout(Duration::from_secs(5), async move {
        let mut stream = TcpStream::connect(host).await?;
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nAccept: {ACCEPT}\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, value) in extra {
            request.push_str(name);
            request.push_str(": ");
            request.push_str(value);
            request.push_str("\r\n");
        }
        request.push_str("\r\n");
        request.push_str(body);
        stream.write_all(request.as_bytes()).await?;
        let bytes = read_to_end(&mut stream).await?;
        parse_response(&bytes)
    })
    .await
    .map_err(|_| io::Error::new(ErrorKind::TimedOut, "mcp http exchange"))?
}

fn split_http_url(url: &str) -> io::Result<(&str, String)> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "url must start with http://"))?;
    let (host, path) = rest
        .split_once('/')
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "url needs a path"))?;
    Ok((host, format!("/{path}")))
}

async fn read_to_end(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Ok(buf);
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

fn parse_response(bytes: &[u8]) -> io::Result<HttpReply> {
    let header_end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "http headers"))?;
    let head = std::str::from_utf8(&bytes[..header_end])
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    let mut lines = head.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "status line"))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "status code"))?;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    let raw = &bytes[header_end + 4..];
    let decoded = if headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("transfer-encoding") && value.contains("chunked")
    }) {
        decode_chunked(raw)?
    } else {
        raw.to_vec()
    };
    let body = String::from_utf8(decoded)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    Ok(HttpReply {
        status,
        headers,
        body,
    })
}

fn decode_chunked(mut input: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    while !input.is_empty() {
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "chunk size"))?;
        let size_line = std::str::from_utf8(&input[..line_end])
            .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
        let size_hex = size_line.split(';').next().unwrap_or(size_line).trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
        input = &input[line_end + 2..];
        if size == 0 {
            break;
        }
        if input.len() < size + 2 {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "chunk data"));
        }
        out.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
    Ok(out)
}

fn sse_data(body: &str) -> Option<&str> {
    body.lines().find_map(|line| {
        let rest = line.strip_prefix("data:")?.trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunked_body_joins_the_pieces() {
        let raw = b"5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
        let decoded = decode_chunked(raw).expect("chunks");
        assert_eq!(decoded, b"hello world");
    }

    #[test]
    fn sse_skips_an_empty_priming_line() {
        let body = "data:\n\ndata: {\"ok\":true}\n\n";
        assert_eq!(sse_data(body), Some("{\"ok\":true}"));
    }
}
