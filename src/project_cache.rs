use crate::config::FoundryConfig;
use crate::config::ProjectIndexCacheMode;
use crate::goto::{CachedBuild, NodeInfo};
use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tiny_keccak::{Hasher, Keccak};

const CACHE_SCHEMA_VERSION_V2: u32 = 2;
const CACHE_DIR: &str = ".solidity-language-server";
const CACHE_FILE_V2: &str = "solidity-lsp-schema-v2.json";
const CACHE_SHARDS_DIR_V2: &str = "reference-index-v2";
const CACHE_GITIGNORE_FILE: &str = ".gitignore";
const CACHE_GITIGNORE_CONTENTS: &str = "*\n";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedNodeEntry {
    id: u64,
    info: NodeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedExternalRef {
    src: String,
    decl_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedFileShardV2 {
    abs_path: String,
    entries: Vec<PersistedNodeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedReferenceCacheV2 {
    schema_version: u32,
    project_root: String,
    config_fingerprint: String,
    file_hashes: BTreeMap<String, String>,
    #[serde(default)]
    file_hash_history: BTreeMap<String, Vec<String>>,
    path_to_abs: HashMap<String, String>,
    id_to_path_map: HashMap<String, String>,
    external_refs: Vec<PersistedExternalRef>,
    // relative-path -> shard file name
    node_shards: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CacheLoadReport {
    pub build: Option<CachedBuild>,
    pub hit: bool,
    pub miss_reason: Option<String>,
    pub file_count_hashed: usize,
    pub file_count_reused: usize,
    pub complete: bool,
    pub duration_ms: u128,
}

#[derive(Debug, Clone)]
pub struct CacheSaveReport {
    pub file_count_hashed: usize,
    pub duration_ms: u128,
}

/// Public helper — returns the root of the on-disk cache directory for
/// `project_root`. Used by the `solidity.clearCache` command handler.
pub fn cache_dir(root: &Path) -> PathBuf {
    root.join(CACHE_DIR)
}

fn cache_file_path_v2(root: &Path) -> PathBuf {
    root.join(CACHE_DIR).join(CACHE_FILE_V2)
}

fn cache_shards_dir_v2(root: &Path) -> PathBuf {
    root.join(CACHE_DIR).join(CACHE_SHARDS_DIR_V2)
}

fn ensure_cache_dir_layout(root: &Path) -> Result<(PathBuf, PathBuf), String> {
    let cache_root = root.join(CACHE_DIR);
    fs::create_dir_all(&cache_root)
        .map_err(|e| format!("failed to create cache dir {}: {e}", cache_root.display()))?;

    // Ensure cache artifacts are ignored by Git in consumer projects.
    let gitignore_path = cache_root.join(CACHE_GITIGNORE_FILE);
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, CACHE_GITIGNORE_CONTENTS).map_err(|e| {
            format!(
                "failed to write cache gitignore {}: {e}",
                gitignore_path.display()
            )
        })?;
    }

    let shards_dir = cache_shards_dir_v2(root);
    fs::create_dir_all(&shards_dir)
        .map_err(|e| format!("failed to create shards dir {}: {e}", shards_dir.display()))?;

    Ok((cache_root, shards_dir))
}

fn shard_file_name_for_rel_path(rel_path: &str) -> String {
    format!("{}.json", keccak_hex(rel_path.as_bytes()))
}

fn write_atomic_json(path: &Path, payload: &[u8]) -> Result<(), String> {
    let tmp_path = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
    ));
    {
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("create tmp {}: {e}", tmp_path.display()))?;
        file.write_all(payload)
            .map_err(|e| format!("write tmp {}: {e}", tmp_path.display()))?;
        file.flush()
            .map_err(|e| format!("flush tmp {}: {e}", tmp_path.display()))?;
        file.sync_all()
            .map_err(|e| format!("sync tmp {}: {e}", tmp_path.display()))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| {
        format!(
            "rename tmp {} -> {}: {e}",
            tmp_path.display(),
            path.display()
        )
    })
}

fn keccak_hex(bytes: &[u8]) -> String {
    let mut out = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut out);
    hex::encode(out)
}

fn file_hash(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(keccak_hex(&bytes))
}

