// SPDX-License-Identifier: MIT
//! Architectural memory — a queryable map of the codebase.
//!
//! Implements #48. A native scan builds a lightweight map (file tree, top-level
//! symbols, imports) so the Auditor can consult structure instead of re-grepping
//! — cutting token spend. Extraction is heuristic per file extension (no AST);
//! an AST/dependency-graph backend and a `tali search` query command + Claude
//! Code hook are roadmap. `build`/`arch_lookup`/`overview` are pure; `scan`/
//! `save`/`load` do the IO.

use serde::{Deserialize, Serialize};
use std::path::Path;
use talicode_core::watch::is_ignored;

/// Repo-local dir for the map (committed — a team-shared project map).
pub const ARCH_DIR: &str = ".talicode";
/// The serialized codebase map.
pub const ARCH_FILE: &str = "architecture.json";
/// Skip files larger than this when scanning.
const MAX_FILE_BYTES: u64 = 512 * 1024;

/// The whole-codebase map.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArchMap {
    /// One entry per scanned source file, sorted by path.
    pub files: Vec<FileEntry>,
}

/// One file's extracted structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Repo-relative path.
    pub path: String,
    /// Top-level symbol names (functions/types/classes).
    pub symbols: Vec<String>,
    /// Imported modules/paths.
    pub imports: Vec<String>,
}

/// A hit from [`arch_lookup`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// The file the hit is in.
    pub path: String,
    /// The matched symbol, if the hit was a symbol (else the path matched).
    pub symbol: Option<String>,
}

impl ArchMap {
    /// Total symbols across all files.
    pub fn symbol_count(&self) -> usize {
        self.files.iter().map(|f| f.symbols.len()).sum()
    }
}

/// Build a map from `(path, content)` pairs. Pure — the testable core of `scan`.
pub fn build(files: &[(String, String)]) -> ArchMap {
    let mut entries: Vec<FileEntry> = files.iter().map(|(p, c)| extract(p, c)).collect();
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    ArchMap { files: entries }
}

/// Scan `root` recursively, extracting structure from known source files
/// (ignoring `.git`/`target`/`.talicode`/`node_modules` and oversized files).
pub fn scan(root: &Path) -> ArchMap {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() || is_ignored(path) || lang_of(path).is_none() {
            continue;
        }
        if entry
            .metadata()
            .map(|m| m.len() > MAX_FILE_BYTES)
            .unwrap_or(true)
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel = path.strip_prefix(root).unwrap_or(path);
        pairs.push((rel.to_string_lossy().into_owned(), content));
    }
    build(&pairs)
}

/// Persist the map to `<root>/.talicode/architecture.json`.
pub fn save(root: &Path, map: &ArchMap) -> std::io::Result<()> {
    let dir = root.join(ARCH_DIR);
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(map).map_err(std::io::Error::other)?;
    std::fs::write(dir.join(ARCH_FILE), json)
}

/// Load the map, or `None` if it is absent/unreadable.
pub fn load(root: &Path) -> Option<ArchMap> {
    let text = std::fs::read_to_string(root.join(ARCH_DIR).join(ARCH_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Look up `query` in the map: matching symbols first, then files whose path
/// matches. Case-insensitive substring. Pure.
pub fn arch_lookup(map: &ArchMap, query: &str) -> Vec<Location> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in &map.files {
        for s in &f.symbols {
            if s.to_ascii_lowercase().contains(&q) {
                out.push(Location {
                    path: f.path.clone(),
                    symbol: Some(s.clone()),
                });
            }
        }
        if f.path.to_ascii_lowercase().contains(&q) {
            out.push(Location {
                path: f.path.clone(),
                symbol: None,
            });
        }
    }
    out
}

/// A compact overview injected into working context: file count, symbol count,
/// and per-file symbol tallies. `""` when the map is empty.
pub fn overview(map: &ArchMap) -> String {
    if map.files.is_empty() {
        return String::new();
    }
    let mut out = format!(
        "Codebase map: {} files, {} symbols.\n",
        map.files.len(),
        map.symbol_count()
    );
    for f in &map.files {
        out.push_str(&format!("- {} ({} symbols)\n", f.path, f.symbols.len()));
    }
    out
}

/// Known source languages by extension.
fn lang_of(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some("rust"),
        Some("py") => Some("python"),
        Some("ts" | "tsx" | "js" | "jsx") => Some("ts"),
        Some("go") => Some("go"),
        _ => None,
    }
}

fn extract(path: &str, content: &str) -> FileEntry {
    let lang = lang_of(Path::new(path)).unwrap_or("rust");
    let (sym_kw, imp_kw) = keywords(lang);
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    for raw in content.lines() {
        let line = normalize(raw.trim());
        if let Some(name) = decl_name(line, sym_kw) {
            if !symbols.contains(&name) {
                symbols.push(name);
            }
        }
        if let Some(imp) = decl_rest(line, imp_kw) {
            if !imports.contains(&imp) {
                imports.push(imp);
            }
        }
    }
    FileEntry {
        path: path.to_string(),
        symbols,
        imports,
    }
}

/// Strip common leading modifiers so declaration keywords match.
fn normalize(line: &str) -> &str {
    for prefix in [
        "pub(crate) ",
        "pub ",
        "export default ",
        "export ",
        "async ",
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return normalize(rest);
        }
    }
    line
}

