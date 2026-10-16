use mcp_icons::{server_card_json, Icon, IconTheme, ServerCard};

fn main() {
    let card = ServerCard {
        name: "widgets".to_string(),
        website_url: Some("https://example.test".to_string()),
        icons: vec![Icon {
            src: "https://example.test/icon.png".to_string(),
            mime_type: Some("image/png".to_string()),
            sizes: vec!["48x48".to_string()],
            theme: Some(IconTheme::Light),
        }],
    };
    let json = server_card_json(&card);
    println!("name: {}", json["name"].as_str().expect("name"));
    println!("website: {}", json["websiteUrl"].as_str().expect("website"));
    println!("src: {}", json["icons"][0]["src"].as_str().expect("src"));
    println!(
        "theme: {}",
        json["icons"][0]["theme"].as_str().expect("theme")
    );
}
