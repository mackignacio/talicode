// SPDX-License-Identifier: MIT
//! Source selection — staged files (via `git`) or a target glob, read into
//! memory with binary/oversize files skipped.
//!
//! Implements #20. The `git` invocation is thin; the parsing and file-reading
//! logic is factored into pure functions so it is unit-tested without a repo.

use globset::Glob;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A source file read for auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path relative to the repo root.
    pub path: String,
    /// File contents.
    pub content: String,
}

/// A file that was selected but not read (with the reason), so nothing is
/// silently dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// Path relative to the repo root.
    pub path: String,
    /// Why it was skipped.
    pub reason: SkipReason,
}

/// Why a selected file was not audited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// Larger than [`MAX_FILE_BYTES`].
    TooLarge,
    /// Looked like binary content (contained a NUL byte).
    Binary,
    /// Could not be read.
    Unreadable,
}

/// Files above this size are skipped (auditing them is rarely useful and burns tokens).
pub const MAX_FILE_BYTES: u64 = 200 * 1024;

/// Errors from source selection.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// The `git` command failed to run.
    #[error("running git: {0}")]
    Git(String),
    /// The target glob was invalid.
    #[error("invalid target glob `{0}`: {1}")]
    Glob(String, String),
}

/// List staged, added/copied/modified files (relative paths) via git.
pub fn staged_files(repo_root: &Path) -> Result<Vec<PathBuf>, GitError> {
    let out = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| GitError::Git(e.to_string()))?;
    if !out.status.success() {
        return Err(GitError::Git(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(parse_name_only(&String::from_utf8_lossy(&out.stdout)))
}

/// Parse `git ... --name-only` stdout into paths (ignoring blank lines).
fn parse_name_only(stdout: &str) -> Vec<PathBuf> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Expand a target glob against the working tree under `repo_root`.
pub fn target_files(repo_root: &Path, glob: &str) -> Result<Vec<PathBuf>, GitError> {
    let matcher = Glob::new(glob)
        .map_err(|e| GitError::Glob(glob.to_string(), e.to_string()))?
        .compile_matcher();
    let mut out = Vec::new();
    for entry in WalkDir::new(repo_root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path());
        // Match against both the "./"-prefixed and bare relative path.
        let bare = rel.to_string_lossy();
        if matcher.is_match(rel) || matcher.is_match(format!("./{bare}")) {
            out.push(rel.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

/// Read the selected paths, skipping binary/oversize/unreadable files.
pub fn read_sources(repo_root: &Path, paths: &[PathBuf]) -> (Vec<SourceFile>, Vec<Skipped>) {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for path in paths {
        let rel = path.to_string_lossy().into_owned();
        let full = repo_root.join(path);
        match classify_and_read(&full) {
            Ok(content) => files.push(SourceFile { path: rel, content }),
            Err(reason) => skipped.push(Skipped { path: rel, reason }),
        }
    }
    (files, skipped)
}

fn classify_and_read(full: &Path) -> Result<String, SkipReason> {
    let meta = std::fs::metadata(full).map_err(|_| SkipReason::Unreadable)?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(SkipReason::TooLarge);
    }
    let bytes = std::fs::read(full).map_err(|_| SkipReason::Unreadable)?;
    if bytes.contains(&0) {
        return Err(SkipReason::Binary);
    }
    String::from_utf8(bytes).map_err(|_| SkipReason::Binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_only_output() {
        let out = "src/a.rs\n\nsrc/b.rs\n";
        assert_eq!(
            parse_name_only(out),
            vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]
        );
    }

    #[test]
    fn target_glob_matches_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/a.rs"), "fn a() {}").unwrap();
        std::fs::write(dir.path().join("src/b.txt"), "nope").unwrap();

        let hits = target_files(dir.path(), "**/*.rs").unwrap();
        assert_eq!(hits, vec![PathBuf::from("src/a.rs")]);
    }

    #[test]
    fn reads_text_and_skips_binary_and_oversize() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("t.rs"), "let x = 1;").unwrap();
        std::fs::write(dir.path().join("b.rs"), [0u8, 1, 2, 3]).unwrap();
        std::fs::write(
            dir.path().join("big.rs"),
            vec![b'a'; (MAX_FILE_BYTES + 1) as usize],
        )
        .unwrap();

        let paths = vec![
            PathBuf::from("t.rs"),
            PathBuf::from("b.rs"),
            PathBuf::from("big.rs"),
            PathBuf::from("missing.rs"),
        ];
        let (files, skipped) = read_sources(dir.path(), &paths);

        assert_eq!(
            files,
            vec![SourceFile {
                path: "t.rs".into(),
                content: "let x = 1;".into()
            }]
        );
        let reasons: Vec<_> = skipped
            .iter()
            .map(|s| (s.path.as_str(), s.reason))
            .collect();
        assert!(reasons.contains(&("b.rs", SkipReason::Binary)));
        assert!(reasons.contains(&("big.rs", SkipReason::TooLarge)));
        assert!(reasons.contains(&("missing.rs", SkipReason::Unreadable)));
    }
}
