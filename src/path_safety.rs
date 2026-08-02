//! Path resolution and workspace containment for the policy layer.
//!
//! Deciding whether a model-supplied path is inside a workspace is what keeps
//! the sandbox honest, and it cannot be done on the raw string:
//! `Path::starts_with` is purely lexical, so `<ws>/../../etc/passwd` and a
//! symlink out of the workspace both test as *inside*.
//!
//! Mirrors upstream `hooks/policy.py:437-506` — `_secure_normalize_path`,
//! `_is_case_insensitive` and `_is_path_in_workspace`.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Matches the usual kernel limit on symlink traversals in one resolution.
const MAX_SYMLINK_HOPS: u32 = 40;

/// Whether this platform's filesystem is case-insensitive by default.
const PLATFORM_CASE_INSENSITIVE: bool = cfg!(any(windows, target_os = "macos"));

enum Seg {
    Cur,
    Parent,
    Normal(OsString),
}

enum Probe {
    Missing,
    Regular,
    Symlink,
    Denied(io::Error),
}

/// Splits a path into its root (prefix + root directory, possibly empty) and
/// owned segments.
///
/// The segments must be owned: `Component<'a>` borrows the path, and symlink
/// targets are pushed back onto the queue as resolution proceeds.
fn split_root(path: &Path) -> (PathBuf, VecDeque<Seg>) {
    let mut root = PathBuf::new();
    let mut segments = VecDeque::new();
    for component in path.components() {
        match component {
            // On Windows, pushing "C:" then "\\" yields `C:\`: `PathBuf::push`
            // keeps the prefix when the pushed path has a root but no prefix.
            Component::Prefix(_) | Component::RootDir => root.push(component.as_os_str()),
            Component::CurDir => segments.push_back(Seg::Cur),
            Component::ParentDir => segments.push_back(Seg::Parent),
            Component::Normal(name) => segments.push_back(Seg::Normal(name.to_os_string())),
        }
    }
    (root, segments)
}

fn probe(path: &Path) -> Probe {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Probe::Symlink,
        Ok(_) => Probe::Regular,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Probe::Missing,
        // `wasm32-unknown-unknown` has no filesystem, so every call is
        // `Unsupported`. Failing closed there would deny every file tool, so
        // degrade to lexical-only normalization instead — `..` collapsing still
        // works, symlink resolution provably cannot.
        Err(e) if e.kind() == io::ErrorKind::Unsupported => Probe::Missing,
        // EACCES, ELOOP, ENOTDIR, ENAMETOOLONG: fail closed.
        Err(e) => Probe::Denied(e),
    }
}

/// Resolves a path the way `pathlib.Path(p).resolve()` does: `.` and `..`
/// collapsed, symlinks followed, and a non-existent leaf (or intermediate
/// directory) still resolved.
///
/// Non-strict by design — `create_file` targets a path that does not exist yet,
/// so requiring existence (as [`std::fs::canonicalize`] does) is unusable here.
/// Upstream makes the same choice for the same reason (`policy.py:437-449`).
///
/// A relative path resolves against the current directory, matching upstream.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when a component cannot be inspected
/// (permission denied, symlink loop, a non-directory used as one). Callers are
/// expected to treat an error as "not in the workspace".
pub fn secure_normalize_path(path: &str) -> io::Result<PathBuf> {
    // Defence in depth: idempotent for anything that is not a file:// or cns://
    // URI, and it protects this path if an extractor arm is ever missed.
    let path = crate::wire_path::normalize_wire_path(path);
    let candidate = Path::new(&path);
    let absolute: PathBuf = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        std::env::current_dir()?.join(candidate)
    };

    let (root, mut pending) = split_root(&absolute);
    let mut resolved = root;
    let mut hops: u32 = 0;

    while let Some(segment) = pending.pop_front() {
        match segment {
            Seg::Cur => {}
            Seg::Parent => {
                // `pop()` at the root is a no-op, matching Python:
                // Path("/..").resolve() == PosixPath("/").
                resolved.pop();
            }
            Seg::Normal(name) => {
                resolved.push(&name);
                match probe(&resolved) {
                    Probe::Denied(e) => return Err(e),
                    Probe::Missing | Probe::Regular => {}
                    Probe::Symlink => {
                        hops += 1;
                        if hops > MAX_SYMLINK_HOPS {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "too many levels of symbolic links",
                            ));
                        }
                        let target = std::fs::read_link(&resolved)?;
                        resolved.pop();
                        let (target_root, target_segments) = split_root(&target);
                        if target.is_absolute() {
                            resolved = target_root;
                        }
                        for segment in target_segments.into_iter().rev() {
                            pending.push_front(segment);
                        }
                    }
                }
            }
        }
    }

    Ok(resolved)
}

/// Whether paths under `path` compare case-insensitively.
///
/// Returns the platform default. Upstream probes the filesystem
/// (`policy.py:452-479`) and falls back to the same default whenever the path
/// does not exist; the residual difference is a case-sensitive volume on macOS
/// (folds when it need not — an under-deny we inherit from the default) and a
/// case-insensitive mount on Linux (does not fold — an over-deny, which is
/// safe).
#[must_use]
pub const fn is_case_insensitive(path: &Path) -> bool {
    let _ = path;
    PLATFORM_CASE_INSENSITIVE
}