fn relative_to_root(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn current_file_hashes(
    config: &FoundryConfig,
    include_libs: bool,
) -> Result<BTreeMap<String, String>, String> {
    let source_files = if include_libs {
        crate::solc::discover_source_files_with_libs(config)
    } else {
        crate::solc::discover_source_files(config)
    };
    hash_file_list(config, &source_files)
}

/// Hash an explicit list of absolute paths (relative to config.root).
fn hash_file_list(
    config: &FoundryConfig,
    source_files: &[PathBuf],
) -> Result<BTreeMap<String, String>, String> {
    if source_files.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut hashes = BTreeMap::new();
    for path in source_files {
        let rel = relative_to_root(&config.root, path);
        let hash = file_hash(path)
            .ok_or_else(|| format!("failed to hash source file {}", path.display()))?;
        hashes.insert(rel, hash);
    }
    Ok(hashes)
}

fn config_fingerprint(config: &FoundryConfig) -> String {
    let payload = serde_json::json!({
        // Including the server version means any binary upgrade automatically
        // invalidates the on-disk cache and triggers a clean rebuild.
        "lsp_version": env!("CARGO_PKG_VERSION"),
        "solc_version": config.solc_version,
        "remappings": config.remappings,
        "evm_version": config.evm_version,
        "sources_dir": config.sources_dir,
        "libs": config.libs,
    });
    keccak_hex(payload.to_string().as_bytes())
}

fn push_hash_history(meta: &mut PersistedReferenceCacheV2, rel: &str, hash: &str) {
    const MAX_HISTORY: usize = 8;
    let history = meta.file_hash_history.entry(rel.to_string()).or_default();
    if history.last().is_some_and(|h| h == hash) {
        return;
    }
    history.push(hash.to_string());
    if history.len() > MAX_HISTORY {
        let drop_count = history.len() - MAX_HISTORY;
        history.drain(0..drop_count);
    }
}

pub fn save_reference_cache(config: &FoundryConfig, build: &CachedBuild) -> Result<(), String> {
    save_reference_cache_with_report(config, build, None).map(|_| ())
}

/// Incrementally upsert v2 cache shards from a partial build (typically a
/// saved file compile). This is a fast-path: it updates per-file shards and
/// file hashes for touched files, while preserving existing global metadata.
///
/// The authoritative full-project cache is still produced by full reconcile.
pub fn upsert_reference_cache_v2_with_report(
    config: &FoundryConfig,
    build: &CachedBuild,
) -> Result<CacheSaveReport, String> {
    let started = Instant::now();
    if !config.root.is_dir() {
        return Err(format!("invalid project root: {}", config.root.display()));
    }

    let (_cache_root, shards_dir) = ensure_cache_dir_layout(&config.root)?;

    let meta_path = cache_file_path_v2(&config.root);
    let mut meta = if let Ok(bytes) = fs::read(&meta_path) {
        serde_json::from_slice::<PersistedReferenceCacheV2>(&bytes).unwrap_or(
            PersistedReferenceCacheV2 {
                schema_version: CACHE_SCHEMA_VERSION_V2,
                project_root: config.root.to_string_lossy().to_string(),
                config_fingerprint: config_fingerprint(config),
                file_hashes: BTreeMap::new(),
                file_hash_history: BTreeMap::new(),
                path_to_abs: HashMap::new(),
                id_to_path_map: HashMap::new(),
                external_refs: Vec::new(),
                node_shards: BTreeMap::new(),
            },
        )
    } else {
        PersistedReferenceCacheV2 {
            schema_version: CACHE_SCHEMA_VERSION_V2,
            project_root: config.root.to_string_lossy().to_string(),
            config_fingerprint: config_fingerprint(config),
            file_hashes: BTreeMap::new(),
            file_hash_history: BTreeMap::new(),
            path_to_abs: HashMap::new(),
            id_to_path_map: HashMap::new(),
            external_refs: Vec::new(),
            node_shards: BTreeMap::new(),
        }
    };

    // Reset metadata when root/fingerprint changed.
    if meta.project_root != config.root.to_string_lossy()
        || meta.config_fingerprint != config_fingerprint(config)
    {
        meta = PersistedReferenceCacheV2 {
            schema_version: CACHE_SCHEMA_VERSION_V2,
            project_root: config.root.to_string_lossy().to_string(),
            config_fingerprint: config_fingerprint(config),
            file_hashes: BTreeMap::new(),
            file_hash_history: BTreeMap::new(),
            path_to_abs: HashMap::new(),
            id_to_path_map: HashMap::new(),
            external_refs: Vec::new(),
            node_shards: BTreeMap::new(),
        };
    }

    // Collect the set of abs_paths being upserted so we can purge stale IDs.
    let upsert_abs: std::collections::HashSet<&str> =
        build.nodes.keys().map(|s| s.as_str()).collect();

    // Remove stale NodeIds for files being overwritten: old IDs are invalid
    // after recompile (solc assigns new integers), so keep id_to_path_map and
    // external_refs clean to avoid phantom cross-file reference hits.
    meta.id_to_path_map
        .retain(|_id, path| !upsert_abs.contains(path.as_str()));
    meta.external_refs
        .retain(|r| !upsert_abs.contains(r.src.as_str()));

    let mut touched = 0usize;
    for (abs_path, file_nodes) in &build.nodes {
        let abs = Path::new(abs_path);
        let rel = relative_to_root(&config.root, abs);
        let shard_name = shard_file_name_for_rel_path(&rel);
        let shard_path = shards_dir.join(&shard_name);

        let mut entries = Vec::with_capacity(file_nodes.len());
        for (id, info) in file_nodes {
            entries.push(PersistedNodeEntry {
                id: id.0,
                info: info.clone(),
            });
        }
        let shard = PersistedFileShardV2 {
            abs_path: abs_path.clone(),
            entries,
        };
        let shard_payload =
            serde_json::to_vec(&shard).map_err(|e| format!("serialize shard {}: {e}", rel))?;
        write_atomic_json(&shard_path, &shard_payload)?;

        if let Some(hash) = file_hash(abs) {
            push_hash_history(&mut meta, &rel, &hash);
            meta.file_hashes.insert(rel.clone(), hash);
            meta.node_shards.insert(rel, shard_name);
            touched += 1;
        }

        meta.path_to_abs.insert(abs_path.clone(), abs_path.clone());
    }

    for (k, v) in &build.id_to_path_map {
        meta.id_to_path_map.insert(k.clone(), v.clone());
    }

    // Also prune external_refs whose target decl_id no longer exists in
    // id_to_path_map (i.e. old IDs from the recompiled files that were just
    // removed above).  This prevents stale cross-file reference hits.
    let live_ids: std::collections::HashSet<u64> = meta
        .id_to_path_map
        .keys()
        .filter_map(|k| k.parse().ok())
        .collect();
    meta.external_refs.retain(|r| live_ids.contains(&r.decl_id));

    // Append replacement external_refs from the partial build so that
    // reference resolution still works for touched files without waiting
    // for a full cache save.
    for (src, decl_id) in &build.external_refs {
        meta.external_refs.push(PersistedExternalRef {
            src: src.clone(),
            decl_id: decl_id.0,
        });
    }

    let payload_v2 = serde_json::to_vec(&meta).map_err(|e| format!("serialize v2 cache: {e}"))?;
    write_atomic_json(&meta_path, &payload_v2)?;

    Ok(CacheSaveReport {
        file_count_hashed: touched,
        duration_ms: started.elapsed().as_millis(),
    })
}

pub fn save_reference_cache_with_report(
    config: &FoundryConfig,
    build: &CachedBuild,
    source_files: Option<&[PathBuf]>,
) -> Result<CacheSaveReport, String> {
    let started = Instant::now();
    if !config.root.is_dir() {
        return Err(format!("invalid project root: {}", config.root.display()));
    }

    // When an explicit file list is given, hash only those.
    // Otherwise derive the list from the build's node keys (the files that
    // were actually compiled) — this avoids walking unrelated lib files.
    let file_hashes = if let Some(files) = source_files {
        hash_file_list(config, files)?
    } else {
        let build_paths: Vec<PathBuf> = build.nodes.keys().map(PathBuf::from).collect();
        if build_paths.is_empty() {
            current_file_hashes(config, true)?
        } else {
            hash_file_list(config, &build_paths)?
        }
    };
    let file_count_hashed = file_hashes.len();
    let external_refs = build
        .external_refs
        .iter()
        .map(|(src, id)| PersistedExternalRef {
            src: src.clone(),
            decl_id: id.0,
        })
        .collect::<Vec<_>>();

    let (_cache_root, shards_dir) = ensure_cache_dir_layout(&config.root)?;

    let mut node_shards: BTreeMap<String, String> = BTreeMap::new();
    let mut live_shards = std::collections::HashSet::new();
    for (abs_path, file_nodes) in &build.nodes {
        let abs = Path::new(abs_path);
        let rel = relative_to_root(&config.root, abs);
        let shard_name = shard_file_name_for_rel_path(&rel);
        let shard_path = shards_dir.join(&shard_name);

        let mut entries = Vec::with_capacity(file_nodes.len());
        for (id, info) in file_nodes {
            entries.push(PersistedNodeEntry {
                id: id.0,
                info: info.clone(),
            });
        }
        let shard = PersistedFileShardV2 {
            abs_path: abs_path.clone(),
            entries,
        };
        let shard_payload =
            serde_json::to_vec(&shard).map_err(|e| format!("serialize shard {}: {e}", rel))?;
        write_atomic_json(&shard_path, &shard_payload)?;
        node_shards.insert(rel, shard_name.clone());
        live_shards.insert(shard_name);
    }

    // Best-effort cleanup of stale shard files.
    if let Ok(dir) = fs::read_dir(&shards_dir) {
        for entry in dir.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !live_shards.contains(&file_name) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let persisted_v2 = PersistedReferenceCacheV2 {
        schema_version: CACHE_SCHEMA_VERSION_V2,
        project_root: config.root.to_string_lossy().to_string(),
        config_fingerprint: config_fingerprint(config),
        file_hashes: file_hashes.clone(),
        file_hash_history: {
            let mut h = BTreeMap::new();
            for (rel, hash) in &file_hashes {
                h.insert(rel.clone(), vec![hash.clone()]);
            }
            h
        },
        path_to_abs: build.path_to_abs.clone(),
        external_refs: external_refs.clone(),
        id_to_path_map: build.id_to_path_map.clone(),
        node_shards,
    };
    let payload_v2 =
        serde_json::to_vec(&persisted_v2).map_err(|e| format!("serialize v2 cache: {e}"))?;
    write_atomic_json(&cache_file_path_v2(&config.root), &payload_v2)?;

    Ok(CacheSaveReport {
        file_count_hashed,
        duration_ms: started.elapsed().as_millis(),
    })
}

pub fn load_reference_cache(config: &FoundryConfig) -> Option<CachedBuild> {
    load_reference_cache_with_report(config, ProjectIndexCacheMode::Auto, false).build
}

/// Return absolute paths of source files whose current hash differs from v2
/// cache metadata (including newly-added files missing from metadata).
pub fn changed_files_since_v2_cache(
    config: &FoundryConfig,
    _include_libs: bool,
) -> Result<Vec<PathBuf>, String> {
    if !config.root.is_dir() {
        return Err(format!("invalid project root: {}", config.root.display()));
    }

    let cache_path_v2 = cache_file_path_v2(&config.root);
    let bytes = fs::read(&cache_path_v2).map_err(|e| format!("cache file read failed: {e}"))?;
    let persisted: PersistedReferenceCacheV2 =
        serde_json::from_slice(&bytes).map_err(|e| format!("cache decode failed: {e}"))?;

    if persisted.schema_version != CACHE_SCHEMA_VERSION_V2 {
        return Err(format!(
            "schema mismatch: cache={}, expected={}",
            persisted.schema_version, CACHE_SCHEMA_VERSION_V2
        ));
    }
    if persisted.project_root != config.root.to_string_lossy() {
        return Err("project root mismatch".to_string());
    }
    if persisted.config_fingerprint != config_fingerprint(config) {
        return Err("config fingerprint mismatch".to_string());
    }

    // Hash only the files listed in the saved cache (the compiled closure),
    // not every file discovered by walking lib dirs.
    let saved_paths: Vec<PathBuf> = persisted
        .file_hashes
        .keys()
        .map(|rel| config.root.join(rel))
        .collect();
    let current_hashes = hash_file_list(config, &saved_paths)?;
    let mut changed = Vec::new();
    for (rel, current_hash) in current_hashes {
        match persisted.file_hashes.get(&rel) {
            Some(prev) if prev == &current_hash => {}
            _ => changed.push(config.root.join(rel)),
        }
    }
    Ok(changed)
}

pub fn load_reference_cache_with_report(
    config: &FoundryConfig,
    cache_mode: ProjectIndexCacheMode,
    _include_libs: bool,
) -> CacheLoadReport {
    let started = Instant::now();
    let miss = |reason: String, file_count_hashed: usize, duration_ms: u128| CacheLoadReport {
        build: None,
        hit: false,
        miss_reason: Some(reason),
        file_count_hashed,
        file_count_reused: 0,
        complete: false,
        duration_ms,
    };

    if !config.root.is_dir() {
        return miss(
            format!("invalid project root: {}", config.root.display()),
            0,
            started.elapsed().as_millis(),
        );
    }

    let should_try_v2 = matches!(
        cache_mode,
        ProjectIndexCacheMode::Auto | ProjectIndexCacheMode::V2
    );

    // Try v2 first (partial warm-start capable).
    let cache_path_v2 = cache_file_path_v2(&config.root);
    if should_try_v2
        && let Ok(bytes) = fs::read(&cache_path_v2)
        && let Ok(persisted) = serde_json::from_slice::<PersistedReferenceCacheV2>(&bytes)
    {
        if persisted.schema_version != CACHE_SCHEMA_VERSION_V2 {
            return miss(
                format!(
                    "schema mismatch: cache={}, expected={}",
                    persisted.schema_version, CACHE_SCHEMA_VERSION_V2
                ),
                0,
                started.elapsed().as_millis(),
            );
        }
        if persisted.project_root != config.root.to_string_lossy() {
            return miss(
                "project root mismatch".to_string(),
                0,
                started.elapsed().as_millis(),
            );
        }
        if persisted.config_fingerprint != config_fingerprint(config) {
            return miss(
                "config fingerprint mismatch".to_string(),
                0,
                started.elapsed().as_millis(),
            );
        }

        // Hash only the files that were saved — no rediscovery needed.
        // This means we compare exactly the closure that was compiled last time.
        let saved_paths: Vec<PathBuf> = persisted
            .file_hashes
            .keys()
            .map(|rel| config.root.join(rel))
            .collect();
        let current_hashes = match hash_file_list(config, &saved_paths) {
            Ok(h) => h,
            Err(e) => return miss(e, 0, started.elapsed().as_millis()),
        };
        let file_count_hashed = current_hashes.len();

        let shards_dir = cache_shards_dir_v2(&config.root);
        let mut nodes: HashMap<String, HashMap<NodeId, NodeInfo>> = HashMap::new();
        let mut file_count_reused = 0usize;
        let mut reused_decl_ids = std::collections::HashSet::new();

        for (rel_path, current_hash) in &current_hashes {
            let Some(cached_hash) = persisted.file_hashes.get(rel_path) else {
                continue;
            };
            if cached_hash != current_hash {
                continue;
            }
            let Some(shard_name) = persisted.node_shards.get(rel_path) else {
                continue;
            };
            let shard_path = shards_dir.join(shard_name);
            let shard_bytes = match fs::read(&shard_path) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let shard: PersistedFileShardV2 = match serde_json::from_slice(&shard_bytes) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let mut file_nodes = HashMap::with_capacity(shard.entries.len());
            for entry in shard.entries {
                reused_decl_ids.insert(entry.id);
                file_nodes.insert(NodeId(entry.id), entry.info);
            }
            nodes.insert(shard.abs_path, file_nodes);
            file_count_reused += 1;
        }

        if file_count_reused == 0 {
            return miss(
                "v2 cache: no reusable files".to_string(),
                file_count_hashed,
                started.elapsed().as_millis(),
            );
        }

        let mut external_refs = HashMap::new();
        for item in persisted.external_refs {
            if reused_decl_ids.contains(&item.decl_id) {
                external_refs.insert(item.src, NodeId(item.decl_id));
            }
        }

        // Complete = every saved file was reused with a matching hash.
        let complete =
            file_count_reused == file_count_hashed && current_hashes == persisted.file_hashes;

        return CacheLoadReport {
            build: Some(CachedBuild::from_reference_index(
                nodes,
                persisted.path_to_abs,
                external_refs,
                persisted.id_to_path_map,
                0,
            )),
            hit: true,
            miss_reason: if complete {
                None
            } else {
                Some("v2 cache partial reuse".to_string())
            },
            file_count_hashed,
            file_count_reused,
            complete,
            duration_ms: started.elapsed().as_millis(),
        };
    }

    miss(
        "cache mode v2: no usable v2 cache".to_string(),
        0,
        started.elapsed().as_millis(),
    )
}
