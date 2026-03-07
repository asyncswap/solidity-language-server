use solidity_language_server::file_operations;
use std::fs;
use tower_lsp::lsp_types::Url;

// =============================================================================
// Live test with ForgeRunner (example/ directory)
// =============================================================================

#[tokio::test]
async fn test_rename_a_to_aa_produces_edit_on_b() {
    let example_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("example");
    let a_path = example_dir.join("A.sol");
    let b_path = example_dir.join("B.sol");
    assert!(a_path.exists(), "example/A.sol must exist");
    assert!(b_path.exists(), "example/B.sol must exist");

    let old_uri = Url::from_file_path(&a_path).unwrap();
    let new_path = example_dir.join("AA.sol");
    let new_uri = Url::from_file_path(&new_path).unwrap();

    // Discover source files — all .sol files in example/
    let source_files: Vec<String> = fs::read_dir(&example_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sol"))
        .filter_map(|e| e.path().to_str().map(String::from))
        .collect();

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let mut edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        &example_dir,
        &provider,
    );

    // Some solc absolutePath shapes are workspace-relative (e.g. "example/A.sol"),
    // which can miss importer files when projected under project_root.
    // Fall back to directory discovery for this assertion path.
    let b_uri = Url::from_file_path(&b_path).unwrap();
    if !edits.contains_key(&b_uri) {
        let fallback_files: Vec<String> = fs::read_dir(&example_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sol"))
            .filter_map(|e| e.path().to_str().map(String::from))
            .collect();
        edits = file_operations::rename_imports_single(
            &fallback_files,
            &old_uri,
            &new_uri,
            &example_dir,
            &provider,
        );
    }

    assert!(
        edits.contains_key(&b_uri),
        "edits should contain B.sol — it imports A.sol"
    );

    let b_edits = &edits[&b_uri];
    assert_eq!(b_edits.len(), 1, "B.sol should have exactly 1 import edit");

    let edit = &b_edits[0];
    assert_eq!(
        edit.new_text, "\"./AA.sol\"",
        "newText should be quoted ./AA.sol"
    );
}

/// Test using solc_project_index to discover source files.
#[tokio::test]
async fn test_rename_a_to_aa_via_solc_project_index() {
    use solidity_language_server::config;
    use solidity_language_server::goto::CachedBuild;

    let example_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("example");
    let a_path = example_dir.join("A.sol");
    let b_path = example_dir.join("B.sol");
    assert!(a_path.exists(), "example/A.sol must exist");
    assert!(b_path.exists(), "example/B.sol must exist");

    let foundry_cfg = config::load_foundry_config(&example_dir);

    // Use solc_project_index to get the build, then extract source files.
    let ast_data = solidity_language_server::solc::solc_project_index(&foundry_cfg, None, None)
        .await
        .expect("solc_project_index should succeed for example project");

    let _build = CachedBuild::new(ast_data, 0, None);
    let source_files: Vec<String> = fs::read_dir(&example_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sol"))
        .filter_map(|e| e.path().to_str().map(String::from))
        .collect();

    let old_uri = Url::from_file_path(&a_path).unwrap();
    let new_path = example_dir.join("AA.sol");
    let new_uri = Url::from_file_path(&new_path).unwrap();

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        &example_dir,
        &provider,
    );

    let b_uri = Url::from_file_path(&b_path).unwrap();
    assert!(
        edits.contains_key(&b_uri),
        "edits should contain B.sol — it imports A.sol"
    );

    let b_edits = &edits[&b_uri];
    assert_eq!(b_edits.len(), 1);
    assert_eq!(b_edits[0].new_text, "\"./AA.sol\"");
}

// =============================================================================
// Tests with synthetic .sol sources (tree-sitter parseable)
// =============================================================================

/// Create a temporary directory with .sol files for testing.
struct TempProject {
    dir: tempfile::TempDir,
}

impl TempProject {
    fn new() -> Self {
        Self {
            dir: tempfile::TempDir::new().unwrap(),
        }
    }

    fn write_file(&self, name: &str, content: &str) -> String {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path.to_str().unwrap().to_string()
    }

    fn path(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join(name)
    }

    fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.path(name)).unwrap()
    }

    fn root(&self) -> &std::path::Path {
        self.dir.path()
    }
}