/// Whether `target` resolves to a location inside `workspace`.
///
/// Both sides are resolved first, then compared component by component, so a
/// sibling directory sharing a name prefix (`/tmp/ws-evil` against `/tmp/ws`)
/// is correctly outside. Fails closed: any resolution error yields `false`.
///
/// Mirrors `policy.py:483-506`.
#[must_use]
pub fn is_path_in_workspace(target: &str, workspace: &str) -> bool {
    let (Ok(target), Ok(workspace)) = (
        secure_normalize_path(target),
        secure_normalize_path(workspace),
    ) else {
        // policy.py:490-492 — an OSError on either side is "outside".
        return false;
    };

    // Upstream probes the workspace, not the target (policy.py:494).
    let fold = is_case_insensitive(&workspace);
    let key = |component: Component<'_>| {
        let part = component.as_os_str().to_string_lossy().into_owned();
        if fold { part.to_lowercase() } else { part }
    };

    let target_parts: Vec<String> = target.components().map(key).collect();
    let workspace_parts: Vec<String> = workspace.components().map(key).collect();

    if target_parts.len() < workspace_parts.len() {
        return false; // policy.py:501-502
    }
    target_parts
        .iter()
        .zip(workspace_parts.iter())
        .all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds `<tmp>/ws/sub/file.txt` plus `<tmp>/outside/evil.txt` and a
    /// symlink `<tmp>/ws/link -> <tmp>/outside`, and returns `<tmp>`.
    fn fixture(tag: &str) -> Option<PathBuf> {
        let root = std::env::temp_dir().join(format!("ag_path_safety_{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("ws/sub")).ok()?;
        std::fs::create_dir_all(root.join("outside")).ok()?;
        std::fs::write(root.join("ws/sub/file.txt"), b"x").ok()?;
        std::fs::write(root.join("outside/evil.txt"), b"x").ok()?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("outside"), root.join("ws/link")).ok()?;
        Some(root)
    }

    fn as_str(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    /// The escape this whole module exists to close. Before it,
    /// `Path::starts_with` reported both of these as inside the workspace.
    #[test]
    fn parent_traversal_escapes_are_outside() {
        let Some(root) = fixture("traversal") else {
            return;
        };
        let ws = as_str(&root.join("ws"));

        assert!(!is_path_in_workspace(
            &as_str(&root.join("ws/../../etc/passwd")),
            &ws
        ));
        assert!(!is_path_in_workspace(
            &as_str(&root.join("ws/./../secret")),
            &ws
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_ancestor_is_outside() {
        let Some(root) = fixture("symlink") else {
            return;
        };
        let ws = as_str(&root.join("ws"));
        assert!(!is_path_in_workspace(
            &as_str(&root.join("ws/link/evil.txt")),
            &ws
        ));
    }

    /// `create_file` targets a path that does not exist yet — resolution must
    /// not require existence, at the leaf or at an intermediate directory.
    #[test]
    fn nonexistent_paths_still_resolve_inside() {
        let Some(root) = fixture("newfile") else {
            return;
        };
        let ws = as_str(&root.join("ws"));
        assert!(is_path_in_workspace(
            &as_str(&root.join("ws/newdir/newfile.txt")),
            &ws
        ));
        assert!(is_path_in_workspace(
            &as_str(&root.join("ws/sub/file.txt")),
            &ws
        ));
    }

    /// The prefix attack Rust's component-wise `starts_with` already handled;
    /// keep it handled.
    #[test]
    fn sibling_with_shared_prefix_is_outside() {
        let Some(root) = fixture("prefix") else {
            return;
        };
        let ws = as_str(&root.join("ws"));
        assert!(!is_path_in_workspace(
            &as_str(&root.join("ws-evil/file.txt")),
            &ws
        ));
    }

    #[test]
    fn workspace_itself_is_inside() {
        let Some(root) = fixture("self") else {
            return;
        };
        let ws = as_str(&root.join("ws"));
        assert!(is_path_in_workspace(&ws, &ws));
    }

    /// Upstream resolves a relative path against the cwd rather than rejecting
    /// it outright (`policy.py:437-449` has no absoluteness precondition).
    #[test]
    fn relative_paths_resolve_against_cwd() {
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        assert!(is_path_in_workspace("Cargo.toml", &as_str(&cwd)));
        assert!(!is_path_in_workspace("../outside.txt", &as_str(&cwd)));
    }

    #[test]
    fn parent_of_root_clamps_to_root() {
        let Ok(resolved) = secure_normalize_path("/..") else {
            return;
        };
        assert_eq!(resolved, Path::new("/"));
    }

    /// A wire-format URI must resolve like the native path it denotes, not be
    /// treated as a relative path and joined onto the cwd.
    #[test]
    fn wire_uris_are_normalized_before_resolution() {
        let Some(root) = fixture("uri") else {
            return;
        };
        let ws = as_str(&root.join("ws"));
        let uri = format!("file://{}", as_str(&root.join("ws/sub/file.txt")));
        assert!(is_path_in_workspace(&uri, &ws));
    }
}
