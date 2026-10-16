//! Icons and the server card a client can show.
//!
//! `src` has to be `http:`, `https:`, or `data:`. Nothing is fetched.

use serde_json::{json, Value};

/// `light` or `dark`. Absent means either theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconTheme {
    Light,
    Dark,
}

impl IconTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// One icon. `sizes` entries are `WxH` or `any`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    pub src: String,
    pub mime_type: Option<String>,
    pub sizes: Vec<String>,
    pub theme: Option<IconTheme>,
}

/// The name, site, and icons a server advertises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCard {
    pub name: String,
    pub website_url: Option<String>,
    pub icons: Vec<Icon>,
}

/// Check one icon.
///
/// # Errors
///
/// `src` is empty or does not start with `https://`, `http://`, or `data:`.
/// A size that is neither `any` nor `WxH` is an error.
pub fn check_icon(icon: &Icon) -> Result<(), String> {
    if icon.src.is_empty() {
        return Err("icon src is empty".to_string());
    }
    let allowed = icon.src.starts_with("https://")
        || icon.src.starts_with("http://")
        || icon.src.starts_with("data:");
    if !allowed {
        return Err("icon src must be https, http, or data".to_string());
    }
    for size in &icon.sizes {
        if size != "any" && !is_pixel_size(size) {
            return Err(format!("icon size must be WxH or any: {size}"));
        }
    }
    Ok(())
}

/// JSON for a server card. Icons that fail [`check_icon`] are left out.
pub fn server_card_json(card: &ServerCard) -> Value {
    let icons: Vec<Value> = card
        .icons
        .iter()
        .filter(|icon| check_icon(icon).is_ok())
        .map(|icon| {
            json!({
                "src": icon.src,
                "mimeType": icon.mime_type,
                "sizes": icon.sizes,
                "theme": icon.theme.map(IconTheme::as_str),
            })
        })
        .collect();
    json!({
        "name": card.name,
        "websiteUrl": card.website_url,
        "icons": icons,
    })
}

fn is_pixel_size(size: &str) -> bool {
    let Some((width, height)) = size.split_once('x') else {
        return false;
    };
    !width.is_empty()
        && !height.is_empty()
        && width.bytes().all(|byte| byte.is_ascii_digit())
        && height.bytes().all(|byte| byte.is_ascii_digit())
}

/// How many icons in the slice [`check_icon`] accepts.
pub fn count_ok(icons: &[Icon]) -> usize {
    icons.iter().filter(|icon| check_icon(icon).is_ok()).count()
}