#[test]
fn test_rename_simple_import() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {A} from \"./A.sol\";\ncontract B is A {}\n",
    );

    let source_files = vec![a_path, b_path];

    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("AA.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    let b_uri = proj.uri("B.sol");
    assert!(edits.contains_key(&b_uri), "B.sol should have edits");
    assert_eq!(edits[&b_uri].len(), 1);
    assert_eq!(edits[&b_uri][0].new_text, "\"./AA.sol\"");
}

#[test]
fn test_rename_nobody_imports_returns_empty() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract B {}\n",
    );

    let source_files = vec![a_path, b_path];

    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("AA.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    assert!(edits.is_empty(), "no edits when nobody imports the file");
}

#[test]
fn test_rename_nonexistent_file_returns_empty() {
    let old_uri = Url::from_file_path("/tmp/nonexistent/Foo.sol").unwrap();
    let new_uri = Url::from_file_path("/tmp/nonexistent/Bar.sol").unwrap();

    let source_files: Vec<String> = vec![];
    let provider = |_: &str| -> Option<Vec<u8>> { None };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        std::path::Path::new("/tmp/nonexistent"),
        &provider,
    );

    assert!(edits.is_empty());
}

#[test]
fn test_rename_multiple_importers() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {A} from \"./A.sol\";\ncontract B is A {}\n",
    );
    let c_path = proj.write_file(
        "C.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport \"./A.sol\";\ncontract C {}\n",
    );

    let source_files = vec![a_path, b_path, c_path];

    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("AA.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    assert_eq!(edits.len(), 2, "both B.sol and C.sol should have edits");

    for (_uri, file_edits) in &edits {
        for te in file_edits {
            assert_eq!(te.new_text, "\"./AA.sol\"");
        }
    }
}

#[test]
fn test_rename_does_not_affect_unrelated_imports() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract B {}\n",
    );
    let c_path = proj.write_file(
        "C.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {B} from \"./B.sol\";\ncontract C is B {}\n",
    );

    let source_files = vec![a_path, b_path, c_path];

    // Rename A.sol — C.sol imports B.sol, not A.sol, so no edits.
    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("AA.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    assert!(edits.is_empty(), "C.sol imports B.sol not A.sol");
}

#[test]
fn test_move_file_updates_own_imports() {
    let proj = TempProject::new();

    let lib_path = proj.write_file(
        "lib/Math.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nlibrary Math {}\n",
    );
    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {Math} from \"./lib/Math.sol\";\ncontract A {}\n",
    );

    let source_files = vec![lib_path, a_path];

    // Move A.sol into sub/ directory.
    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("sub/A.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    assert!(
        edits.contains_key(&old_uri),
        "A.sol should have edits (own imports updated after move)"
    );

    let a_edits = &edits[&old_uri];
    assert_eq!(a_edits.len(), 1);
    // From sub/A.sol, lib/Math.sol is at ../lib/Math.sol
    assert_eq!(a_edits[0].new_text, "\"../lib/Math.sol\"");
}

#[test]
fn test_same_dir_rename_does_not_rewrite_own_imports() {
    let proj = TempProject::new();

    let lib_path = proj.write_file(
        "lib/Math.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nlibrary Math {}\n",
    );
    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {Math} from \"./lib/Math.sol\";\ncontract A {}\n",
    );

    let source_files = vec![lib_path, a_path];

    // Rename in same dir — own imports shouldn't change.
    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("AA.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    assert!(
        !edits.contains_key(&old_uri),
        "same-dir rename should not edit the file's own imports"
    );
}