fn keywords(lang: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match lang {
        "python" => (&["def ", "class "], &["import ", "from "]),
        "ts" => (
            &["function ", "class ", "const ", "interface ", "type "],
            &["import "],
        ),
        "go" => (&["func ", "type "], &["import "]),
        // rust and default
        _ => (
            &[
                "fn ", "struct ", "enum ", "trait ", "const ", "type ", "mod ",
            ],
            &["use "],
        ),
    }
}

/// If `line` starts with one of `keywords`, return the identifier that follows.
fn decl_name(line: &str, keywords: &[&str]) -> Option<String> {
    for kw in keywords {
        if let Some(rest) = line.strip_prefix(kw) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

/// If `line` starts with one of `keywords`, return the rest (trimmed of `;`).
fn decl_rest(line: &str, keywords: &[&str]) -> Option<String> {
    for kw in keywords {
        if let Some(rest) = line.strip_prefix(kw) {
            let val = rest.trim_end_matches(';').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUST: &str = "use std::path::Path;\npub fn sweep() {}\nstruct Auditor;\n";
    const PY: &str =
        "import os\nfrom sys import argv\ndef run():\n    pass\nclass Engine:\n    pass\n";

    #[test]
    fn build_extracts_symbols_and_imports() {
        let map = build(&[
            ("src/lib.rs".into(), RUST.into()),
            ("app/run.py".into(), PY.into()),
        ]);
        let rust = map.files.iter().find(|f| f.path == "src/lib.rs").unwrap();
        assert!(rust.symbols.contains(&"sweep".to_string()));
        assert!(rust.symbols.contains(&"Auditor".to_string()));
        assert!(rust.imports.contains(&"std::path::Path".to_string()));

        let py = map.files.iter().find(|f| f.path == "app/run.py").unwrap();
        assert!(py.symbols.contains(&"run".to_string()));
        assert!(py.symbols.contains(&"Engine".to_string()));
    }

    #[test]
    fn arch_lookup_finds_symbols_and_paths() {
        let map = build(&[("src/lib.rs".into(), RUST.into())]);
        let hits = arch_lookup(&map, "sweep");
        assert_eq!(hits[0].symbol.as_deref(), Some("sweep"));
        let by_path = arch_lookup(&map, "lib.rs");
        assert!(by_path
            .iter()
            .any(|h| h.symbol.is_none() && h.path == "src/lib.rs"));
        assert!(arch_lookup(&map, "").is_empty());
    }

    #[test]
    fn overview_reports_counts_or_blank() {
        assert_eq!(overview(&ArchMap::default()), "");
        let map = build(&[("src/lib.rs".into(), RUST.into())]);
        let ov = overview(&map);
        assert!(ov.contains("1 files"));
        assert!(ov.contains("src/lib.rs"));
    }

    #[test]
    fn scan_save_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), RUST).unwrap();
        // Ignored dirs are skipped.
        std::fs::create_dir_all(dir.path().join("target")).unwrap();
        std::fs::write(dir.path().join("target/x.rs"), "fn hidden() {}").unwrap();

        let map = scan(dir.path());
        assert_eq!(map.files.len(), 1, "target/ is ignored");
        assert_eq!(map.files[0].path, "src/main.rs");

        save(dir.path(), &map).unwrap();
        assert_eq!(load(dir.path()).unwrap(), map);
    }
}
