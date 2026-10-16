use mcp_icons::{check_icon, server_card_json, Icon, IconTheme, ServerCard};

fn icon(src: &str) -> Icon {
    Icon {
        src: src.to_string(),
        mime_type: Some("image/png".to_string()),
        sizes: vec!["48x48".to_string()],
        theme: Some(IconTheme::Light),
    }
}

#[test]
fn an_https_icon_is_kept_and_a_script_src_is_not() {
    check_icon(&icon("https://example.test/icon.png")).expect("https");
    check_icon(&icon("data:image/png;base64,aaaa")).expect("data");
    let err = check_icon(&icon("javascript:alert(1)")).expect_err("script");
    assert_eq!(err, "icon src must be https, http, or data");
    let mut bad_size = icon("https://example.test/icon.png");
    bad_size.sizes = vec!["big".to_string()];
    assert!(check_icon(&bad_size).is_err());
}

#[test]
fn the_server_card_drops_a_bad_icon() {
    let card = ServerCard {
        name: "widgets".to_string(),
        website_url: Some("https://example.test".to_string()),
        icons: vec![
            icon("https://example.test/icon.png"),
            icon("file:///tmp/icon.png"),
        ],
    };
    let json = server_card_json(&card);
    assert_eq!(json["name"], "widgets");
    assert_eq!(json["websiteUrl"], "https://example.test");
    assert_eq!(json["icons"].as_array().expect("icons").len(), 1);
    assert_eq!(json["icons"][0]["theme"], "light");
    assert_eq!(IconTheme::Light.as_str(), "light");
}