#[test]
fn test_rename_cross_directory_import() {
    let proj = TempProject::new();

    let src_path = proj.write_file(
        "src/Token.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract Token {}\n",
    );
    let test_path = proj.write_file(
        "test/Token.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {Token} from \"../src/Token.sol\";\ncontract TokenTest {}\n",
    );

    let source_files = vec![src_path, test_path];

    let old_uri = proj.uri("src/Token.sol");
    let new_uri = proj.uri("src/TokenV2.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    let test_uri = proj.uri("test/Token.t.sol");
    assert!(edits.contains_key(&test_uri));
    assert_eq!(edits[&test_uri][0].new_text, "\"../src/TokenV2.sol\"");
}

#[test]
fn test_skips_non_relative_imports() {
    let proj = TempProject::new();

    // A file with a remapped import (not starting with ./ or ../)
    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport \"forge-std/Test.sol\";\nimport {B} from \"./B.sol\";\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract B {}\n",
    );

    let source_files = vec![a_path, b_path];

    // Move A.sol to sub/ — only the relative import ./B.sol should be updated,
    // NOT the remapped import forge-std/Test.sol
    let old_uri = proj.uri("A.sol");
    let new_uri = proj.uri("sub/A.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    if let Some(a_edits) = edits.get(&old_uri) {
        // Should only have 1 edit (for ./B.sol), not for forge-std/Test.sol
        assert_eq!(
            a_edits.len(),
            1,
            "only relative imports should be rewritten"
        );
        assert!(a_edits[0].new_text.contains("B.sol"));
    }
}

