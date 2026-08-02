//! Normalization of paths that arrive from, or are sent to, the harness.
//!
//! The harness may express paths as `file://` or `cns://` URIs. Every consumer
//! on this side — the policy layer, hooks, and the workspace list we hand back
//! to the harness — needs native paths, so URIs are normalized at the boundary.
//!
//! Mirrors upstream `normalize_wire_path`, which has existed since 0.1.1
//! (`local_connection.py:215-222`, `cns://` added in 0.1.3) and lives at
//! `local_connection_config.py:68-84` in 0.1.9.
//!
//! Hand-rolled rather than pulling in `url`/`percent-encoding`: it is ~60 lines
//! and the crate must keep building for `wasm32`.

/// Tool-call argument keys that carry wire-format paths.
///
/// Mirrors upstream's `WIRE_PATH_ARGUMENT_KEYS`
/// (`local_connection_config.py:63-66`). Upstream uses a `frozenset`, whose
/// iteration order is arbitrary; this ordered slice is a deliberate divergence
/// that makes `canonical_path` derivation deterministic. It is unobservable for
/// the built-in tools — none of them carries two of these keys.
pub const WIRE_PATH_ARGUMENT_KEYS: [&str; 4] =
    ["path", "file_path", "TargetFile", "directory_path"];

/// Converts a wire-format path into a native path.
///
/// `file://` URIs are percent-decoded (and, on Windows, separator-swapped);
/// `cns://cell/rest` becomes `/cns/cell/rest`. Anything without an RFC-3986
/// scheme — including ordinary absolute and relative paths — passes through
/// unchanged.
#[must_use]
pub fn normalize_wire_path(path: &str) -> String {
    match split_uri(path) {
        Some((scheme, _netloc, p)) if scheme == "file" => url2pathname(p),
        // urlparse("cns://el-d/home/x") gives netloc "el-d" and path "/home/x".
        Some((scheme, netloc, p)) if scheme == "cns" => format!("/cns/{netloc}{p}"),
        _ => path.to_string(),
    }
}

/// Minimal `urllib.parse.urlparse` split into (lowercased scheme, netloc, path).
///
/// Returns `None` when there is no RFC-3986 scheme, so plain paths — and
/// Windows drive letters, whose "scheme" would be a single character followed
/// by a non-`//` remainder — pass through untouched.
fn split_uri(s: &str) -> Option<(String, &str, &str)> {
    let colon = s.find(':')?;
    let scheme = s.get(..colon)?;
    let mut chars = scheme.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    let rest = s.get(colon + 1..)?;
    // urlparse strips the fragment, then the query.
    let rest = rest.split_once('#').map_or(rest, |(head, _)| head);
    let rest = rest.split_once('?').map_or(rest, |(head, _)| head);
    let (netloc, path) = rest.strip_prefix("//").map_or(("", rest), |r| {
        r.find('/').map_or((r, ""), |i| (&r[..i], &r[i..]))
    });
    Some((scheme.to_ascii_lowercase(), netloc, path))
}

/// `urllib.request.url2pathname`.
///
/// POSIX: percent-decode only. Windows (`nturl2path`): also swap separators and
/// strip the leading slash that precedes a drive letter.
fn url2pathname(p: &str) -> String {
    let decoded = percent_decode(p);
    #[cfg(windows)]
    {
        let swapped = decoded.replace('/', "\\");
        let bytes = swapped.as_bytes();
        if bytes.len() >= 3
            && bytes[0] == b'\\'
            && (bytes[1] as char).is_ascii_alphabetic()
            && bytes[2] == b':'
        {
            return swapped.get(1..).unwrap_or("").to_string();
        }
        swapped
    }
    #[cfg(not(windows))]
    {
        decoded
    }
}

