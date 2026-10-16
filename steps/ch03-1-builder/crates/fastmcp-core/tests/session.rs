//! One in-memory pipe, the same read/write transport stdio uses.

use fastmcp_core::{era_note, era_of, FastMcp};
use rmcp::model::ProtocolVersion;
use rmcp::{model::ClientConfig, ClientHandler, ClientLifecycleMode, ClientServiceExt, ServiceExt};

struct VersionClient {
    version: ProtocolVersion,
}

impl ClientHandler for VersionClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = self.version.clone();
        info
    }
}

async fn handshake(version: ProtocolVersion) -> ProtocolVersion {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = FastMcp::new("fastmcp-rs")
            .serve(server_io)
            .await
            .expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient { version }
        .serve(client_io)
        .await
        .expect("client");
    let negotiated = client
        .peer_info()
        .expect("peer info")
        .protocol_version
        .clone();
    let tools = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("tools/list");
    let description = tools.tools[0].description.clone().expect("description");
    assert_eq!(description.as_ref(), era_note(era_of(&negotiated)));
    client.cancel().await.expect("cancel");
    negotiated
}

#[tokio::test]
async fn initialize_echoes_2025_06_18() {
    let negotiated = handshake(ProtocolVersion::V_2025_06_18).await;
    assert_eq!(negotiated, ProtocolVersion::V_2025_06_18);
}

#[tokio::test]
async fn initialize_does_not_agree_to_2026_07_28() {
    let negotiated = handshake(ProtocolVersion::V_2026_07_28).await;
    assert_eq!(negotiated, ProtocolVersion::V_2025_11_25);
}

#[tokio::test]
async fn discover_offers_2026_07_28() {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = FastMcp::new("fastmcp-rs")
            .serve(server_io)
            .await
            .expect("server");
        let _ = running.waiting().await;
    });
    let client = VersionClient {
        version: ProtocolVersion::V_2026_07_28,
    }
    .serve_with_lifecycle(
        client_io,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
        },
    )
    .await
    .expect("discover");
    let tools = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("tools/list");
    let description = tools.tools[0].description.clone().expect("description");
    assert_eq!(
        description.as_ref(),
        era_note(era_of(&ProtocolVersion::V_2026_07_28))
    );
    client.cancel().await.expect("cancel");
}