#[test]
fn test_rename_non_relative_import_resolved_against_project_root() {
    // Simulates Foundry-style imports: `import "src/interfaces/IPoolManager.sol";`
    // These are resolved against the project root, not the importing file's directory.
    let proj = TempProject::new();

    let pm_path = proj.write_file(
        "src/PoolManager.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract PoolManager {}\n",
    );
    let test_path = proj.write_file(
        "test/PoolManager.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {PoolManager} from \"src/PoolManager.sol\";\ncontract PoolManagerTest {}\n",
    );

    let source_files = vec![pm_path, test_path];

    let old_uri = proj.uri("src/PoolManager.sol");
    let new_uri = proj.uri("src/PoolManagerV2.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    let test_uri = proj.uri("test/PoolManager.t.sol");
    assert!(
        edits.contains_key(&test_uri),
        "test file with non-relative import should have edits"
    );
    assert_eq!(edits[&test_uri].len(), 1);
    // The replacement should preserve non-relative style (project-root-relative).
    assert_eq!(
        edits[&test_uri][0].new_text, "\"src/PoolManagerV2.sol\"",
        "non-relative import should be updated relative to project root"
    );
}

#[test]
fn test_rename_non_relative_import_does_not_match_library_imports() {
    // Remapped imports like "forge-std/Test.sol" should NOT match project files,
    // even if a file at <project_root>/forge-std/Test.sol happened to exist.
    let proj = TempProject::new();

    let pm_path = proj.write_file(
        "src/PoolManager.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract PoolManager {}\n",
    );
    // This test file imports PoolManager via non-relative path and also
    // has a forge-std import that should NOT be affected.
    let test_path = proj.write_file(
        "test/PM.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport \"forge-std/Test.sol\";\nimport {PoolManager} from \"src/PoolManager.sol\";\ncontract PMTest {}\n",
    );

    let source_files = vec![pm_path, test_path];

    let old_uri = proj.uri("src/PoolManager.sol");
    let new_uri = proj.uri("src/PoolManagerV2.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    let test_uri = proj.uri("test/PM.t.sol");
    assert!(edits.contains_key(&test_uri));
    // Only 1 edit: the src/PoolManager.sol import, NOT forge-std/Test.sol
    assert_eq!(
        edits[&test_uri].len(),
        1,
        "only the matching non-relative import should be edited"
    );
    assert_eq!(edits[&test_uri][0].new_text, "\"src/PoolManagerV2.sol\"");
}

#[test]
fn test_rename_mixed_relative_and_non_relative_importers() {
    // One file uses relative import, another uses non-relative. Both should be updated.
    let proj = TempProject::new();

    let pm_path = proj.write_file(
        "src/PoolManager.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract PoolManager {}\n",
    );
    // Relative import from same directory
    let helper_path = proj.write_file(
        "src/Helper.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {PoolManager} from \"./PoolManager.sol\";\ncontract Helper {}\n",
    );
    // Non-relative import from test directory
    let test_path = proj.write_file(
        "test/PM.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {PoolManager} from \"src/PoolManager.sol\";\ncontract PMTest {}\n",
    );

    let source_files = vec![pm_path, helper_path, test_path];

    let old_uri = proj.uri("src/PoolManager.sol");
    let new_uri = proj.uri("src/PoolManagerV2.sol");

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits = file_operations::rename_imports_single(
        &source_files,
        &old_uri,
        &new_uri,
        proj.root(),
        &provider,
    );

    // Both files should have edits
    let helper_uri = proj.uri("src/Helper.sol");
    let test_uri = proj.uri("test/PM.t.sol");

    assert!(
        edits.contains_key(&helper_uri),
        "relative importer should have edits"
    );
    assert!(
        edits.contains_key(&test_uri),
        "non-relative importer should have edits"
    );

    // Relative importer keeps relative style
    assert_eq!(edits[&helper_uri][0].new_text, "\"./PoolManagerV2.sol\"");
    // Non-relative importer keeps non-relative style
    assert_eq!(edits[&test_uri][0].new_text, "\"src/PoolManagerV2.sol\"");
}

// =============================================================================
// Batch rename tests (multi-file / folder rename)
// =============================================================================

#[test]
fn test_batch_rename_folder_move_preserves_sibling_imports() {
    // When a folder is renamed, files inside it that import each other
    // should NOT have their imports rewritten (relative paths stay the same).
    let proj = TempProject::new();

    proj.write_file(
        "src/A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {B} from \"./B.sol\";\ncontract A is B {}\n",
    );
    proj.write_file(
        "src/B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract B {}\n",
    );

    let source_files = vec![
        proj.path("src/A.sol").to_str().unwrap().to_string(),
        proj.path("src/B.sol").to_str().unwrap().to_string(),
    ];

    // Both files move from src/ to contracts/ (folder rename).
    let renames = vec![
        file_operations::FileRename {
            old_path: proj.path("src/A.sol"),
            new_path: proj.path("contracts/A.sol"),
        },
        file_operations::FileRename {
            old_path: proj.path("src/B.sol"),
            new_path: proj.path("contracts/B.sol"),
        },
    ];

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits =
        file_operations::rename_imports(&source_files, &renames, proj.root(), &provider).edits;

    // A.sol imports ./B.sol — both moved together, relative path unchanged.
    assert!(
        edits.is_empty(),
        "sibling imports should not change during folder rename, got: {:?}",
        edits
    );
}

#[test]
fn test_batch_rename_folder_move_updates_external_importers() {
    // Files outside the moved folder that import files inside it must be updated.
    let proj = TempProject::new();

    proj.write_file(
        "src/Token.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract Token {}\n",
    );
    proj.write_file(
        "test/Token.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {Token} from \"../src/Token.sol\";\ncontract TokenTest {}\n",
    );

    let source_files = vec![
        proj.path("src/Token.sol").to_str().unwrap().to_string(),
        proj.path("test/Token.t.sol").to_str().unwrap().to_string(),
    ];

    // src/Token.sol moves to contracts/Token.sol. test/Token.t.sol stays.
    let renames = vec![file_operations::FileRename {
        old_path: proj.path("src/Token.sol"),
        new_path: proj.path("contracts/Token.sol"),
    }];

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits =
        file_operations::rename_imports(&source_files, &renames, proj.root(), &provider).edits;

    let test_uri = proj.uri("test/Token.t.sol");
    assert!(
        edits.contains_key(&test_uri),
        "external importer should have edits"
    );
    assert_eq!(
        edits[&test_uri][0].new_text, "\"../contracts/Token.sol\"",
        "import path should point to new location"
    );
}

#[test]
fn test_batch_rename_importer_also_moved() {
    // Both the imported file and the importer are moved. The importer's
    // import path should be computed from its NEW directory.
    let proj = TempProject::new();

    proj.write_file(
        "src/Token.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract Token {}\n",
    );
    proj.write_file(
        "src/Helper.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {Token} from \"./Token.sol\";\ncontract Helper {}\n",
    );

    let source_files = vec![
        proj.path("src/Token.sol").to_str().unwrap().to_string(),
        proj.path("src/Helper.sol").to_str().unwrap().to_string(),
    ];

    // Token.sol moves to lib/Token.sol, Helper.sol moves to contracts/Helper.sol.
    let renames = vec![
        file_operations::FileRename {
            old_path: proj.path("src/Token.sol"),
            new_path: proj.path("lib/Token.sol"),
        },
        file_operations::FileRename {
            old_path: proj.path("src/Helper.sol"),
            new_path: proj.path("contracts/Helper.sol"),
        },
    ];

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits =
        file_operations::rename_imports(&source_files, &renames, proj.root(), &provider).edits;

    // Helper.sol (keyed by old URI) should have an edit.
    // From contracts/Helper.sol, lib/Token.sol is ../lib/Token.sol.
    let helper_uri = proj.uri("src/Helper.sol");
    assert!(
        edits.contains_key(&helper_uri),
        "moved importer should have edits"
    );
    assert_eq!(
        edits[&helper_uri][0].new_text, "\"../lib/Token.sol\"",
        "import path should be computed from importer's new location"
    );
}

#[test]
fn test_batch_rename_non_relative_import_folder_move() {
    // Non-relative import (Foundry-style) when the target file is moved.
    let proj = TempProject::new();

    proj.write_file(
        "src/PoolManager.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract PoolManager {}\n",
    );
    proj.write_file(
        "test/PM.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {PoolManager} from \"src/PoolManager.sol\";\ncontract PMTest {}\n",
    );

    let source_files = vec![
        proj.path("src/PoolManager.sol")
            .to_str()
            .unwrap()
            .to_string(),
        proj.path("test/PM.t.sol").to_str().unwrap().to_string(),
    ];

    // Move src/PoolManager.sol to contracts/PoolManager.sol.
    let renames = vec![file_operations::FileRename {
        old_path: proj.path("src/PoolManager.sol"),
        new_path: proj.path("contracts/PoolManager.sol"),
    }];

    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };
    let edits =
        file_operations::rename_imports(&source_files, &renames, proj.root(), &provider).edits;

    let test_uri = proj.uri("test/PM.t.sol");
    assert!(edits.contains_key(&test_uri));
    // Non-relative import should update to new project-root-relative path.
    assert_eq!(
        edits[&test_uri][0].new_text, "\"contracts/PoolManager.sol\"",
        "non-relative import should reflect new location relative to project root"
    );
}

// =============================================================================
// normalize_path edge cases
// =============================================================================

#[test]
fn test_normalize_path_basic() {
    use std::path::Path;
    let p = file_operations::normalize_path(Path::new("/a/b/../c/./d"));
    assert_eq!(p, Path::new("/a/c/d"));
}

#[test]
fn test_normalize_path_excessive_parent() {
    use std::path::Path;
    // Excessive .. should not pop past the root.
    let p = file_operations::normalize_path(Path::new("/a/../../b"));
    assert_eq!(p, Path::new("/b"), "should not pop past root: got {:?}", p);
}

#[test]
fn test_normalize_path_relative() {
    use std::path::Path;
    let p = file_operations::normalize_path(Path::new("a/b/../c"));
    assert_eq!(p, Path::new("a/c"));
}

// =============================================================================
// apply_text_edits tests
// =============================================================================

#[test]
fn test_apply_text_edits_basic() {
    use tower_lsp::lsp_types::{Position, Range, TextEdit};

    let source = "import \"./old.sol\";\ncontract A {}\n";
    let edits = vec![TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 7,
            },
            end: Position {
                line: 0,
                character: 18,
            },
        },
        new_text: "\"./new.sol\"".to_string(),
    }];

    let result = file_operations::apply_text_edits(source, &edits);
    assert_eq!(result, "import \"./new.sol\";\ncontract A {}\n");
}

