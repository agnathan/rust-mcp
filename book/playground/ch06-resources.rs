//! `{name}` captures one path segment. The name borrows the pattern.
//! The value borrows the URI. No crates beyond std.

fn match_template<'p, 'u>(pattern: &'p str, uri: &'u str) -> Option<Vec<(&'p str, &'u str)>> {
    let uri = if pattern.contains('#') {
        uri
    } else {
        uri.split_once('#').map(|(path, _)| path).unwrap_or(uri)
    };
    let uri = uri.split_once('?').map(|(path, _)| path).unwrap_or(uri);

    let mut names = Vec::new();
    let mut rest_pattern = pattern;
    let mut rest_uri = uri;
    loop {
        let Some(index) = rest_pattern.find('{') else {
            return if rest_pattern == rest_uri {
                Some(names)
            } else {
                None
            };
        };
        let literal = &rest_pattern[..index];
        if !rest_uri.starts_with(literal) {
            return None;
        }
        rest_uri = &rest_uri[literal.len()..];
        let after = &rest_pattern[index + 1..];
        let close = after.find('}')?;
        let name = &after[..close];
        if name.is_empty() || name.starts_with('?') || name.ends_with('*') {
            return None;
        }
        rest_pattern = &after[close + 1..];
        let literal_len = rest_pattern.find('{').unwrap_or(rest_pattern.len());
        let following = &rest_pattern[..literal_len];
        let value_end = if following.is_empty() {
            rest_uri.find('/').unwrap_or(rest_uri.len())
        } else {
            rest_uri.find(following)?
        };
        let value = &rest_uri[..value_end];
        if value.is_empty() || value.contains('/') {
            return None;
        }
        rest_uri = &rest_uri[value_end..];
        names.push((name, value));
    }
}

fn main() {
    let pattern = "res://{owner}/{repo}";
    let uri = "res://acme/widgets#top";
    let captures = match_template(pattern, uri).expect("match");
    for (name, value) in captures {
        println!("{name}={value}");
    }
}
