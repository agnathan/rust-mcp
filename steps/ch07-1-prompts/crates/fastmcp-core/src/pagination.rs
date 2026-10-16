//! Opaque cursors for `list_*`.
//!
//! The cursor is a URL-safe base64 encoding of `{"o": offset}`. A client
//! sends it back. It does not parse it. A cursor that is not a non-negative
//! integer offset is `invalid_params` at the call site.

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use serde_json::Value;

/// One page, and the cursor for the page after it.
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

/// Encode `offset` the way FastMCP's `CursorState.encode` does.
pub fn encode_cursor(offset: usize) -> String {
    let data = serde_json::to_vec(&serde_json::json!({ "o": offset })).expect("offset json");
    URL_SAFE.encode(data)
}

/// Decode a cursor. Anything but a whole non-negative offset is an error.
pub fn decode_cursor(cursor: &str) -> Result<usize, String> {
    let invalid = || format!("Invalid cursor: {cursor}");
    let bytes = URL_SAFE.decode(cursor).map_err(|_| invalid())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let Some(offset) = value.get("o") else {
        return Err(invalid());
    };
    match offset {
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(invalid),
        _ => Err(invalid()),
    }
}

/// The items on one page. The return type is `impl Iterator` so the caller
/// does not have to name `Skip` and `Take`.
pub fn page_items<T>(items: &[T], offset: usize, page_size: usize) -> impl Iterator<Item = &T> {
    items.iter().skip(offset).take(page_size.max(1))
}

/// Slice `items` at the cursor. `next_cursor` is `None` on the last page.
pub fn paginate_sequence<T: Clone>(
    items: &[T],
    cursor: Option<&str>,
    page_size: usize,
) -> Result<Page<T>, String> {
    let offset = match cursor {
        Some(cursor) if !cursor.is_empty() => decode_cursor(cursor)?,
        _ => 0,
    };
    let page_size = page_size.max(1);
    let end = offset.saturating_add(page_size);
    let page_items = page_items(items, offset, page_size).cloned().collect();
    let next_cursor = if end < items.len() {
        Some(encode_cursor(end))
    } else {
        None
    };
    Ok(Page {
        items: page_items,
        next_cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cursor_round_trips_the_offset() {
        let cursor = encode_cursor(4);
        assert_eq!(decode_cursor(&cursor).expect("cursor"), 4);
    }

    #[test]
    fn the_second_page_starts_where_the_first_ended() {
        let items: Vec<_> = (0..5).collect();
        let first = paginate_sequence(&items, None, 2).expect("first");
        assert_eq!(first.items, vec![0, 1]);
        let second = paginate_sequence(&items, first.next_cursor.as_deref(), 2).expect("second");
        assert_eq!(second.items, vec![2, 3]);
        let third = paginate_sequence(&items, second.next_cursor.as_deref(), 2).expect("third");
        assert_eq!(third.items, vec![4]);
        assert!(third.next_cursor.is_none());
    }

    #[test]
    fn a_bool_offset_is_rejected() {
        let cursor = URL_SAFE.encode(br#"{"o":true}"#);
        assert!(decode_cursor(&cursor).is_err());
    }

    #[test]
    fn a_negative_offset_is_rejected() {
        let cursor = URL_SAFE.encode(br#"{"o":-1}"#);
        assert!(decode_cursor(&cursor).is_err());
    }
}