#[test]
fn test_apply_text_edits_skips_overlapping_edits() {
    use tower_lsp::lsp_types::{Position, Range, TextEdit};

    let source = "import \"./old.sol\";\ncontract A {}\n";
    let edits = vec![
        TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 7,
                },
                end: Position {
                    line: 0,
                    character: 18,
                },
            },
            new_text: "\"./new.sol\"".to_string(),
        },
        // Overlaps the first edit and should be ignored.
        TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 10,
                },
                end: Position {
                    line: 0,
                    character: 15,
                },
            },
            new_text: "\"BROKEN\"".to_string(),
        },
    ];

    let result = file_operations::apply_text_edits(source, &edits);
    assert_eq!(result, "import \"./new.sol\";\ncontract A {}\n");
}

#[test]
fn test_expand_folder_renames_from_paths_uses_component_prefix() {
    let old_uri = Url::from_file_path("/tmp/project/src").unwrap();
    let new_uri = Url::from_file_path("/tmp/project/contracts").unwrap();
    let params = vec![(old_uri, new_uri)];
    let candidates = vec![
        std::path::PathBuf::from("/tmp/project/src/A.sol"),
        std::path::PathBuf::from("/tmp/project/src2/B.sol"),
    ];

    let expanded = file_operations::expand_folder_renames_from_paths(&params, &candidates);
    assert_eq!(expanded.len(), 1, "should not match sibling src2 path");
    assert!(expanded[0].0.ends_with("/tmp/project/src/A.sol"));
    assert!(expanded[0].1.ends_with("/tmp/project/contracts/A.sol"));
}