/// Byte-level percent decoding, then lossy UTF-8.
///
/// Multi-byte characters are percent-encoded one byte at a time, so decoding
/// must happen over bytes rather than chars. An invalid escape passes through
/// literally, matching Python's `unquote("%zz") == "%zz"`.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            )
        {
            #[allow(clippy::cast_possible_truncation)]
            out.push((hi * 16 + lo) as u8);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Rewrites every [`WIRE_PATH_ARGUMENT_KEYS`] entry of a JSON object in place.
///
/// Non-objects and non-string values are left untouched. Mirrors
/// `hook_router.py:63-74`.
pub fn normalize_path_args(args: &mut serde_json::Value) {
    let Some(map) = args.as_object_mut() else {
        return;
    };
    for key in WIRE_PATH_ARGUMENT_KEYS {
        let normalized = match map.get(key) {
            Some(serde_json::Value::String(s)) if !s.is_empty() => Some(normalize_wire_path(s)),
            _ => None,
        };
        if let Some(value) = normalized {
            map.insert(key.to_string(), serde_json::Value::String(value));
        }
    }
}

/// Returns the first non-empty [`WIRE_PATH_ARGUMENT_KEYS`] string, in slice order.
///
/// Mirrors `hook_router.py:193-200`.
#[must_use]
pub fn canonical_path_from_args(args: &serde_json::Value) -> Option<String> {
    let map = args.as_object()?;
    WIRE_PATH_ARGUMENT_KEYS
        .iter()
        .find_map(|key| match map.get(*key) {
            Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The three cases upstream pins in `event_processor_test.py:35-53`.
    #[test]
    fn upstream_normalization_cases() {
        assert_eq!(
            normalize_wire_path("file:///dev/shm/workspace/foo.py"),
            "/dev/shm/workspace/foo.py"
        );
        assert_eq!(
            normalize_wire_path("cns://el-d/home/user/workspace/kittens.md"),
            "/cns/el-d/home/user/workspace/kittens.md"
        );
        assert_eq!(normalize_wire_path("/tmp/clean-path"), "/tmp/clean-path");
    }

    #[test]
    fn percent_escapes_are_decoded() {
        assert_eq!(
            normalize_wire_path("file:///tmp/my%20dir/a%2Bb.py"),
            "/tmp/my dir/a+b.py"
        );
        // Multi-byte characters are encoded per byte.
        assert_eq!(
            normalize_wire_path("file:///tmp/caf%C3%A9/x.py"),
            "/tmp/café/x.py"
        );
        // An invalid escape survives literally, as Python's unquote does.
        assert_eq!(normalize_wire_path("file:///tmp/%zz"), "/tmp/%zz");
    }

    #[test]
    fn netloc_query_and_fragment_are_dropped() {
        assert_eq!(normalize_wire_path("file://localhost/tmp/x"), "/tmp/x");
        assert_eq!(normalize_wire_path("file:///tmp/a?q=1"), "/tmp/a");
        assert_eq!(normalize_wire_path("file:///tmp/a#frag"), "/tmp/a");
    }

    #[test]
    fn non_uris_pass_through() {
        for input in ["relative/path.py", "/abs/path.py", "C:/win/path", ""] {
            assert_eq!(normalize_wire_path(input), input);
        }
    }

    #[test]
    fn unknown_schemes_pass_through() {
        assert_eq!(
            normalize_wire_path("https://example.com/x"),
            "https://example.com/x"
        );
    }

    #[test]
    fn path_args_are_normalized_in_place() {
        let mut args = json!({
            "file_path": "file:///tmp/ws/foo.py",
            "directory_path": "cns://cell/dir",
            "query": "file:///not/a/path/key",
            "start_line": 3,
        });
        normalize_path_args(&mut args);
        assert_eq!(args["file_path"], "/tmp/ws/foo.py");
        assert_eq!(args["directory_path"], "/cns/cell/dir");
        // Keys outside WIRE_PATH_ARGUMENT_KEYS are untouched.
        assert_eq!(args["query"], "file:///not/a/path/key");
        assert_eq!(args["start_line"], 3);
    }

    #[test]
    fn normalize_path_args_ignores_non_objects() {
        let mut args = json!("file:///tmp/x");
        normalize_path_args(&mut args);
        assert_eq!(args, json!("file:///tmp/x"));
    }

    #[test]
    fn canonical_path_takes_the_first_key_in_slice_order() {
        assert_eq!(
            canonical_path_from_args(&json!({"file_path": "/a", "directory_path": "/b"})),
            Some("/a".to_string())
        );
        assert_eq!(
            canonical_path_from_args(&json!({"directory_path": "/b"})),
            Some("/b".to_string())
        );
    }

    /// An absent field serialises to `null` and an empty string is "no path";
    /// both must yield `None`, which the policy layer already treats as allow.
    #[test]
    fn absent_and_empty_paths_yield_none() {
        assert_eq!(canonical_path_from_args(&json!({"file_path": null})), None);
        assert_eq!(canonical_path_from_args(&json!({"file_path": ""})), None);
        assert_eq!(canonical_path_from_args(&json!({"prompt": "hi"})), None);
        assert_eq!(canonical_path_from_args(&json!([1, 2])), None);
    }
}
