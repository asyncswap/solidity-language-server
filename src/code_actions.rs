/// JSON-driven code-action database.
///
/// The database is compiled into the binary via `include_str!` so there is no
/// runtime I/O and no additional install step for users.
///
/// # Schema
///
/// Each entry in `data/error_codes.json` may carry an `"action"` object.
/// `null` means no quick-fix is available.  A non-null object has the shape:
///
/// ```json
/// {
///   "kind":  "insert" | "replace_token" | "delete_token" |
///            "delete_node" | "insert_before_node" | "custom",
///   "title": "<human-readable label shown in the editor>",
///
///   // insert only
///   "text":   "<text to insert>",
///   "anchor": "file_start",          // only value for now
///
///   // replace_token only
///   "replacement": "<new text>",
///
///   // delete_node only
///   "node": "<tree-sitter node kind to delete>",
///
///   // insert_before_node only
///   "walk_to":      "<tree-sitter node kind to walk up to>",
///   "before_child": ["<first matching child kind>", ...],
///   "text":         "<text to insert>"
/// }
/// ```
///
/// `"custom"` entries have no extra fields — the handler falls through to the
/// hand-written match arms in `lsp.rs`.
use std::collections::HashMap;

use crate::types::ErrorCode;
use serde::Deserialize;

// ── JSON types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawEntry {
    code: ErrorCode,
    action: Option<RawAction>,
}

#[derive(Debug, Deserialize)]
struct RawAction {
    kind: String,
    title: Option<String>,
    // insert / insert_before_node
    text: Option<String>,
    anchor: Option<String>,
    // replace_token
    replacement: Option<String>,
    // delete_node
    node: Option<String>,
    // insert_before_node
    walk_to: Option<String>,
    before_child: Option<Vec<String>>,
}

// ── Public types ─────────────────────────────────────────────────────────────

/// A fully-typed quick-fix action loaded from the database.
#[derive(Debug, Clone)]
pub struct CodeActionDef {
    pub title: String,
    pub fix: FixKind,
}

#[derive(Debug, Clone)]
pub enum FixKind {
    /// Insert fixed text at a well-known anchor.
    Insert { text: String, anchor: InsertAnchor },

    /// Replace the token whose byte range starts at `diag_range.start`.
    /// When `walk_to` is `Some`, walk up the TS tree to the first ancestor of
    /// that kind and replace that whole node instead of just the leaf token.
    ReplaceToken {
        replacement: String,
        walk_to: Option<String>,
    },

    /// Delete the token at `diag_range.start` (+ one trailing space if present).
    DeleteToken,

    /// Walk the TS tree up to `node_kind`, then delete the whole node
    /// (including leading whitespace/newline so the line disappears cleanly).
    DeleteNode { node_kind: String },

    /// Walk the TS tree up to `walk_to`, then delete the first child whose
    /// kind matches any entry in `child_kinds` (tried in order).
    /// Used when the diagnostic points to the parent node (e.g. 4126: diag
    /// starts at `function` keyword but we need to delete the `visibility` child).
    DeleteChildNode {
        walk_to: String,
        child_kinds: Vec<String>,
    },

    /// Walk the TS tree up to `walk_to`, then replace the first child whose
    /// kind matches `child_kind` with `replacement`.
    /// Used for 1560/1159/4095: replace wrong visibility with `external`.
    ReplaceChildNode {
        walk_to: String,
        child_kind: String,
        replacement: String,
    },

    /// Walk the TS tree up to `walk_to`, then insert `text` immediately before
    /// the first child whose kind matches any entry in `before_child`.
    InsertBeforeNode {
        walk_to: String,
        before_child: Vec<String>,
        text: String,
    },

    /// No generic fix available — the handler falls through to a hand-written
    /// match arm in `lsp.rs`.
    Custom,
}

#[derive(Debug, Clone)]
pub enum InsertAnchor {
    FileStart,
}

// ── Database ─────────────────────────────────────────────────────────────────

static ERROR_CODES_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/error_codes.json"
));

/// Parse the embedded JSON once and return a map from error code → action.
/// Entries whose `action` is `null` are omitted from the map.
/// Call this once at server startup and store the result.
pub fn load() -> HashMap<ErrorCode, CodeActionDef> {
    let raw: Vec<RawEntry> =
        serde_json::from_str(ERROR_CODES_JSON).expect("data/error_codes.json is malformed");

    let mut map = HashMap::new();
    for entry in raw {
        let Some(action) = entry.action else { continue };
        let Some(def) = parse_action(action) else {
            continue;
        };
        map.insert(entry.code, def);
    }
    map
}