// =============================================================================
// delete_imports tests
// =============================================================================

#[test]
fn test_delete_file_removes_relative_import_statement() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {A} from \"./A.sol\";\ncontract B is A {}\n",
    );

    let source_files = vec![a_path, b_path.clone()];
    let deletes = vec![proj.path("A.sol")];
    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let result = file_operations::delete_imports(&source_files, &deletes, proj.root(), &provider);
    let b_uri = proj.uri("B.sol");
    assert!(result.edits.contains_key(&b_uri));
    assert_eq!(result.edits[&b_uri].len(), 1);

    let b_source = std::fs::read_to_string(proj.path("B.sol")).unwrap();
    let patched = file_operations::apply_text_edits(&b_source, &result.edits[&b_uri]);
    assert!(!patched.contains("import {A} from \"./A.sol\";"));
    assert!(patched.contains("contract B is A {}"));
}

#[test]
fn test_delete_file_removes_non_relative_import_statement() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "src/A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let t_path = proj.write_file(
        "test/T.t.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {A} from \"src/A.sol\";\ncontract T is A {}\n",
    );

    let source_files = vec![a_path, t_path];
    let deletes = vec![proj.path("src/A.sol")];
    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let result = file_operations::delete_imports(&source_files, &deletes, proj.root(), &provider);
    let t_uri = proj.uri("test/T.t.sol");
    assert!(result.edits.contains_key(&t_uri));
    assert_eq!(result.edits[&t_uri].len(), 1);
}

#[test]
fn test_delete_file_removes_multiline_import_statement() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "src/A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let c_path = proj.write_file(
        "src/C.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport {\n    A\n} from \"./A.sol\";\ncontract C is A {}\n",
    );

    let source_files = vec![a_path, c_path.clone()];
    let deletes = vec![proj.path("src/A.sol")];
    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let result = file_operations::delete_imports(&source_files, &deletes, proj.root(), &provider);
    let c_uri = proj.uri("src/C.sol");
    assert!(result.edits.contains_key(&c_uri));

    let c_source = std::fs::read_to_string(proj.path("src/C.sol")).unwrap();
    let patched = file_operations::apply_text_edits(&c_source, &result.edits[&c_uri]);
    assert!(!patched.contains("from \"./A.sol\";"));
    assert!(patched.contains("contract C is A {}"));
}

#[test]
fn test_delete_file_no_matching_imports_returns_empty() {
    let proj = TempProject::new();

    let a_path = proj.write_file(
        "A.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract A {}\n",
    );
    let b_path = proj.write_file(
        "B.sol",
        "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\ncontract B {}\n",
    );

    let source_files = vec![a_path, b_path];
    let deletes = vec![proj.path("Missing.sol")];
    let provider = |fs_path: &str| -> Option<Vec<u8>> { std::fs::read(fs_path).ok() };

    let result = file_operations::delete_imports(&source_files, &deletes, proj.root(), &provider);
    assert!(result.edits.is_empty());
}

