//! Static resources and `{param}` URI templates.
//!
//! `match_template` borrows the pattern and the URI. The names it returns
//! borrow the pattern. The values borrow the URI. The server copies them
//! into `String`s before it calls a template's function, because that
//! function outlives the request.

use std::sync::Arc;

/// One capture: the parameter name, borrowed from the pattern, and the
/// value, borrowed from the URI.
pub fn match_template<'p, 'u>(pattern: &'p str, uri: &'u str) -> Option<Vec<(&'p str, &'u str)>> {
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

#[derive(Clone)]
pub struct StaticResource {
    pub uri: String,
    pub name: String,
    pub mime: String,
    pub body: String,
}

type RenderFn = dyn Fn(&[(String, String)]) -> Result<String, String> + Send + Sync;

#[derive(Clone)]
pub struct UriTemplate {
    pub pattern: String,
    pub name: String,
    pub mime: String,
    render: Arc<RenderFn>,
}

impl UriTemplate {
    pub fn new<F>(
        pattern: impl Into<String>,
        name: impl Into<String>,
        mime: impl Into<String>,
        render: F,
    ) -> Self
    where
        F: Fn(&[(String, String)]) -> Result<String, String> + Send + Sync + 'static,
    {
        Self {
            pattern: pattern.into(),
            name: name.into(),
            mime: mime.into(),
            render: Arc::new(render),
        }
    }

    pub fn render(&self, uri: &str) -> Option<Result<String, String>> {
        let captures = match_template(&self.pattern, uri)?;
        let owned: Vec<(String, String)> = captures
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        Some((self.render)(&owned))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_segment_borrows_the_uri() {
        let pattern = "file://{name}";
        let uri = "file://readme";
        let captures = match_template(pattern, uri).expect("match");
        assert_eq!(captures, vec![("name", "readme")]);
    }

    #[test]
    fn two_segments_keep_their_names() {
        let captures = match_template("res://{owner}/{repo}", "res://acme/widgets").expect("match");
        assert_eq!(captures, vec![("owner", "acme"), ("repo", "widgets")]);
    }

    #[test]
    fn a_slash_does_not_fit_one_segment() {
        assert!(match_template("file://{name}", "file://a/b").is_none());
    }

    #[test]
    fn a_fragment_is_not_part_of_the_match() {
        let captures = match_template("file://{name}", "file://readme#top").expect("match");
        assert_eq!(captures[0].1, "readme");
    }
}