fn parse_action(a: RawAction) -> Option<CodeActionDef> {
    let title = a.title.unwrap_or_default();
    let fix = match a.kind.as_str() {
        "insert" => FixKind::Insert {
            text: a.text?,
            anchor: match a.anchor.as_deref() {
                Some("file_start") | None => InsertAnchor::FileStart,
                other => {
                    eprintln!("unknown insert anchor: {other:?}");
                    return None;
                }
            },
        },

        "replace_token" => FixKind::ReplaceToken {
            replacement: a.replacement?,
            walk_to: a.walk_to,
        },

        "delete_token" => FixKind::DeleteToken,

        "delete_node" => FixKind::DeleteNode { node_kind: a.node? },

        "delete_child_node" => {
            // `before_child` is an ordered list of candidate kinds (first match wins).
            // `node` is a single-kind shorthand; if both present, before_child wins.
            let child_kinds = a.before_child.or_else(|| a.node.map(|n| vec![n]))?;
            FixKind::DeleteChildNode {
                walk_to: a.walk_to?,
                child_kinds,
            }
        }

        "replace_child_node" => FixKind::ReplaceChildNode {
            walk_to: a.walk_to?,
            child_kind: a.node?,
            replacement: a.replacement?,
        },

        "insert_before_node" => FixKind::InsertBeforeNode {
            walk_to: a.walk_to?,
            before_child: a.before_child?,
            text: a.text?,
        },

        "custom" => FixKind::Custom,

        other => {
            eprintln!("unknown action kind: {other:?}");
            return None;
        }
    };

    Some(CodeActionDef { title, fix })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_parses_without_panic() {
        let db = load();
        assert!(!db.is_empty(), "database should have at least one action");
    }

    #[test]
    fn test_known_codes_present() {
        let db = load();
        // Every code we explicitly handle should be in the map.
        let expected = [
            1878u32, 2072, 2074, 7591, 1827, 9102, 9125, 2662, 6879, 9348, 5424, 7359, 3557, 4538,
            8050, 1400, 2256, 8113, // constructor visibility
            2462, 9239, 8295, 1845, // payable
            9559, 7708, 5587, // interface/fallback/receive must be external
            1560, 1159, 4095, 7341, // modifier virtual
            8063, // free function visibility (fixed kind)
            4126,
        ];
        for code in expected {
            assert!(db.contains_key(&code), "missing code {code}");
        }
    }

    #[test]
    fn test_1878_is_insert() {
        let db = load();
        let def = db.get(&1878).unwrap();
        assert!(def.title.contains("SPDX"));
        assert!(matches!(def.fix, FixKind::Insert { .. }));
        if let FixKind::Insert { text, anchor } = &def.fix {
            assert!(text.contains("SPDX-License-Identifier"));
            assert!(matches!(anchor, InsertAnchor::FileStart));
        }
    }

    #[test]
    fn test_7359_is_replace_token() {
        let db = load();
        let def = db.get(&7359).unwrap();
        if let FixKind::ReplaceToken { replacement, .. } = &def.fix {
            assert_eq!(replacement, "block.timestamp");
        } else {
            panic!("expected ReplaceToken for 7359");
        }
    }

    #[test]
    fn test_2072_is_delete_node() {
        let db = load();
        let def = db.get(&2072).unwrap();
        if let FixKind::DeleteNode { node_kind } = &def.fix {
            assert_eq!(node_kind, "variable_declaration_statement");
        } else {
            panic!("expected DeleteNode for 2072");
        }
    }

    #[test]
    fn test_5424_is_insert_before_node() {
        let db = load();
        let def = db.get(&5424).unwrap();
        if let FixKind::InsertBeforeNode {
            walk_to,
            before_child,
            text,
        } = &def.fix
        {
            assert_eq!(walk_to, "function_definition");
            assert!(before_child.contains(&"return_type_definition".to_string()));
            assert!(before_child.contains(&";".to_string()));
            assert_eq!(text, "virtual ");
        } else {
            panic!("expected InsertBeforeNode for 5424");
        }
    }

    #[test]
    fn test_custom_codes_are_custom() {
        let db = load();
        for code in [2018u32, 9456] {
            let def = db.get(&code).unwrap();
            assert!(
                matches!(def.fix, FixKind::Custom),
                "code {code} should be Custom"
            );
        }
    }

    #[test]
    fn test_null_actions_not_in_map() {
        let db = load();
        // 1005 has no action in the JSON
        assert!(!db.contains_key(&1005));
    }
}