#[test]
fn test_expand_folder_deletes_from_paths_uses_component_prefix() {
    let delete_uri = Url::from_file_path("/tmp/project/src").unwrap();
    let params = vec![delete_uri];
    let candidates = vec![
        std::path::PathBuf::from("/tmp/project/src/A.sol"),
        std::path::PathBuf::from("/tmp/project/src2/B.sol"),
    ];

    let expanded = file_operations::expand_folder_deletes_from_paths(&params, &candidates);
    assert_eq!(expanded.len(), 1, "should not match sibling src2 path");
    assert!(expanded[0].ends_with("/tmp/project/src/A.sol"));
}

// =============================================================================
// Scaffold generation tests
// =============================================================================

#[test]
fn test_scaffold_basic_contract() {
    let uri = Url::from_file_path("/tmp/project/src/MyToken.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("pragma solidity ^0.8.0;"));
    assert!(scaffold.contains("contract MyToken {"));
    assert!(scaffold.contains("// SPDX-License-Identifier: MIT"));
}

#[test]
fn test_scaffold_with_solc_version() {
    let uri = Url::from_file_path("/tmp/project/src/Vault.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, Some("0.8.26")).unwrap();
    assert!(scaffold.contains("pragma solidity ^0.8.26;"));
    assert!(scaffold.contains("contract Vault {"));
}

#[test]
fn test_scaffold_with_prefixed_version() {
    let uri = Url::from_file_path("/tmp/project/src/Vault.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, Some(">=0.8.0 <0.9.0")).unwrap();
    assert!(scaffold.contains("pragma solidity >=0.8.0 <0.9.0;"));
}

#[test]
fn test_scaffold_interface() {
    let uri = Url::from_file_path("/tmp/project/src/IPoolManager.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("interface IPoolManager {"));
}

#[test]
fn test_scaffold_library() {
    let uri = Url::from_file_path("/tmp/project/src/LibMath.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("library LibMath {"));
}

#[test]
fn test_scaffold_test_file() {
    // Foo.t.sol → contract FooTest is Test (adds Test suffix)
    let uri = Url::from_file_path("/tmp/project/test/Foo.t.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("import {Test} from \"forge-std/Test.sol\""));
    assert!(scaffold.contains("contract FooTest is Test {"));
}

#[test]
fn test_scaffold_script_file() {
    // Deploy.s.sol → contract DeployScript is Script (adds Script suffix)
    let uri = Url::from_file_path("/tmp/project/script/Deploy.s.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("import {Script} from \"forge-std/Script.sol\""));
    assert!(scaffold.contains("contract DeployScript is Script {"));
}

#[test]
fn test_scaffold_test_file_forces_contract_kind() {
    let uri = Url::from_file_path("/tmp/project/test/IFoo.t.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("contract IFooTest is Test {"));
    assert!(!scaffold.contains("interface IFooTest is Test {"));
}

#[test]
fn test_scaffold_script_file_forces_contract_kind() {
    let uri = Url::from_file_path("/tmp/project/script/LibDeploy.s.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("contract LibDeployScript is Script {"));
    assert!(!scaffold.contains("library LibDeployScript is Script {"));
}

#[test]
fn test_scaffold_non_sol_returns_none() {
    let uri = Url::from_file_path("/tmp/project/README.md").unwrap();
    assert!(file_operations::generate_scaffold(&uri, None).is_none());
}

#[test]
fn test_scaffold_sanitizes_identifier() {
    // Filename with hyphens: "my-token.sol" → contract mytoken
    let uri = Url::from_file_path("/tmp/project/src/my-token.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("contract mytoken {"));
}

#[test]
fn test_scaffold_digit_prefix() {
    // Filename starting with digit: "1inch.sol" → contract _1inch
    let uri = Url::from_file_path("/tmp/project/src/1inch.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("contract _1inch {"));
}

#[test]
fn test_scaffold_keyword_identifier_prefixed() {
    let uri = Url::from_file_path("/tmp/project/src/contract.sol").unwrap();
    let scaffold = file_operations::generate_scaffold(&uri, None).unwrap();
    assert!(scaffold.contains("contract _contract {"));
}
