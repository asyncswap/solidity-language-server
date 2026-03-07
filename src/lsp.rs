use crate::completion;
use crate::config::{self, FoundryConfig, LintConfig, Settings};
use crate::file_operations;
use crate::folding;
use crate::goto;
use crate::highlight;
use crate::hover;
use crate::inlay_hints;
use crate::links;
use crate::references;
use crate::rename;
use crate::runner::{ForgeRunner, Runner};
use crate::selection;
use crate::semantic_tokens;
use crate::symbols;
use crate::types::DocumentUri;
use crate::types::ErrorCode;
use crate::utils;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tower_lsp::{Client, LanguageServer, lsp_types::*};

/// Per-document semantic token cache: `result_id` + token list.
type SemanticTokenCache = HashMap<DocumentUri, (String, Vec<SemanticToken>)>;

#[derive(Clone)]
pub struct ForgeLsp {
    client: Client,
    compiler: Arc<dyn Runner>,
    ast_cache: Arc<RwLock<HashMap<DocumentUri, Arc<goto::CachedBuild>>>>,
    /// Text cache for opened documents
    ///
    /// The key is the file's URI converted to string, and the value is a tuple of (version, content).
    text_cache: Arc<RwLock<HashMap<DocumentUri, (i32, String)>>>,
    completion_cache: Arc<RwLock<HashMap<DocumentUri, Arc<completion::CompletionCache>>>>,
    /// Cached lint configuration from `foundry.toml`.
    lint_config: Arc<RwLock<LintConfig>>,
    /// Cached project configuration from `foundry.toml`.
    foundry_config: Arc<RwLock<FoundryConfig>>,
    /// Client capabilities received during initialization.
    client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
    /// Editor-provided settings (from `initializationOptions` / `didChangeConfiguration`).
    settings: Arc<RwLock<Settings>>,
    /// Whether to use solc directly for AST generation (with forge fallback).
    use_solc: bool,
    /// Cache of semantic tokens per document for delta support.
    semantic_token_cache: Arc<RwLock<SemanticTokenCache>>,
    /// Monotonic counter for generating unique result_ids.
    semantic_token_id: Arc<AtomicU64>,
    /// Workspace root URI from `initialize`. Used for project-wide file discovery.
    root_uri: Arc<RwLock<Option<Url>>>,
    /// Whether background project indexing has already been triggered.
    project_indexed: Arc<std::sync::atomic::AtomicBool>,
    /// Whether workspace file operations changed project structure and
    /// the persisted reference cache should be refreshed from disk.
    project_cache_dirty: Arc<std::sync::atomic::AtomicBool>,
    /// Whether a didSave cache-sync worker is currently running.
    project_cache_sync_running: Arc<std::sync::atomic::AtomicBool>,
    /// Whether a didSave cache-sync pass is pending (set by save bursts).
    project_cache_sync_pending: Arc<std::sync::atomic::AtomicBool>,
    /// When true the next cache-sync worker iteration must run a full project
    /// rebuild, bypassing the incremental scoped path.  Set by
    /// `solidity.reindex` to guarantee a complete reindex even when a worker
    /// spawned by didSave is already running with `aggressive_scoped=true`.
    project_cache_force_full_rebuild: Arc<std::sync::atomic::AtomicBool>,
    /// Whether a didSave v2-upsert worker is currently running.
    project_cache_upsert_running: Arc<std::sync::atomic::AtomicBool>,
    /// Whether a didSave v2-upsert pass is pending (set by save bursts).
    project_cache_upsert_pending: Arc<std::sync::atomic::AtomicBool>,
    /// Absolute file paths changed during the session and awaiting dirty-sync
    /// planning for aggressive affected-closure reindex.
    project_cache_changed_files: Arc<RwLock<HashSet<String>>>,
    /// Absolute file paths queued for debounced v2 shard upserts.
    project_cache_upsert_files: Arc<RwLock<HashSet<String>>>,
    /// URIs recently scaffolded in willCreateFiles (used to avoid re-applying
    /// edits again in didCreateFiles for the same create operation).
    pending_create_scaffold: Arc<RwLock<HashSet<DocumentUri>>>,
    /// Whether settings were loaded from `initializationOptions`.  When false,
    /// the server will pull settings via `workspace/configuration` during
    /// `initialized()`.
    settings_from_init: Arc<std::sync::atomic::AtomicBool>,
    /// Per-URI watch channels for serialising didSave background work.
    ///
    /// Each URI gets one long-lived worker task that always processes the
    /// *latest* save params.  Rapid saves collapse: the worker wakes once per
    /// compile cycle and picks up the newest params via `borrow_and_update`.
    did_save_workers: Arc<
        RwLock<HashMap<DocumentUri, tokio::sync::watch::Sender<Option<DidSaveTextDocumentParams>>>>,
    >,
    /// JSON-driven code-action database loaded once at startup.
    code_action_db: Arc<HashMap<ErrorCode, crate::code_actions::CodeActionDef>>,
    /// Cached builds loaded from sub-project (lib) caches.
    ///
    /// When `fullProjectScan` is enabled, the server discovers existing LSP
    /// caches in lib sub-projects (e.g. `lib/v4-core/.solidity-language-server/`)
    /// and loads them.  The references handler searches these alongside the
    /// main project build to find cross-file references in lib test files
    /// that aren't part of the root project's compilation closure.
    ///
    /// Each sub-cache has its own node ID space — matching across caches
    /// is done by absolute file path + byte offset, not by node ID.
    sub_caches: Arc<RwLock<Vec<Arc<goto::CachedBuild>>>>,
    /// Guards against multiple concurrent sub-cache loading tasks.
    sub_caches_loading: Arc<std::sync::atomic::AtomicBool>,
}

/// Spawn a background task to discover, build (if missing), and load caches
/// from lib sub-projects.  Non-blocking: returns immediately.
///
/// Accepts already-resolved `FoundryConfig` so it can be called from inside
/// `tokio::spawn` closures that don't have access to `&self`.
fn spawn_load_lib_sub_caches_task(
    foundry_config: crate::config::FoundryConfig,
    sub_caches: Arc<RwLock<Vec<Arc<goto::CachedBuild>>>>,
    loading_flag: Arc<std::sync::atomic::AtomicBool>,
    client: Client,
) {
    // Atomic guard: only one task runs at a time.
    if loading_flag
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return;
    }
    tokio::spawn(async move {
        let cfg = foundry_config.clone();
        let discovered = tokio::task::spawn_blocking(move || {
            crate::project_cache::discover_lib_sub_projects(&cfg)
        })
        .await
        .unwrap_or_else(|_| crate::project_cache::DiscoveredLibs {
            cached: Vec::new(),
            uncached: Vec::new(),
        });

        // Build missing caches first (compile each sub-project).
        for sub_root in &discovered.uncached {
            let sub_config =
                crate::config::load_foundry_config_from_toml(&sub_root.join("foundry.toml"));
            client
                .log_message(
                    MessageType::INFO,
                    format!("sub-cache: building {} ...", sub_root.display()),
                )
                .await;
            match crate::solc::solc_project_index(&sub_config, Some(&client), None).await {
                Ok(ast_data) => {
                    let build = crate::goto::CachedBuild::new(ast_data, 0);
                    let source_count = build.nodes.len();
                    if source_count == 0 {
                        client
                            .log_message(
                                MessageType::WARNING,
                                format!(
                                    "sub-cache: build produced 0 sources for {}",
                                    sub_root.display()
                                ),
                            )
                            .await;
                        continue;
                    }
                    // Persist to disk so next startup can load from cache.
                    let cfg_for_save = sub_config.clone();
                    let build_for_save = build.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        crate::project_cache::save_reference_cache_with_report(
                            &cfg_for_save,
                            &build_for_save,
                            None,
                        )
                    })
                    .await;
                    client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "sub-cache: built and saved {} (sources={})",
                                sub_root.display(),
                                source_count
                            ),
                        )
                        .await;
                }
                Err(e) => {
                    client
                        .log_message(
                            MessageType::WARNING,
                            format!("sub-cache: build failed for {}: {}", sub_root.display(), e),
                        )
                        .await;
                }
            }
        }

        // Now load all caches (existing + newly built).
        let cfg2 = foundry_config.clone();
        let all_cached =
            tokio::task::spawn_blocking(move || crate::project_cache::discover_lib_caches(&cfg2))
                .await
                .unwrap_or_default();

        if all_cached.is_empty() {
            return;
        }

        let mut loaded = Vec::new();
        for sub_root in &all_cached {
            let root = sub_root.clone();
            let build =
                tokio::task::spawn_blocking(move || crate::project_cache::load_lib_cache(&root))
                    .await
                    .ok()
                    .flatten();
            if let Some(build) = build {
                let source_count = build.nodes.len();
                client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "sub-cache loaded: {} (sources={})",
                            sub_root.display(),
                            source_count
                        ),
                    )
                    .await;
                loaded.push(Arc::new(build));
            }
        }

        if !loaded.is_empty() {
            let total: usize = loaded.iter().map(|b| b.nodes.len()).sum();
            client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "sub-caches: loaded {} lib caches ({} total sources)",
                        loaded.len(),
                        total
                    ),
                )
                .await;
            *sub_caches.write().await = loaded;
        }
    });
}

impl ForgeLsp {
    pub fn new(client: Client, use_solar: bool, use_solc: bool) -> Self {
        let compiler: Arc<dyn Runner> = if use_solar {
            Arc::new(crate::solar_runner::SolarRunner)
        } else {
            Arc::new(ForgeRunner)
        };
        let ast_cache = Arc::new(RwLock::new(HashMap::new()));
        let text_cache = Arc::new(RwLock::new(HashMap::new()));
        let completion_cache = Arc::new(RwLock::new(HashMap::new()));
        let lint_config = Arc::new(RwLock::new(LintConfig::default()));
        let foundry_config = Arc::new(RwLock::new(FoundryConfig::default()));
        let client_capabilities = Arc::new(RwLock::new(None));
        let settings = Arc::new(RwLock::new(Settings::default()));
        Self {
            client,
            compiler,
            ast_cache,
            text_cache,
            completion_cache,
            lint_config,
            foundry_config,
            client_capabilities,
            settings,
            use_solc,
            semantic_token_cache: Arc::new(RwLock::new(HashMap::new())),
            semantic_token_id: Arc::new(AtomicU64::new(0)),
            root_uri: Arc::new(RwLock::new(None)),
            project_indexed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_dirty: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_sync_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_sync_pending: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_force_full_rebuild: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_upsert_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_upsert_pending: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            project_cache_changed_files: Arc::new(RwLock::new(HashSet::new())),
            project_cache_upsert_files: Arc::new(RwLock::new(HashSet::new())),
            pending_create_scaffold: Arc::new(RwLock::new(HashSet::new())),
            settings_from_init: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            did_save_workers: Arc::new(RwLock::new(HashMap::new())),
            code_action_db: Arc::new(crate::code_actions::load()),
            sub_caches: Arc::new(RwLock::new(Vec::new())),
            sub_caches_loading: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Resolve the foundry configuration for a specific file.
    ///
    /// Looks for `foundry.toml` starting from the file's own directory, which
    /// handles files in nested projects (e.g. `lib/`, `example/`,
    /// `node_modules/`).  When no `foundry.toml` exists at all (Hardhat, bare
    /// projects), the file's git root or parent directory is used as the
    /// project root so solc can still resolve imports.
    async fn foundry_config_for_file(&self, file_path: &std::path::Path) -> FoundryConfig {
        config::load_foundry_config(file_path)
    }

    /// Canonical project cache key for project-wide index entries.
    ///
    /// Prefer workspace root URI when available. If not provided by the
    /// client, derive a file URI from detected foundry root.
    async fn project_cache_key(&self) -> Option<String> {
        if let Some(uri) = self.root_uri.read().await.as_ref() {
            return Some(uri.to_string());
        }

        let mut root = self.foundry_config.read().await.root.clone();
        if !root.is_absolute()
            && let Ok(cwd) = std::env::current_dir()
        {
            root = cwd.join(root);
        }
        if !root.is_dir() {
            root = root.parent()?.to_path_buf();
        }
        Url::from_directory_path(root).ok().map(|u| u.to_string())
    }

    /// Spawn a background task to discover and load existing caches from lib
    /// sub-projects.
    ///
    /// Non-blocking: returns immediately.  The background task discovers
    /// sub-projects with `.solidity-language-server/` cache dirs and loads
    /// them into `self.sub_caches`.  The references handler reads whatever
    /// sub-caches are available at query time — if loading hasn't finished
    /// yet, the first query returns only root-project references.
    fn spawn_load_lib_sub_caches(&self) {
        let foundry_config = self.foundry_config.clone();
        let sub_caches = self.sub_caches.clone();
        let loading_flag = self.sub_caches_loading.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            let cfg = foundry_config.read().await.clone();
            spawn_load_lib_sub_caches_task(cfg, sub_caches, loading_flag, client);
        });
    }

    /// Ensure project-wide cached build is available for cross-file features.
    ///
    /// Fast path: return in-memory root build if present.
    /// Slow path: load persisted cache from disk and insert it under project key.
    async fn ensure_project_cached_build(&self) -> Option<Arc<goto::CachedBuild>> {
        let root_key = self.project_cache_key().await?;
        if let Some(existing) = self.ast_cache.read().await.get(&root_key).cloned() {
            // Kick off sub-cache loading in the background (no-op if already loaded).
            self.spawn_load_lib_sub_caches();
            return Some(existing);
        }

        let settings = self.settings.read().await.clone();
        if !self.use_solc || !settings.project_index.full_project_scan {
            return None;
        }

        let foundry_config = self.foundry_config.read().await.clone();
        if !foundry_config.root.is_dir() {
            return None;
        }

        let cache_mode = settings.project_index.cache_mode.clone();
        let cfg_for_load = foundry_config.clone();
        let load_res = tokio::task::spawn_blocking(move || {
            crate::project_cache::load_reference_cache_with_report(&cfg_for_load, cache_mode, true)
        })
        .await;

        let Ok(report) = load_res else {
            return None;
        };
        let Some(build) = report.build else {
            return None;
        };

        let source_count = build.nodes.len();
        let complete = report.complete;
        let duration_ms = report.duration_ms;
        let reused = report.file_count_reused;
        let hashed = report.file_count_hashed;
        let arc = Arc::new(build);
        self.ast_cache
            .write()
            .await
            .insert(root_key.clone().into(), arc.clone());
        self.project_indexed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "references warm-load: project cache loaded (sources={}, reused_files={}/{}, complete={}, duration={}ms)",
                    source_count, reused, hashed, complete, duration_ms
                ),
            )
            .await;

        // Kick off sub-cache loading in the background (no-op if already loaded).
        self.spawn_load_lib_sub_caches();

        if complete {
            return Some(arc);
        }

        // Partial warm load: immediately reconcile changed files only, merge,
        // and persist back to disk so subsequent opens are fast and complete.
        let cfg_for_diff = foundry_config.clone();
        let changed = tokio::task::spawn_blocking(move || {
            crate::project_cache::changed_files_since_v2_cache(&cfg_for_diff, true)
        })
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default();

        if changed.is_empty() {
            return Some(arc);
        }

        let remappings = crate::solc::resolve_remappings(&foundry_config).await;
        let cfg_for_plan = foundry_config.clone();
        let changed_for_plan = changed.clone();
        let remappings_for_plan = remappings.clone();
        let affected_set = tokio::task::spawn_blocking(move || {
            compute_reverse_import_closure(&cfg_for_plan, &changed_for_plan, &remappings_for_plan)
        })
        .await
        .ok()
        .unwrap_or_default();
        let mut affected_files: Vec<PathBuf> = affected_set.into_iter().collect();
        if affected_files.is_empty() {
            affected_files = changed;
        }

        let text_cache_snapshot = self.text_cache.read().await.clone();
        match crate::solc::solc_project_index_scoped(
            &foundry_config,
            Some(&self.client),
            Some(&text_cache_snapshot),
            &affected_files,
        )
        .await
        {
            Ok(ast_data) => {
                let scoped_build = Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                let mut merge_error: Option<String> = None;
                let merged = {
                    let mut cache = self.ast_cache.write().await;
                    let merged = if let Some(existing) = cache.get(&root_key).cloned() {
                        let mut merged = (*existing).clone();
                        match merge_scoped_cached_build(&mut merged, (*scoped_build).clone()) {
                            Ok(_) => Arc::new(merged),
                            Err(e) => {
                                merge_error = Some(e);
                                scoped_build.clone()
                            }
                        }
                    } else {
                        scoped_build.clone()
                    };
                    cache.insert(root_key.clone().into(), merged.clone());
                    merged
                };
                if let Some(e) = merge_error {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!(
                                "references warm-load reconcile: merge failed, using scoped build: {}",
                                e
                            ),
                        )
                        .await;
                }

                let cfg_for_save = foundry_config.clone();
                let build_for_save = (*merged).clone();
                let save_res = tokio::task::spawn_blocking(move || {
                    crate::project_cache::save_reference_cache_with_report(
                        &cfg_for_save,
                        &build_for_save,
                        None,
                    )
                })
                .await;
                if let Ok(Ok(report)) = save_res {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "references warm-load reconcile: saved cache (affected={}, hashed_files={}, duration={}ms)",
                                affected_files.len(),
                                report.file_count_hashed,
                                report.duration_ms
                            ),
                        )
                        .await;
                }
                Some(merged)
            }
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!(
                            "references warm-load reconcile: scoped reindex failed: {}",
                            e
                        ),
                    )
                    .await;
                Some(arc)
            }
        }
    }

    /// Best-effort persistence of the current in-memory project index.
    ///
    /// This writes the root project CachedBuild to disk if available.
    async fn flush_project_cache_to_disk(&self, reason: &str) {
        if !self.use_solc || !self.settings.read().await.project_index.full_project_scan {
            return;
        }
        let Some(root_key) = self.project_cache_key().await else {
            return;
        };
        let build = {
            let cache = self.ast_cache.read().await;
            cache.get(&root_key).cloned()
        };
        let Some(build) = build else {
            return;
        };

        let foundry_config = self.foundry_config.read().await.clone();
        let build_for_save = (*build).clone();
        let res = tokio::task::spawn_blocking(move || {
            crate::project_cache::save_reference_cache_with_report(
                &foundry_config,
                &build_for_save,
                None,
            )
        })
        .await;

        match res {
            Ok(Ok(report)) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "project cache flush ({}): saved hashed_files={}, duration={}ms",
                            reason, report.file_count_hashed, report.duration_ms
                        ),
                    )
                    .await;
            }
            Ok(Err(e)) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("project cache flush ({}) failed: {}", reason, e),
                    )
                    .await;
            }
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("project cache flush ({}) task failed: {}", reason, e),
                    )
                    .await;
            }
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let uri = params.uri.clone();
        let version = params.version;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file URI")
                    .await;
                return;
            }
        };

        let path_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file path")
                    .await;
                return;
            }
        };

        // Skip rebuild if the content is identical to what was last compiled.
        // This avoids a redundant solc invocation on format-on-save loops where
        // the formatter returns edits, Neovim applies them, saves again, and the
        // resulting text is already fully formatted (same bytes as last build).
        {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            params.text.hash(&mut hasher);
            let incoming_hash = hasher.finish();

            let cache = self.ast_cache.read().await;
            if let Some(cached) = cache.get(&uri.to_string()) {
                if cached.content_hash != 0 && cached.content_hash == incoming_hash {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            "on_change: content unchanged since last build, skipping rebuild",
                        )
                        .await;
                    return;
                }
            }
        }

        // Clear stale diagnostics immediately so the user sees instant feedback
        // while solc is compiling.  Fresh diagnostics (if any) are published
        // below once the build finishes.
        self.client
            .publish_diagnostics(uri.clone(), vec![], None)
            .await;

        // Check if linting should be skipped based on foundry.toml + editor settings.
        let (should_lint, lint_settings) = {
            let lint_cfg = self.lint_config.read().await;
            let settings = self.settings.read().await;
            let enabled = lint_cfg.should_lint(&file_path) && settings.lint.enabled;
            let ls = settings.lint.clone();
            (enabled, ls)
        };

        // When use_solc is enabled, run solc once for both AST and diagnostics.
        // This avoids running `forge build` separately (~27s on large projects).
        // On solc failure, fall back to the forge-based pipeline.
        let (lint_result, build_result, ast_result) = if self.use_solc {
            let foundry_cfg = self.foundry_config_for_file(&file_path).await;
            let solc_future = crate::solc::solc_ast(path_str, &foundry_cfg, Some(&self.client));

            if should_lint {
                let (lint, solc) = tokio::join!(
                    self.compiler.get_lint_diagnostics(&uri, &lint_settings),
                    solc_future
                );
                match solc {
                    Ok(data) => {
                        self.client
                            .log_message(
                                MessageType::INFO,
                                "solc: AST + diagnostics from single run",
                            )
                            .await;
                        // Extract diagnostics from the same solc output
                        let content = tokio::fs::read_to_string(&file_path)
                            .await
                            .unwrap_or_default();
                        let build_diags = crate::build::build_output_to_diagnostics(
                            &data,
                            &file_path,
                            &content,
                            &foundry_cfg.ignored_error_codes,
                        );
                        (Some(lint), Ok(build_diags), Ok(data))
                    }
                    Err(e) => {
                        self.client
                            .log_message(
                                MessageType::WARNING,
                                format!("solc failed, falling back to forge: {e}"),
                            )
                            .await;
                        let (build, ast) = tokio::join!(
                            self.compiler.get_build_diagnostics(&uri),
                            self.compiler.ast(path_str)
                        );
                        (Some(lint), build, ast)
                    }
                }
            } else {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("skipping lint for ignored file: {path_str}"),
                    )
                    .await;
                match solc_future.await {
                    Ok(data) => {
                        self.client
                            .log_message(
                                MessageType::INFO,
                                "solc: AST + diagnostics from single run",
                            )
                            .await;
                        let content = tokio::fs::read_to_string(&file_path)
                            .await
                            .unwrap_or_default();
                        let build_diags = crate::build::build_output_to_diagnostics(
                            &data,
                            &file_path,
                            &content,
                            &foundry_cfg.ignored_error_codes,
                        );
                        (None, Ok(build_diags), Ok(data))
                    }
                    Err(e) => {
                        self.client
                            .log_message(
                                MessageType::WARNING,
                                format!("solc failed, falling back to forge: {e}"),
                            )
                            .await;
                        let (build, ast) = tokio::join!(
                            self.compiler.get_build_diagnostics(&uri),
                            self.compiler.ast(path_str)
                        );
                        (None, build, ast)
                    }
                }
            }
        } else {
            // forge-only pipeline (--use-forge)
            if should_lint {
                let (lint, build, ast) = tokio::join!(
                    self.compiler.get_lint_diagnostics(&uri, &lint_settings),
                    self.compiler.get_build_diagnostics(&uri),
                    self.compiler.ast(path_str)
                );
                (Some(lint), build, ast)
            } else {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("skipping lint for ignored file: {path_str}"),
                    )
                    .await;
                let (build, ast) = tokio::join!(
                    self.compiler.get_build_diagnostics(&uri),
                    self.compiler.ast(path_str)
                );
                (None, build, ast)
            }
        };

        // Only replace cache with new AST if build succeeded (no errors; warnings are OK)
        let build_succeeded = matches!(&build_result, Ok(diagnostics) if diagnostics.iter().all(|d| d.severity != Some(DiagnosticSeverity::ERROR)));

        // Compute content hash once — used to stamp the build and skip
        // future rebuilds when content hasn't changed.
        let content_hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            params.text.hash(&mut hasher);
            hasher.finish()
        };

        if build_succeeded {
            if let Ok(ast_data) = ast_result {
                let mut cached_build = goto::CachedBuild::new(ast_data, version);
                cached_build.content_hash = content_hash;
                let cached_build = Arc::new(cached_build);
                let mut cache = self.ast_cache.write().await;
                cache.insert(uri.to_string().into(), cached_build.clone());
                drop(cache);

                // Insert pre-built completion cache (built during CachedBuild::new)
                {
                    let mut cc = self.completion_cache.write().await;
                    cc.insert(
                        uri.to_string().into(),
                        cached_build.completion_cache.clone(),
                    );
                }
                self.client
                    .log_message(MessageType::INFO, "Build successful, AST cache updated")
                    .await;
            } else if let Err(e) = ast_result {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Build succeeded but failed to get AST: {e}"),
                    )
                    .await;
            }
        } else {
            // Build has errors — keep the existing AST (don't invalidate
            // navigation) but stamp the content_hash so the next save with
            // *different* content is not falsely skipped by the early-return
            // guard above.  Without this, fixing an error and reverting to a
            // previously-compiled state would match the old hash → skip the
            // rebuild → leave stale error diagnostics in the editor.
            {
                let mut cache = self.ast_cache.write().await;
                let uri_key = uri.to_string();
                if let Some(existing) = cache.get(&uri_key).cloned() {
                    let mut updated = (*existing).clone();
                    updated.content_hash = content_hash;
                    cache.insert(uri_key.into(), Arc::new(updated));
                }
            }
            self.client
                .log_message(
                    MessageType::INFO,
                    "Build errors detected, keeping existing AST cache",
                )
                .await;
        }

        // cache text — only if no newer version exists (e.g. from formatting/did_change)
        {
            let mut text_cache = self.text_cache.write().await;
            let uri_str = uri.to_string();
            let existing_version = text_cache.get(&uri_str).map(|(v, _)| *v).unwrap_or(-1);
            if version >= existing_version {
                text_cache.insert(uri_str.into(), (version, params.text));
            }
        }

        let mut all_diagnostics = vec![];

        if let Some(lint_result) = lint_result {
            match lint_result {
                Ok(mut lints) => {
                    // Filter out excluded lint rules from editor settings.
                    if !lint_settings.exclude.is_empty() {
                        lints.retain(|d| {
                            if let Some(NumberOrString::String(code)) = &d.code {
                                !lint_settings.exclude.iter().any(|ex| ex == code)
                            } else {
                                true
                            }
                        });
                    }
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("found {} lint diagnostics", lints.len()),
                        )
                        .await;
                    all_diagnostics.append(&mut lints);
                }
                Err(e) => {
                    self.client
                        .log_message(
                            MessageType::ERROR,
                            format!("Forge lint diagnostics failed: {e}"),
                        )
                        .await;
                }
            }
        }

        match build_result {
            Ok(mut builds) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("found {} build diagnostics", builds.len()),
                    )
                    .await;
                all_diagnostics.append(&mut builds);
            }
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Forge build diagnostics failed: {e}"),
                    )
                    .await;
            }
        }

        // Sanitize: some LSP clients (e.g. trunk.io) crash on diagnostics with
        // empty message fields. Replace any empty message with a safe fallback
        // before publishing regardless of which diagnostic source produced it.
        for diag in &mut all_diagnostics {
            if diag.message.is_empty() {
                diag.message = "Unknown issue".to_string();
            }
        }

        // Publish diagnostics immediately — don't block on project indexing.
        self.client
            .publish_diagnostics(uri, all_diagnostics, None)
            .await;

        // Refresh inlay hints after everything is updated
        if build_succeeded {
            let client = self.client.clone();
            tokio::spawn(async move {
                let _ = client.inlay_hint_refresh().await;
            });
        }

        // Trigger project index in the background on first successful build.
        // This compiles all project files (src, test, script) in a single solc
        // invocation so that cross-file features (references, rename) discover
        // the full project. Runs asynchronously after diagnostics are published
        // so the user sees diagnostics immediately without waiting for the index.
        if build_succeeded
            && self.use_solc
            && self.settings.read().await.project_index.full_project_scan
            && !self
                .project_indexed
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            let cache_mode = self.settings.read().await.project_index.cache_mode.clone();
            self.project_indexed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let foundry_config = self.foundry_config.read().await.clone();
            let cache_key = self.project_cache_key().await;
            let ast_cache = self.ast_cache.clone();
            let client = self.client.clone();

            tokio::spawn(async move {
                let Some(cache_key) = cache_key else {
                    return;
                };
                if !foundry_config.root.is_dir() {
                    client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "project index: {} not found, skipping",
                                foundry_config.root.display(),
                            ),
                        )
                        .await;
                    return;
                }

                // Create a progress token to show indexing status in the editor.
                let token = NumberOrString::String("solidity/projectIndex".to_string());
                let _ = client
                    .send_request::<request::WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                        token: token.clone(),
                    })
                    .await;

                // Begin progress: show spinner in the status bar.
                client
                    .send_notification::<notification::Progress>(ProgressParams {
                        token: token.clone(),
                        value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                            WorkDoneProgressBegin {
                                title: "Indexing project".to_string(),
                                message: Some("Discovering source files...".to_string()),
                                cancellable: Some(false),
                                percentage: None,
                            },
                        )),
                    })
                    .await;

                // Try persisted reference index first (fast warm start).
                let cfg_for_load = foundry_config.clone();
                let cache_mode_for_load = cache_mode.clone();
                let load_res = tokio::task::spawn_blocking(move || {
                    crate::project_cache::load_reference_cache_with_report(
                        &cfg_for_load,
                        cache_mode_for_load,
                        true,
                    )
                })
                .await;
                match load_res {
                    Ok(report) => {
                        if let Some(cached_build) = report.build {
                            let source_count = cached_build.nodes.len();
                            ast_cache
                                .write()
                                .await
                                .insert(cache_key.clone().into(), Arc::new(cached_build));
                            client
                                .log_message(
                                    MessageType::INFO,
                                    format!(
                                        "project index: cache load hit (sources={}, reused_files={}/{}, complete={}, duration={}ms)",
                                        source_count,
                                        report.file_count_reused,
                                        report.file_count_hashed,
                                        report.complete,
                                        report.duration_ms
                                    ),
                                )
                                .await;
                            if report.complete {
                                client
                                    .send_notification::<notification::Progress>(ProgressParams {
                                        token: token.clone(),
                                        value: ProgressParamsValue::WorkDone(
                                            WorkDoneProgress::End(WorkDoneProgressEnd {
                                                message: Some(format!(
                                                    "Loaded {} source files from cache",
                                                    source_count
                                                )),
                                            }),
                                        ),
                                    })
                                    .await;
                                return;
                            }
                        }

                        client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "project index: cache load miss/partial (reason={}, reused_files={}/{}, duration={}ms)",
                                    report
                                        .miss_reason
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    report.file_count_reused,
                                    report.file_count_hashed,
                                    report.duration_ms
                                ),
                            )
                            .await;
                    }
                    Err(e) => {
                        client
                            .log_message(
                                MessageType::WARNING,
                                format!("project index: cache load task failed: {e}"),
                            )
                            .await;
                    }
                }

                match crate::solc::solc_project_index(&foundry_config, Some(&client), None).await {
                    Ok(ast_data) => {
                        let mut new_build = crate::goto::CachedBuild::new(ast_data, 0);
                        // Merge any files from the previous cache that the
                        // new build doesn't cover (preserves warm-loaded data).
                        if let Some(prev) = ast_cache.read().await.get(&cache_key) {
                            new_build.merge_missing_from(prev);
                        }
                        let source_count = new_build.nodes.len();
                        let cached_build = Arc::new(new_build);
                        let build_for_save = (*cached_build).clone();
                        ast_cache
                            .write()
                            .await
                            .insert(cache_key.clone().into(), cached_build);
                        client
                            .log_message(
                                MessageType::INFO,
                                format!("project index: cached {} source files", source_count),
                            )
                            .await;

                        let cfg_for_save = foundry_config.clone();
                        let client_for_save = client.clone();
                        tokio::spawn(async move {
                            let res = tokio::task::spawn_blocking(move || {
                                crate::project_cache::save_reference_cache_with_report(
                                    &cfg_for_save,
                                    &build_for_save,
                                    None,
                                )
                            })
                            .await;
                            match res {
                                Ok(Ok(report)) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::INFO,
                                            format!(
                                                "project index: cache save complete (hashed_files={}, duration={}ms)",
                                                report.file_count_hashed, report.duration_ms
                                            ),
                                        )
                                        .await;
                                }
                                Ok(Err(e)) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::WARNING,
                                            format!("project index: failed to persist cache: {e}"),
                                        )
                                        .await;
                                }
                                Err(e) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::WARNING,
                                            format!("project index: cache save task failed: {e}"),
                                        )
                                        .await;
                                }
                            }
                        });

                        // End progress: indexing complete.
                        client
                            .send_notification::<notification::Progress>(ProgressParams {
                                token: token.clone(),
                                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                                    WorkDoneProgressEnd {
                                        message: Some(format!(
                                            "Indexed {} source files",
                                            source_count
                                        )),
                                    },
                                )),
                            })
                            .await;
                    }
                    Err(e) => {
                        client
                            .log_message(MessageType::WARNING, format!("project index failed: {e}"))
                            .await;

                        // End progress on failure too.
                        client
                            .send_notification::<notification::Progress>(ProgressParams {
                                token: token.clone(),
                                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                                    WorkDoneProgressEnd {
                                        message: Some("Indexing failed".to_string()),
                                    },
                                )),
                            })
                            .await;
                    }
                }
            });
        }
    }

    /// Get a CachedBuild from the cache, or fetch and build one on demand.
    /// If `insert_on_miss` is true, the freshly-built entry is inserted into the cache
    /// (used by references handler so cross-file lookups can find it later).
    ///
    /// When the entry is in the cache but marked stale (text_cache changed
    /// since the last build), the text_cache content is flushed to disk and
    /// the AST is rebuilt so that rename / references work correctly on
    /// unsaved buffers.
    async fn get_or_fetch_build(
        &self,
        uri: &Url,
        file_path: &std::path::Path,
        insert_on_miss: bool,
    ) -> Option<Arc<goto::CachedBuild>> {
        let uri_str = uri.to_string();

        // Return cached entry if it exists (stale or not — stale entries are
        // still usable, positions may be slightly off like goto-definition).
        {
            let cache = self.ast_cache.read().await;
            if let Some(cached) = cache.get(&uri_str) {
                return Some(cached.clone());
            }
        }

        // Cache miss — if caller doesn't want to trigger a build, return None.
        // This prevents inlay hints, code lens, etc. from blocking on a full
        // solc/forge build. The cache will be populated by on_change (did_open/did_save).
        if !insert_on_miss {
            return None;
        }

        // Cache miss — build the AST from disk.
        let path_str = file_path.to_str()?;
        let ast_result = if self.use_solc {
            let foundry_cfg = self.foundry_config_for_file(&file_path).await;
            match crate::solc::solc_ast(path_str, &foundry_cfg, Some(&self.client)).await {
                Ok(data) => Ok(data),
                Err(_) => self.compiler.ast(path_str).await,
            }
        } else {
            self.compiler.ast(path_str).await
        };
        match ast_result {
            Ok(data) => {
                // Built from disk (cache miss) — use version 0; the next
                // didSave/on_change will stamp the correct version.
                let build = Arc::new(goto::CachedBuild::new(data, 0));
                let mut cache = self.ast_cache.write().await;
                cache.insert(uri_str.clone().into(), build.clone());
                Some(build)
            }
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("failed to get AST: {e}"))
                    .await;
                None
            }
        }
    }

    /// Get the source bytes for a file, preferring the in-memory text cache
    /// (which reflects unsaved editor changes) over reading from disk.
    async fn get_source_bytes(&self, uri: &Url, file_path: &std::path::Path) -> Option<Vec<u8>> {
        {
            let text_cache = self.text_cache.read().await;
            if let Some((_, content)) = text_cache.get(&uri.to_string()) {
                return Some(content.as_bytes().to_vec());
            }
        }
        match std::fs::read(file_path) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Benign during create/delete races when the editor emits
                    // didOpen/didChange before the file is materialized on disk.
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("file not found yet (transient): {e}"),
                        )
                        .await;
                } else {
                    self.client
                        .log_message(MessageType::ERROR, format!("failed to read file: {e}"))
                        .await;
                }
                None
            }
        }
    }
}

fn update_imports_on_delete_enabled(settings: &crate::config::Settings) -> bool {
    settings.file_operations.update_imports_on_delete
}

fn start_or_mark_project_cache_sync_pending(
    pending: &std::sync::atomic::AtomicBool,
    running: &std::sync::atomic::AtomicBool,
) -> bool {
    pending.store(true, Ordering::Release);
    running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn take_project_cache_sync_pending(pending: &std::sync::atomic::AtomicBool) -> bool {
    pending.swap(false, Ordering::AcqRel)
}

fn stop_project_cache_sync_worker_or_reclaim(
    pending: &std::sync::atomic::AtomicBool,
    running: &std::sync::atomic::AtomicBool,
) -> bool {
    running.store(false, Ordering::Release);
    pending.load(Ordering::Acquire)
        && running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
}

fn try_claim_project_cache_dirty(dirty: &std::sync::atomic::AtomicBool) -> bool {
    dirty
        .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn start_or_mark_project_cache_upsert_pending(
    pending: &std::sync::atomic::AtomicBool,
    running: &std::sync::atomic::AtomicBool,
) -> bool {
    pending.store(true, Ordering::Release);
    running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn take_project_cache_upsert_pending(pending: &std::sync::atomic::AtomicBool) -> bool {
    pending.swap(false, Ordering::AcqRel)
}

fn stop_project_cache_upsert_worker_or_reclaim(
    pending: &std::sync::atomic::AtomicBool,
    running: &std::sync::atomic::AtomicBool,
) -> bool {
    running.store(false, Ordering::Release);
    pending.load(Ordering::Acquire)
        && running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::RootDir => out.push(comp.as_os_str()),
            Component::Prefix(_) => out.push(comp.as_os_str()),
            Component::Normal(seg) => out.push(seg),
        }
    }
    out
}

fn resolve_import_spec_to_abs(
    project_root: &Path,
    importer_abs: &Path,
    import_path: &str,
    remappings: &[String],
) -> Option<PathBuf> {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        let base = importer_abs.parent()?;
        return Some(lexical_normalize(&base.join(import_path)));
    }

    for remap in remappings {
        let mut it = remap.splitn(2, '=');
        let prefix = it.next().unwrap_or_default();
        let target = it.next().unwrap_or_default();
        if prefix.is_empty() || target.is_empty() {
            continue;
        }
        if import_path.starts_with(prefix) {
            let suffix = import_path.strip_prefix(prefix).unwrap_or_default();
            return Some(lexical_normalize(
                &project_root.join(format!("{target}{suffix}")),
            ));
        }
    }

    Some(lexical_normalize(&project_root.join(import_path)))
}

fn compute_reverse_import_closure(
    config: &FoundryConfig,
    changed_abs: &[PathBuf],
    remappings: &[String],
) -> HashSet<PathBuf> {
    let source_files = crate::solc::discover_source_files(config);
    let mut reverse_edges: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();

    for importer in &source_files {
        let Ok(bytes) = std::fs::read(importer) else {
            continue;
        };
        for imp in links::ts_find_imports(&bytes) {
            let Some(imported_abs) =
                resolve_import_spec_to_abs(&config.root, importer, &imp.path, remappings)
            else {
                continue;
            };
            if !imported_abs.starts_with(&config.root) {
                continue;
            }
            reverse_edges
                .entry(imported_abs)
                .or_default()
                .insert(importer.clone());
        }
    }

    let mut affected: HashSet<PathBuf> = HashSet::new();
    let mut queue: std::collections::VecDeque<PathBuf> = std::collections::VecDeque::new();

    for path in changed_abs {
        if !path.starts_with(&config.root) {
            continue;
        }
        let normalized = lexical_normalize(path);
        if affected.insert(normalized.clone()) {
            queue.push_back(normalized);
        }
    }

    while let Some(current) = queue.pop_front() {
        if let Some(importers) = reverse_edges.get(&current) {
            for importer in importers {
                if affected.insert(importer.clone()) {
                    queue.push_back(importer.clone());
                }
            }
        }
    }

    // Keep only files that currently exist and are source files known to the project.
    let source_set: HashSet<PathBuf> = source_files.into_iter().collect();
    affected
        .into_iter()
        .filter(|p| source_set.contains(p) && p.is_file())
        .collect()
}

fn src_file_id(src: &str) -> Option<&str> {
    src.rsplit(':').next().filter(|id| !id.is_empty())
}

fn remap_src_file_id(
    src: &str,
    id_remap: &HashMap<crate::types::SolcFileId, crate::types::SolcFileId>,
) -> String {
    let Some(old_id) = src_file_id(src) else {
        return src.to_string();
    };
    let Some(new_id) = id_remap.get(old_id) else {
        return src.to_string();
    };
    if new_id.as_str() == old_id {
        return src.to_string();
    }
    let prefix_len = src.len().saturating_sub(old_id.len());
    format!("{}{}", &src[..prefix_len], new_id)
}

fn remap_node_info_file_ids(
    info: &mut goto::NodeInfo,
    id_remap: &HashMap<crate::types::SolcFileId, crate::types::SolcFileId>,
) {
    info.src = crate::types::SrcLocation::new(remap_src_file_id(info.src.as_str(), id_remap));
    if let Some(loc) = info.name_location.as_mut() {
        *loc = remap_src_file_id(loc, id_remap);
    }
    for loc in &mut info.name_locations {
        *loc = remap_src_file_id(loc, id_remap);
    }
    if let Some(loc) = info.member_location.as_mut() {
        *loc = remap_src_file_id(loc, id_remap);
    }
}

fn doc_key_path(key: &hover::DocKey) -> Option<&str> {
    match key {
        hover::DocKey::Contract(k) | hover::DocKey::StateVar(k) | hover::DocKey::Method(k) => {
            k.split_once(':').map(|(path, _)| path)
        }
        hover::DocKey::Func(_) | hover::DocKey::Event(_) => None,
    }
}

fn merge_scoped_cached_build(
    existing: &mut goto::CachedBuild,
    mut scoped: goto::CachedBuild,
) -> Result<usize, String> {
    let affected_paths: HashSet<String> = scoped.nodes.keys().map(|p| p.to_string()).collect();
    if affected_paths.is_empty() {
        return Ok(0);
    }
    let affected_abs_paths: HashSet<crate::types::AbsPath> =
        scoped.path_to_abs.values().cloned().collect();

    // Safety guard: reject scoped merge when declaration IDs collide with
    // unaffected files in the existing cache.
    for scoped_id in scoped.decl_index.keys() {
        if existing.decl_index.contains_key(scoped_id)
            && let Some(path) = existing.node_id_to_source_path.get(scoped_id)
            && !affected_abs_paths.contains(path)
        {
            return Err(format!(
                "decl id collision for id={} in unaffected path {}",
                scoped_id, path
            ));
        }
    }

    // Remap scoped local source IDs to existing/canonical IDs.
    use crate::types::SolcFileId;
    let mut path_to_existing_id: HashMap<crate::types::RelPath, SolcFileId> = HashMap::new();
    for (id, path) in &existing.id_to_path_map {
        path_to_existing_id
            .entry(crate::types::RelPath::new(path.clone()))
            .or_insert_with(|| id.clone());
    }
    let mut used_ids: HashSet<SolcFileId> = existing.id_to_path_map.keys().cloned().collect();
    let mut next_id = used_ids
        .iter()
        .filter_map(|k| k.as_str().parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    let mut id_remap: HashMap<SolcFileId, SolcFileId> = HashMap::new();
    for (scoped_id, path) in &scoped.id_to_path_map {
        let scoped_path = crate::types::RelPath::new(path.clone());
        let canonical = if let Some(id) = path_to_existing_id.get(&scoped_path) {
            id.clone()
        } else {
            let id = loop {
                let candidate = SolcFileId::new(next_id.to_string());
                next_id = next_id.saturating_add(1);
                if used_ids.insert(candidate.clone()) {
                    break candidate;
                }
            };
            path_to_existing_id.insert(scoped_path, id.clone());
            id
        };
        id_remap.insert(scoped_id.clone(), canonical);
    }

    for file_nodes in scoped.nodes.values_mut() {
        for info in file_nodes.values_mut() {
            remap_node_info_file_ids(info, &id_remap);
        }
    }
    let scoped_external_refs: goto::ExternalRefs = scoped
        .external_refs
        .into_iter()
        .map(|(src, decl_id)| {
            (
                crate::types::SrcLocation::new(remap_src_file_id(src.as_str(), &id_remap)),
                decl_id,
            )
        })
        .collect();

    let old_id_to_path = existing.id_to_path_map.clone();
    existing.external_refs.retain(|src, _| {
        src_file_id(src.as_str())
            .and_then(|fid| old_id_to_path.get(fid))
            .map(|path| !affected_paths.contains(path))
            .unwrap_or(true)
    });
    existing
        .nodes
        .retain(|path, _| !affected_paths.contains(path.as_str()));
    existing
        .path_to_abs
        .retain(|path, _| !affected_paths.contains(path.as_str()));
    existing
        .id_to_path_map
        .retain(|_, path| !affected_paths.contains(path));

    existing
        .node_id_to_source_path
        .retain(|_, path| !affected_abs_paths.contains(path));
    existing
        .decl_index
        .retain(|id, _| match existing.node_id_to_source_path.get(id) {
            Some(path) => !affected_abs_paths.contains(path),
            None => true,
        });
    existing
        .hint_index
        .retain(|abs_path, _| !affected_abs_paths.contains(abs_path));
    existing.gas_index.retain(|k, _| {
        k.path()
            .map(|path| !affected_paths.contains(path))
            .unwrap_or(true)
    });
    existing.doc_index.retain(|k, _| {
        doc_key_path(k)
            .map(|p| !affected_paths.contains(p))
            .unwrap_or(true)
    });

    existing.nodes.extend(scoped.nodes);
    existing.path_to_abs.extend(scoped.path_to_abs);
    existing.external_refs.extend(scoped_external_refs);
    for (old_id, path) in scoped.id_to_path_map {
        let canonical = id_remap.get(&old_id).cloned().unwrap_or(old_id);
        existing.id_to_path_map.insert(canonical, path);
    }
    existing.decl_index.extend(scoped.decl_index);
    existing
        .node_id_to_source_path
        .extend(scoped.node_id_to_source_path);
    existing.gas_index.extend(scoped.gas_index);
    existing.hint_index.extend(scoped.hint_index);
    existing.doc_index.extend(scoped.doc_index);

    Ok(affected_paths.len())
}

/// Core per-save work: compile, diagnostics, cache upsert.
///
/// Called from the per-URI worker loop (see `did_save_workers`).  Because the
/// worker serialises calls for the same URI, this function never runs
/// concurrently for the same document.
async fn run_did_save(this: ForgeLsp, params: DidSaveTextDocumentParams) {
    this.client
        .log_message(MessageType::INFO, "file saved")
        .await;

    let mut text_content = if let Some(text) = params.text {
        text
    } else {
        // Prefer text_cache (reflects unsaved changes), fall back to disk
        let cached = {
            let text_cache = this.text_cache.read().await;
            text_cache
                .get(params.text_document.uri.as_str())
                .map(|(_, content)| content.clone())
        };
        if let Some(content) = cached {
            content
        } else {
            match std::fs::read_to_string(params.text_document.uri.path()) {
                Ok(content) => content,
                Err(e) => {
                    this.client
                        .log_message(
                            MessageType::ERROR,
                            format!("Failed to read file on save: {e}"),
                        )
                        .await;
                    return;
                }
            }
        }
    };

    // Recovery path for create-file races:
    // if a newly-created file is still whitespace-only at first save,
    // regenerate scaffold and apply it to the open buffer.
    let uri_str = params.text_document.uri.to_string();
    let template_on_create = this
        .settings
        .read()
        .await
        .file_operations
        .template_on_create;
    let needs_recover_scaffold = {
        let pending = this.pending_create_scaffold.read().await;
        template_on_create
            && pending.contains(&uri_str)
            && !text_content.chars().any(|ch| !ch.is_whitespace())
    };
    if needs_recover_scaffold {
        let solc_version = this.foundry_config.read().await.solc_version.clone();
        if let Some(scaffold) =
            file_operations::generate_scaffold(&params.text_document.uri, solc_version.as_deref())
        {
            let end = utils::byte_offset_to_position(&text_content, text_content.len());
            let edit = WorkspaceEdit {
                changes: Some(HashMap::from([(
                    params.text_document.uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: Position::default(),
                            end,
                        },
                        new_text: scaffold.clone(),
                    }],
                )])),
                document_changes: None,
                change_annotations: None,
            };
            if this
                .client
                .apply_edit(edit)
                .await
                .as_ref()
                .is_ok_and(|r| r.applied)
            {
                text_content = scaffold.clone();
                let version = this
                    .text_cache
                    .read()
                    .await
                    .get(params.text_document.uri.as_str())
                    .map(|(v, _)| *v)
                    .unwrap_or_default();
                this.text_cache
                    .write()
                    .await
                    .insert(uri_str.clone().into(), (version, scaffold));
                this.pending_create_scaffold.write().await.remove(&uri_str);
                this.client
                    .log_message(
                        MessageType::INFO,
                        format!("didSave: recovered scaffold for {}", uri_str),
                    )
                    .await;
            }
        }
    }

    let version = this
        .text_cache
        .read()
        .await
        .get(params.text_document.uri.as_str())
        .map(|(version, _)| *version)
        .unwrap_or_default();

    let saved_uri = params.text_document.uri.clone();
    if let Ok(saved_file_path) = saved_uri.to_file_path() {
        let saved_abs = saved_file_path.to_string_lossy().to_string();
        this.project_cache_changed_files
            .write()
            .await
            .insert(saved_abs.clone());
        this.project_cache_upsert_files
            .write()
            .await
            .insert(saved_abs);
    }
    this.on_change(TextDocumentItem {
        uri: saved_uri.clone(),
        text: text_content,
        version,
        language_id: "".to_string(),
    })
    .await;

    let settings_snapshot = this.settings.read().await.clone();

    // Fast-path incremental v2 cache upsert on save (debounced single-flight):
    // serialize the authoritative root-key CachedBuild to disk, writing
    // shards only for recently changed files.  Global metadata (path_to_abs,
    // id_to_path_map, external_refs) comes from the merged in-memory build
    // which has correct globally-remapped file IDs.
    if this.use_solc
        && settings_snapshot.project_index.full_project_scan
        && matches!(
            settings_snapshot.project_index.cache_mode,
            crate::config::ProjectIndexCacheMode::V2 | crate::config::ProjectIndexCacheMode::Auto
        )
    {
        if start_or_mark_project_cache_upsert_pending(
            &this.project_cache_upsert_pending,
            &this.project_cache_upsert_running,
        ) {
            let upsert_files = this.project_cache_upsert_files.clone();
            let ast_cache = this.ast_cache.clone();
            let client = this.client.clone();
            let running_flag = this.project_cache_upsert_running.clone();
            let pending_flag = this.project_cache_upsert_pending.clone();
            let foundry_config = this.foundry_config.read().await.clone();
            let root_key = this.project_cache_key().await;

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(350)).await;

                    if !take_project_cache_upsert_pending(&pending_flag) {
                        if stop_project_cache_upsert_worker_or_reclaim(&pending_flag, &running_flag)
                        {
                            continue;
                        }
                        break;
                    }

                    let changed_paths: Vec<String> = {
                        let mut paths = upsert_files.write().await;
                        paths.drain().collect()
                    };
                    if changed_paths.is_empty() {
                        continue;
                    }

                    // Read the authoritative merged root-key build from
                    // ast_cache.  This is the CachedBuild that
                    // merge_scoped_cached_build has already remapped with
                    // correct global file IDs.
                    let Some(ref rk) = root_key else {
                        continue;
                    };
                    let Some(root_build) = ast_cache.read().await.get(rk).cloned() else {
                        continue;
                    };

                    let cfg = foundry_config.clone();
                    let build = (*root_build).clone();
                    let changed = changed_paths.clone();

                    let res = tokio::task::spawn_blocking(move || {
                        crate::project_cache::upsert_reference_cache_v2_with_report(
                            &cfg, &build, &changed,
                        )
                    })
                    .await;

                    match res {
                        Ok(Ok(report)) => {
                            client
                                .log_message(
                                    MessageType::INFO,
                                    format!(
                                        "project cache v2 upsert (debounced): touched_files={}, duration={}ms",
                                        report.file_count_hashed, report.duration_ms
                                    ),
                                )
                                .await;
                        }
                        Ok(Err(e)) => {
                            client
                                .log_message(
                                    MessageType::WARNING,
                                    format!("project cache v2 upsert: {e}"),
                                )
                                .await;
                        }
                        Err(e) => {
                            client
                                .log_message(
                                    MessageType::WARNING,
                                    format!("project cache v2 upsert task failed: {e}"),
                                )
                                .await;
                        }
                    }
                }
            });
        }
    }

    // If workspace file-ops changed project structure, schedule a
    // debounced latest-wins sync of on-disk reference cache.
    if this.use_solc
        && settings_snapshot.project_index.full_project_scan
        && this.project_cache_dirty.load(Ordering::Acquire)
    {
        if start_or_mark_project_cache_sync_pending(
            &this.project_cache_sync_pending,
            &this.project_cache_sync_running,
        ) {
            let foundry_config = this.foundry_config.read().await.clone();
            let root_key = this.project_cache_key().await;
            let ast_cache = this.ast_cache.clone();
            let text_cache = this.text_cache.clone();
            let client = this.client.clone();
            let dirty_flag = this.project_cache_dirty.clone();
            let running_flag = this.project_cache_sync_running.clone();
            let pending_flag = this.project_cache_sync_pending.clone();
            let changed_files = this.project_cache_changed_files.clone();
            let aggressive_scoped = settings_snapshot.project_index.incremental_edit_reindex;
            let force_full_rebuild_flag = this.project_cache_force_full_rebuild.clone();

            tokio::spawn(async move {
                loop {
                    // Debounce save bursts into one trailing sync.
                    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

                    if !take_project_cache_sync_pending(&pending_flag) {
                        if stop_project_cache_sync_worker_or_reclaim(&pending_flag, &running_flag) {
                            continue;
                        }
                        break;
                    }

                    if !try_claim_project_cache_dirty(&dirty_flag) {
                        continue;
                    }

                    let Some(cache_key) = &root_key else {
                        dirty_flag.store(true, Ordering::Release);
                        continue;
                    };
                    if !foundry_config.root.is_dir() {
                        dirty_flag.store(true, Ordering::Release);
                        client
                            .log_message(
                                MessageType::WARNING,
                                format!(
                                    "didSave cache sync: invalid project root {}, deferring",
                                    foundry_config.root.display()
                                ),
                            )
                            .await;
                        continue;
                    }

                    let mut scoped_ok = false;

                    // If solidity.reindex was called while this worker was
                    // already running, bypass the incremental scoped path and
                    // do a full rebuild instead.
                    let force_full = force_full_rebuild_flag.swap(false, Ordering::AcqRel);

                    if aggressive_scoped && !force_full {
                        let changed_abs: Vec<PathBuf> = {
                            let mut changed = changed_files.write().await;
                            let drained =
                                changed.drain().map(PathBuf::from).collect::<Vec<PathBuf>>();
                            drained
                        };
                        if !changed_abs.is_empty() {
                            let remappings = crate::solc::resolve_remappings(&foundry_config).await;
                            let cfg_for_plan = foundry_config.clone();
                            let changed_for_plan = changed_abs.clone();
                            let remappings_for_plan = remappings.clone();
                            let plan_res = tokio::task::spawn_blocking(move || {
                                compute_reverse_import_closure(
                                    &cfg_for_plan,
                                    &changed_for_plan,
                                    &remappings_for_plan,
                                )
                            })
                            .await;

                            let affected_files = match plan_res {
                                Ok(set) => set.into_iter().collect::<Vec<PathBuf>>(),
                                Err(_) => Vec::new(),
                            };
                            if !affected_files.is_empty() {
                                client
                                    .log_message(
                                        MessageType::INFO,
                                        format!(
                                            "didSave cache sync: aggressive scoped reindex (affected={})",
                                            affected_files.len(),
                                        ),
                                    )
                                    .await;

                                let text_cache_snapshot = text_cache.read().await.clone();
                                match crate::solc::solc_project_index_scoped(
                                    &foundry_config,
                                    Some(&client),
                                    Some(&text_cache_snapshot),
                                    &affected_files,
                                )
                                .await
                                {
                                    Ok(ast_data) => {
                                        let scoped_build =
                                            Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                                        let source_count = scoped_build.nodes.len();
                                        enum ScopedApply {
                                            Merged { affected_count: usize },
                                            Stored,
                                            Failed(String),
                                        }
                                        let apply_outcome = {
                                            let mut cache = ast_cache.write().await;
                                            if let Some(existing) = cache.get(cache_key).cloned() {
                                                let mut merged = (*existing).clone();
                                                match merge_scoped_cached_build(
                                                    &mut merged,
                                                    (*scoped_build).clone(),
                                                ) {
                                                    Ok(affected_count) => {
                                                        cache.insert(
                                                            cache_key.clone().into(),
                                                            Arc::new(merged),
                                                        );
                                                        ScopedApply::Merged { affected_count }
                                                    }
                                                    Err(e) => ScopedApply::Failed(e),
                                                }
                                            } else {
                                                cache
                                                    .insert(cache_key.clone().into(), scoped_build);
                                                ScopedApply::Stored
                                            }
                                        };

                                        match apply_outcome {
                                            ScopedApply::Merged { affected_count } => {
                                                client
                                                    .log_message(
                                                        MessageType::INFO,
                                                        format!(
                                                            "didSave cache sync: scoped merge applied (scoped_sources={}, affected_paths={})",
                                                            source_count, affected_count
                                                        ),
                                                    )
                                                    .await;
                                                scoped_ok = true;
                                            }
                                            ScopedApply::Stored => {
                                                client
                                                    .log_message(
                                                        MessageType::INFO,
                                                        format!(
                                                            "didSave cache sync: scoped cache stored (scoped_sources={})",
                                                            source_count
                                                        ),
                                                    )
                                                    .await;
                                                scoped_ok = true;
                                            }
                                            ScopedApply::Failed(e) => {
                                                client
                                                .log_message(
                                                    MessageType::WARNING,
                                                    format!(
                                                        "didSave cache sync: scoped merge rejected, will retry scoped on next save: {e}"
                                                    ),
                                                )
                                                .await;
                                                dirty_flag.store(true, Ordering::Release);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        client
                                            .log_message(
                                                MessageType::WARNING,
                                                format!(
                                                    "didSave cache sync: scoped reindex failed, will retry scoped on next save: {e}"
                                                ),
                                            )
                                            .await;
                                        dirty_flag.store(true, Ordering::Release);
                                    }
                                }
                            } else {
                                client
                                    .log_message(
                                        MessageType::INFO,
                                        "didSave cache sync: no affected files from scoped planner",
                                    )
                                    .await;
                            }
                        }
                    }

                    if scoped_ok {
                        continue;
                    }
                    if aggressive_scoped {
                        continue;
                    }

                    client
                        .log_message(
                            MessageType::INFO,
                            "didSave cache sync: rebuilding project index from disk",
                        )
                        .await;

                    match crate::solc::solc_project_index(&foundry_config, Some(&client), None)
                        .await
                    {
                        Ok(ast_data) => {
                            let mut new_build = crate::goto::CachedBuild::new(ast_data, 0);
                            if let Some(prev) = ast_cache.read().await.get(cache_key) {
                                new_build.merge_missing_from(prev);
                            }
                            let source_count = new_build.nodes.len();
                            let cached_build = Arc::new(new_build);
                            let build_for_save = (*cached_build).clone();
                            ast_cache
                                .write()
                                .await
                                .insert(cache_key.clone().into(), cached_build);

                            let cfg_for_save = foundry_config.clone();
                            let save_res = tokio::task::spawn_blocking(move || {
                                crate::project_cache::save_reference_cache_with_report(
                                    &cfg_for_save,
                                    &build_for_save,
                                    None,
                                )
                            })
                            .await;

                            match save_res {
                                Ok(Ok(report)) => {
                                    changed_files.write().await.clear();
                                    client
                                        .log_message(
                                            MessageType::INFO,
                                            format!(
                                                "didSave cache sync: persisted cache (sources={}, hashed_files={}, duration={}ms)",
                                                source_count, report.file_count_hashed, report.duration_ms
                                            ),
                                        )
                                        .await;
                                }
                                Ok(Err(e)) => {
                                    dirty_flag.store(true, Ordering::Release);
                                    client
                                        .log_message(
                                            MessageType::WARNING,
                                            format!(
                                                "didSave cache sync: persist failed, will retry: {e}"
                                            ),
                                        )
                                        .await;
                                }
                                Err(e) => {
                                    dirty_flag.store(true, Ordering::Release);
                                    client
                                        .log_message(
                                            MessageType::WARNING,
                                            format!(
                                                "didSave cache sync: save task failed, will retry: {e}"
                                            ),
                                        )
                                        .await;
                                }
                            }
                        }
                        Err(e) => {
                            dirty_flag.store(true, Ordering::Release);
                            client
                                .log_message(
                                    MessageType::WARNING,
                                    format!("didSave cache sync: re-index failed, will retry: {e}"),
                                )
                                .await;
                        }
                    }
                }
            });
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ForgeLsp {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        // Store client capabilities for use during `initialized()`.
        {
            let mut caps = self.client_capabilities.write().await;
            *caps = Some(params.capabilities.clone());
        }

        // Read editor settings from initializationOptions.
        if let Some(init_opts) = &params.initialization_options {
            let s = config::parse_settings(init_opts);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "settings: inlayHints.parameters={}, inlayHints.gasEstimates={}, lint.enabled={}, lint.severity={:?}, lint.only={:?}, lint.exclude={:?}, fileOperations.templateOnCreate={}, fileOperations.updateImportsOnRename={}, fileOperations.updateImportsOnDelete={}, projectIndex.fullProjectScan={}, projectIndex.cacheMode={:?}, projectIndex.incrementalEditReindex={}",
                        s.inlay_hints.parameters, s.inlay_hints.gas_estimates, s.lint.enabled, s.lint.severity, s.lint.only, s.lint.exclude, s.file_operations.template_on_create, s.file_operations.update_imports_on_rename, s.file_operations.update_imports_on_delete, s.project_index.full_project_scan, s.project_index.cache_mode, s.project_index.incremental_edit_reindex,
                    ),
                )
                .await;
            let mut settings = self.settings.write().await;
            *settings = s;
            self.settings_from_init
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Store root URI for project-wide file discovery.
        if let Some(uri) = params.root_uri.as_ref() {
            let mut root = self.root_uri.write().await;
            *root = Some(uri.clone());
        }

        // Load config from the workspace root's foundry.toml.
        if let Some(root_uri) = params
            .root_uri
            .as_ref()
            .and_then(|uri| uri.to_file_path().ok())
        {
            let lint_cfg = config::load_lint_config(&root_uri);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "loaded foundry.toml lint config: lint_on_build={}, ignore_patterns={}",
                        lint_cfg.lint_on_build,
                        lint_cfg.ignore_patterns.len()
                    ),
                )
                .await;
            let mut config = self.lint_config.write().await;
            *config = lint_cfg;

            let foundry_cfg = config::load_foundry_config(&root_uri);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "loaded foundry.toml project config: solc_version={:?}, remappings={}",
                        foundry_cfg.solc_version,
                        foundry_cfg.remappings.len()
                    ),
                )
                .await;
            if foundry_cfg.via_ir {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        "via_ir is enabled in foundry.toml — gas estimate inlay hints are disabled to avoid slow compilation",
                    )
                    .await;
            }
            let mut fc = self.foundry_config.write().await;
            *fc = foundry_cfg;
        }

        // Negotiate position encoding with the client (once, for the session).
        let client_encodings = params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_deref());
        let encoding = utils::PositionEncoding::negotiate(client_encodings);
        utils::set_encoding(encoding);

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Solidity Language Server".to_string(),
                version: Some(env!("LONG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                position_encoding: Some(encoding.into()),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "\"".to_string(),
                        "'".to_string(),
                        "/".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec![
                        "(".to_string(),
                        ",".to_string(),
                        "[".to_string(),
                    ]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                })),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                code_lens_provider: None,
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
                    InlayHintOptions {
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                ))),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens::legend(),
                            full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            range: Some(true),
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: None,
                            },
                        },
                    ),
                ),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        will_save: Some(true),
                        will_save_wait_until: None,
                        open_close: Some(true),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        change: Some(TextDocumentSyncKind::FULL),
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: None,
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        will_rename: Some(FileOperationRegistrationOptions {
                            filters: vec![
                                // Match .sol files
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**/*.sol".to_string(),
                                        matches: Some(FileOperationPatternKind::File),
                                        options: None,
                                    },
                                },
                                // Match folders (moving a directory moves all .sol files within)
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**".to_string(),
                                        matches: Some(FileOperationPatternKind::Folder),
                                        options: None,
                                    },
                                },
                            ],
                        }),
                        did_rename: Some(FileOperationRegistrationOptions {
                            filters: vec![
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**/*.sol".to_string(),
                                        matches: Some(FileOperationPatternKind::File),
                                        options: None,
                                    },
                                },
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**".to_string(),
                                        matches: Some(FileOperationPatternKind::Folder),
                                        options: None,
                                    },
                                },
                            ],
                        }),
                        will_delete: Some(FileOperationRegistrationOptions {
                            filters: vec![
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**/*.sol".to_string(),
                                        matches: Some(FileOperationPatternKind::File),
                                        options: None,
                                    },
                                },
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**".to_string(),
                                        matches: Some(FileOperationPatternKind::Folder),
                                        options: None,
                                    },
                                },
                            ],
                        }),
                        did_delete: Some(FileOperationRegistrationOptions {
                            filters: vec![
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**/*.sol".to_string(),
                                        matches: Some(FileOperationPatternKind::File),
                                        options: None,
                                    },
                                },
                                FileOperationFilter {
                                    scheme: Some("file".to_string()),
                                    pattern: FileOperationPattern {
                                        glob: "**".to_string(),
                                        matches: Some(FileOperationPatternKind::Folder),
                                        options: None,
                                    },
                                },
                            ],
                        }),
                        will_create: Some(FileOperationRegistrationOptions {
                            filters: vec![FileOperationFilter {
                                scheme: Some("file".to_string()),
                                pattern: FileOperationPattern {
                                    glob: "**/*.sol".to_string(),
                                    matches: Some(FileOperationPatternKind::File),
                                    options: None,
                                },
                            }],
                        }),
                        did_create: Some(FileOperationRegistrationOptions {
                            filters: vec![FileOperationFilter {
                                scheme: Some("file".to_string()),
                                pattern: FileOperationPattern {
                                    glob: "**/*.sol".to_string(),
                                    matches: Some(FileOperationPatternKind::File),
                                    options: None,
                                },
                            }],
                        }),
                        ..Default::default()
                    }),
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "solidity.clearCache".to_string(),
                        "solidity.reindex".to_string(),
                    ],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "lsp server initialized.")
            .await;

        // Dynamically register a file watcher for foundry.toml changes.
        let supports_dynamic = self
            .client_capabilities
            .read()
            .await
            .as_ref()
            .and_then(|caps| caps.workspace.as_ref())
            .and_then(|ws| ws.did_change_watched_files.as_ref())
            .and_then(|dcwf| dcwf.dynamic_registration)
            .unwrap_or(false);

        if supports_dynamic {
            let registration = Registration {
                id: "foundry-toml-watcher".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(
                    serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String("**/foundry.toml".to_string()),
                                kind: Some(WatchKind::all()),
                            },
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String("**/remappings.txt".to_string()),
                                kind: Some(WatchKind::all()),
                            },
                        ],
                    })
                    .unwrap(),
                ),
            };

            if let Err(e) = self.client.register_capability(vec![registration]).await {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("failed to register foundry.toml watcher: {e}"),
                    )
                    .await;
            } else {
                self.client
                    .log_message(MessageType::INFO, "registered foundry.toml file watcher")
                    .await;
            }
        }

        // Pull settings from the client via workspace/configuration.
        // Neovim (and other editors) expose user settings through this
        // request rather than initializationOptions, so we need to pull
        // them explicitly if initializationOptions was absent.
        if !self
            .settings_from_init
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let supports_config = self
                .client_capabilities
                .read()
                .await
                .as_ref()
                .and_then(|caps| caps.workspace.as_ref())
                .and_then(|ws| ws.configuration)
                .unwrap_or(false);

            if supports_config {
                match self
                    .client
                    .configuration(vec![ConfigurationItem {
                        scope_uri: None,
                        section: Some("solidity-language-server".to_string()),
                    }])
                    .await
                {
                    Ok(values) => {
                        if let Some(val) = values.into_iter().next() {
                            if !val.is_null() {
                                let s = config::parse_settings(&val);
                                self.client
                                    .log_message(
                                        MessageType::INFO,
                                        format!(
                                            "settings (workspace/configuration): lint.enabled={}, lint.exclude={:?}, projectIndex.fullProjectScan={}, projectIndex.cacheMode={:?}",
                                            s.lint.enabled, s.lint.exclude, s.project_index.full_project_scan, s.project_index.cache_mode,
                                        ),
                                    )
                                    .await;
                                let mut settings = self.settings.write().await;
                                *settings = s;
                            }
                        }
                    }
                    Err(e) => {
                        self.client
                            .log_message(
                                MessageType::WARNING,
                                format!("workspace/configuration request failed: {e}"),
                            )
                            .await;
                    }
                }
            }
        }

        // Eagerly build the project index on startup so cross-file features
        // (willRenameFiles, references, goto) work immediately — even before
        // the user opens any .sol file.
        if self.use_solc && self.settings.read().await.project_index.full_project_scan {
            let cache_mode = self.settings.read().await.project_index.cache_mode.clone();
            self.project_indexed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let foundry_config = self.foundry_config.read().await.clone();
            let cache_key = self.project_cache_key().await;
            let ast_cache = self.ast_cache.clone();
            let client = self.client.clone();
            let sub_caches_arc = self.sub_caches.clone();
            let sub_caches_loading_flag = self.sub_caches_loading.clone();

            tokio::spawn(async move {
                let Some(cache_key) = cache_key else {
                    return;
                };
                if !foundry_config.root.is_dir() {
                    client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "project index: {} not found, skipping eager index",
                                foundry_config.root.display(),
                            ),
                        )
                        .await;
                    return;
                }

                let token = NumberOrString::String("solidity/projectIndex".to_string());
                let _ = client
                    .send_request::<request::WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                        token: token.clone(),
                    })
                    .await;

                client
                    .send_notification::<notification::Progress>(ProgressParams {
                        token: token.clone(),
                        value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                            WorkDoneProgressBegin {
                                title: "Indexing project".to_string(),
                                message: Some("Discovering source files...".to_string()),
                                cancellable: Some(false),
                                percentage: None,
                            },
                        )),
                    })
                    .await;

                // Try persisted reference index first (fast warm start).
                let cfg_for_load = foundry_config.clone();
                let cache_mode_for_load = cache_mode.clone();
                let load_res = tokio::task::spawn_blocking(move || {
                    crate::project_cache::load_reference_cache_with_report(
                        &cfg_for_load,
                        cache_mode_for_load,
                        true,
                    )
                })
                .await;
                match load_res {
                    Ok(report) => {
                        if let Some(cached_build) = report.build {
                            let source_count = cached_build.nodes.len();
                            ast_cache
                                .write()
                                .await
                                .insert(cache_key.clone().into(), Arc::new(cached_build));
                            client
                                .log_message(
                                    MessageType::INFO,
                                    format!(
                                         "project index (eager): cache load hit (sources={}, reused_files={}/{}, complete={}, duration={}ms)",
                                        source_count,
                                        report.file_count_reused,
                                        report.file_count_hashed,
                                        report.complete,
                                        report.duration_ms
                                    ),
                                )
                                .await;
                            if report.complete {
                                // Pre-load lib sub-caches so references
                                // include lib test files on the first call.
                                spawn_load_lib_sub_caches_task(
                                    foundry_config.clone(),
                                    sub_caches_arc.clone(),
                                    sub_caches_loading_flag.clone(),
                                    client.clone(),
                                );
                                client
                                    .send_notification::<notification::Progress>(ProgressParams {
                                        token: token.clone(),
                                        value: ProgressParamsValue::WorkDone(
                                            WorkDoneProgress::End(WorkDoneProgressEnd {
                                                message: Some(format!(
                                                    "Loaded {} source files from cache",
                                                    source_count
                                                )),
                                            }),
                                        ),
                                    })
                                    .await;
                                return;
                            }
                        }

                        client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "project index (eager): cache load miss/partial (reason={}, reused_files={}/{}, duration={}ms)",
                                    report
                                        .miss_reason
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    report.file_count_reused,
                                    report.file_count_hashed,
                                    report.duration_ms
                                ),
                            )
                            .await;
                    }
                    Err(e) => {
                        client
                            .log_message(
                                MessageType::WARNING,
                                format!("project index (eager): cache load task failed: {e}"),
                            )
                            .await;
                    }
                }

                match crate::solc::solc_project_index(&foundry_config, Some(&client), None).await {
                    Ok(ast_data) => {
                        let mut new_build = crate::goto::CachedBuild::new(ast_data, 0);
                        if let Some(prev) = ast_cache.read().await.get(&cache_key) {
                            new_build.merge_missing_from(prev);
                        }
                        let source_count = new_build.nodes.len();
                        let cached_build = Arc::new(new_build);
                        let build_for_save = (*cached_build).clone();
                        ast_cache
                            .write()
                            .await
                            .insert(cache_key.clone().into(), cached_build);
                        client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "project index (eager): cached {} source files",
                                    source_count
                                ),
                            )
                            .await;

                        // Pre-load lib sub-caches after full build too.
                        spawn_load_lib_sub_caches_task(
                            foundry_config.clone(),
                            sub_caches_arc.clone(),
                            sub_caches_loading_flag.clone(),
                            client.clone(),
                        );

                        let cfg_for_save = foundry_config.clone();
                        let client_for_save = client.clone();
                        tokio::spawn(async move {
                            let res = tokio::task::spawn_blocking(move || {
                                crate::project_cache::save_reference_cache_with_report(
                                    &cfg_for_save,
                                    &build_for_save,
                                    None,
                                )
                            })
                            .await;
                            match res {
                                Ok(Ok(report)) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::INFO,
                                            format!(
                                                "project index (eager): cache save complete (hashed_files={}, duration={}ms)",
                                                report.file_count_hashed, report.duration_ms
                                            ),
                                        )
                                        .await;
                                }
                                Ok(Err(e)) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::WARNING,
                                            format!(
                                                "project index (eager): failed to persist cache: {e}"
                                            ),
                                        )
                                        .await;
                                }
                                Err(e) => {
                                    client_for_save
                                        .log_message(
                                            MessageType::WARNING,
                                            format!(
                                                "project index (eager): cache save task failed: {e}"
                                            ),
                                        )
                                        .await;
                                }
                            }
                        });

                        client
                            .send_notification::<notification::Progress>(ProgressParams {
                                token: token.clone(),
                                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                                    WorkDoneProgressEnd {
                                        message: Some(format!(
                                            "Indexed {} source files",
                                            source_count
                                        )),
                                    },
                                )),
                            })
                            .await;
                    }
                    Err(e) => {
                        client
                            .log_message(
                                MessageType::WARNING,
                                format!("project index (eager): failed: {e}"),
                            )
                            .await;

                        client
                            .send_notification::<notification::Progress>(ProgressParams {
                                token,
                                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                                    WorkDoneProgressEnd {
                                        message: Some(format!("Index failed: {e}")),
                                    },
                                )),
                            })
                            .await;
                    }
                }
            });
        }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            // ----------------------------------------------------------------
            // solidity.clearCache
            //
            // Deletes the entire `.solidity-language-server/` directory on disk
            // and wipes the in-memory AST cache for the current project root.
            // The next save or file-open will trigger a clean rebuild from
            // scratch.
            //
            // Usage (nvim):
            //   vim.lsp.buf.execute_command({ command = "solidity.clearCache" })
            // ----------------------------------------------------------------
            "solidity.clearCache" => {
                let root = self.foundry_config.read().await.root.clone();
                let cache_dir = crate::project_cache::cache_dir(&root);

                let disk_result = if cache_dir.exists() {
                    std::fs::remove_dir_all(&cache_dir).map_err(|e| format!("{e}"))
                } else {
                    Ok(())
                };

                if let Some(root_key) = self.project_cache_key().await {
                    self.ast_cache.write().await.remove(&root_key);
                }

                match disk_result {
                    Ok(()) => {
                        self.client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "solidity.clearCache: removed {} and cleared in-memory cache",
                                    cache_dir.display()
                                ),
                            )
                            .await;
                        Ok(Some(serde_json::json!({ "success": true })))
                    }
                    Err(e) => {
                        self.client
                            .log_message(
                                MessageType::ERROR,
                                format!("solidity.clearCache: failed to remove cache dir: {e}"),
                            )
                            .await;
                        Err(tower_lsp::jsonrpc::Error {
                            code: tower_lsp::jsonrpc::ErrorCode::InternalError,
                            message: std::borrow::Cow::Owned(e),
                            data: None,
                        })
                    }
                }
            }

            // ----------------------------------------------------------------
            // solidity.reindex
            //
            // Evicts the in-memory AST cache entry for the current project root
            // and sets the dirty flag so the background cache-sync worker
            // triggers a fresh project index build. The on-disk cache is left
            // intact so the warm-load on reindex will be fast.
            //
            // Usage (nvim):
            //   vim.lsp.buf.execute_command({ command = "solidity.reindex" })
            // ----------------------------------------------------------------
            "solidity.reindex" => {
                if let Some(root_key) = self.project_cache_key().await {
                    self.ast_cache.write().await.remove(&root_key);
                }
                self.project_cache_dirty
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                // Signal any already-running cache-sync worker (spawned from
                // didSave) that it must skip the incremental scoped path and
                // perform a full rebuild on its next iteration.
                self.project_cache_force_full_rebuild
                    .store(true, std::sync::atomic::Ordering::Release);

                // Wake the background cache-sync worker directly. Setting the
                // dirty flag alone is not enough — the worker is only started
                // from the didSave path, so without an explicit spawn here the
                // reindex would be silently deferred until the next file save.
                if start_or_mark_project_cache_sync_pending(
                    &self.project_cache_sync_pending,
                    &self.project_cache_sync_running,
                ) {
                    let foundry_config = self.foundry_config.read().await.clone();
                    let root_key = self.project_cache_key().await;
                    let ast_cache = self.ast_cache.clone();
                    let client = self.client.clone();
                    let dirty_flag = self.project_cache_dirty.clone();
                    let running_flag = self.project_cache_sync_running.clone();
                    let pending_flag = self.project_cache_sync_pending.clone();
                    let changed_files = self.project_cache_changed_files.clone();
                    let force_full_rebuild_flag = self.project_cache_force_full_rebuild.clone();

                    tokio::spawn(async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(700)).await;

                            if !take_project_cache_sync_pending(&pending_flag) {
                                if stop_project_cache_sync_worker_or_reclaim(
                                    &pending_flag,
                                    &running_flag,
                                ) {
                                    continue;
                                }
                                break;
                            }

                            if !try_claim_project_cache_dirty(&dirty_flag) {
                                continue;
                            }

                            let Some(cache_key) = &root_key else {
                                dirty_flag.store(true, Ordering::Release);
                                continue;
                            };
                            if !foundry_config.root.is_dir() {
                                dirty_flag.store(true, Ordering::Release);
                                client
                                    .log_message(
                                        MessageType::WARNING,
                                        format!(
                                            "solidity.reindex cache sync: invalid project root {}, deferring",
                                            foundry_config.root.display()
                                        ),
                                    )
                                    .await;
                                continue;
                            }

                            client
                                .log_message(
                                    MessageType::INFO,
                                    "solidity.reindex: rebuilding project index from disk",
                                )
                                .await;

                            match crate::solc::solc_project_index(
                                &foundry_config,
                                Some(&client),
                                None,
                            )
                            .await
                            {
                                Ok(ast_data) => {
                                    let mut new_build = crate::goto::CachedBuild::new(ast_data, 0);
                                    if let Some(prev) = ast_cache.read().await.get(cache_key) {
                                        new_build.merge_missing_from(prev);
                                    }
                                    let source_count = new_build.nodes.len();
                                    let cached_build = Arc::new(new_build);
                                    let build_for_save = (*cached_build).clone();
                                    ast_cache
                                        .write()
                                        .await
                                        .insert(cache_key.clone().into(), cached_build);

                                    let cfg_for_save = foundry_config.clone();
                                    let save_res = tokio::task::spawn_blocking(move || {
                                        crate::project_cache::save_reference_cache_with_report(
                                            &cfg_for_save,
                                            &build_for_save,
                                            None,
                                        )
                                    })
                                    .await;

                                    match save_res {
                                        Ok(Ok(report)) => {
                                            changed_files.write().await.clear();
                                            // Clear any pending force-full flag: this worker
                                            // already did a full rebuild on behalf of reindex.
                                            force_full_rebuild_flag.store(false, Ordering::Release);
                                            client
                                                .log_message(
                                                    MessageType::INFO,
                                                    format!(
                                                        "solidity.reindex: persisted cache (sources={}, hashed_files={}, duration={}ms)",
                                                        source_count, report.file_count_hashed, report.duration_ms
                                                    ),
                                                )
                                                .await;
                                        }
                                        Ok(Err(e)) => {
                                            dirty_flag.store(true, Ordering::Release);
                                            client
                                                .log_message(
                                                    MessageType::WARNING,
                                                    format!(
                                                        "solidity.reindex: persist failed, will retry: {e}"
                                                    ),
                                                )
                                                .await;
                                        }
                                        Err(e) => {
                                            dirty_flag.store(true, Ordering::Release);
                                            client
                                                .log_message(
                                                    MessageType::WARNING,
                                                    format!(
                                                        "solidity.reindex: save task failed, will retry: {e}"
                                                    ),
                                                )
                                                .await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    dirty_flag.store(true, Ordering::Release);
                                    client
                                        .log_message(
                                            MessageType::WARNING,
                                            format!(
                                                "solidity.reindex: re-index failed, will retry: {e}"
                                            ),
                                        )
                                        .await;
                                }
                            }

                            if stop_project_cache_sync_worker_or_reclaim(
                                &pending_flag,
                                &running_flag,
                            ) {
                                continue;
                            }
                            break;
                        }
                    });
                }

                self.client
                    .log_message(
                        MessageType::INFO,
                        "solidity.reindex: in-memory cache evicted, background reindex triggered",
                    )
                    .await;
                Ok(Some(serde_json::json!({ "success": true })))
            }

            _ => Err(tower_lsp::jsonrpc::Error::method_not_found()),
        }
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        self.flush_project_cache_to_disk("shutdown").await;
        self.client
            .log_message(MessageType::INFO, "lsp server shutting down.")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened")
            .await;

        let mut td = params.text_document;
        let template_on_create = self
            .settings
            .read()
            .await
            .file_operations
            .template_on_create;

        // Fallback path for clients/flows that don't emit file-operation
        // create events reliably: scaffold an empty newly-opened `.sol` file.
        let should_attempt_scaffold = template_on_create
            && td.text.chars().all(|ch| ch.is_whitespace())
            && td.uri.scheme() == "file"
            && td
                .uri
                .to_file_path()
                .ok()
                .and_then(|p| p.extension().map(|e| e == "sol"))
                .unwrap_or(false);

        if should_attempt_scaffold {
            let uri_str = td.uri.to_string();
            let create_flow_pending = {
                let pending = self.pending_create_scaffold.read().await;
                pending.contains(&uri_str)
            };
            if create_flow_pending {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "didOpen: skip scaffold for {} (didCreateFiles scaffold pending)",
                            uri_str
                        ),
                    )
                    .await;
            } else {
                let cache_has_content = {
                    let tc = self.text_cache.read().await;
                    tc.get(&uri_str)
                        .map_or(false, |(_, c)| c.chars().any(|ch| !ch.is_whitespace()))
                };

                if !cache_has_content {
                    let file_has_content = td.uri.to_file_path().ok().is_some_and(|p| {
                        std::fs::read_to_string(&p)
                            .map_or(false, |c| c.chars().any(|ch| !ch.is_whitespace()))
                    });

                    if !file_has_content {
                        let solc_version = self.foundry_config.read().await.solc_version.clone();
                        if let Some(scaffold) =
                            file_operations::generate_scaffold(&td.uri, solc_version.as_deref())
                        {
                            let end = utils::byte_offset_to_position(&td.text, td.text.len());
                            let edit = WorkspaceEdit {
                                changes: Some(HashMap::from([(
                                    td.uri.clone(),
                                    vec![TextEdit {
                                        range: Range {
                                            start: Position::default(),
                                            end,
                                        },
                                        new_text: scaffold.clone(),
                                    }],
                                )])),
                                document_changes: None,
                                change_annotations: None,
                            };
                            if self
                                .client
                                .apply_edit(edit)
                                .await
                                .as_ref()
                                .is_ok_and(|r| r.applied)
                            {
                                td.text = scaffold;
                                self.client
                                    .log_message(
                                        MessageType::INFO,
                                        format!("didOpen: scaffolded empty file {}", uri_str),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
        }

        self.on_change(td).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed")
            .await;

        // update text cache
        if let Some(change) = params.content_changes.into_iter().next() {
            let has_substantive_content = change.text.chars().any(|ch| !ch.is_whitespace());
            let mut text_cache = self.text_cache.write().await;
            text_cache.insert(
                params.text_document.uri.to_string().into(),
                (params.text_document.version, change.text),
            );
            drop(text_cache);

            if has_substantive_content {
                self.pending_create_scaffold
                    .write()
                    .await
                    .remove(params.text_document.uri.as_str());
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // did_save is a notification — return to the editor immediately.
        // We route each URI through a dedicated watch channel so that rapid
        // saves collapse: the worker always picks up the *latest* params via
        // `borrow_and_update`, avoiding stale-result races.
        let uri_key = params.text_document.uri.to_string();

        // Fast path: worker already running for this URI — just send new params.
        {
            let workers = self.did_save_workers.read().await;
            if let Some(tx) = workers.get(&uri_key) {
                // Ignore send errors — worker may have panicked; fall through to
                // the slow path below which will respawn it.
                if tx.send(Some(params.clone())).is_ok() {
                    return;
                }
            }
        }

        // Slow path: first save for this URI (or worker died) — create channel
        // and spawn the worker.
        let (tx, mut rx) = tokio::sync::watch::channel(Some(params));
        self.did_save_workers
            .write()
            .await
            .insert(uri_key.into(), tx);

        let this = self.clone();
        tokio::spawn(async move {
            loop {
                // Wait for a new value to be sent.
                if rx.changed().await.is_err() {
                    // All senders dropped — shouldn't happen while ForgeLsp is
                    // alive, but exit cleanly just in case.
                    break;
                }
                let params = match rx.borrow_and_update().clone() {
                    Some(p) => p,
                    None => continue,
                };
                run_did_save(this.clone(), params).await;
            }
        });
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "file will save reason:{:?} {}",
                    params.reason, params.text_document.uri
                ),
            )
            .await;
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<TextEdit>>> {
        self.client
            .log_message(MessageType::INFO, "formatting request")
            .await;

        let uri = params.text_document.uri;
        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file URI for formatting")
                    .await;
                return Ok(None);
            }
        };
        let path_str = match file_path.to_str() {
            Some(s) => s,
            None => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file path for formatting")
                    .await;
                return Ok(None);
            }
        };

        // Get original content
        let original_content = {
            let text_cache = self.text_cache.read().await;
            if let Some((_, content)) = text_cache.get(&uri.to_string()) {
                content.clone()
            } else {
                // Fallback to reading file
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => content,
                    Err(_) => {
                        self.client
                            .log_message(MessageType::ERROR, "Failed to read file for formatting")
                            .await;
                        return Ok(None);
                    }
                }
            }
        };

        // Get formatted content
        let formatted_content = match self.compiler.format(path_str).await {
            Ok(content) => content,
            Err(e) => {
                self.client
                    .log_message(MessageType::WARNING, format!("Formatting failed: {e}"))
                    .await;
                return Ok(None);
            }
        };

        // If changed, update text_cache with formatted content and return edit
        if original_content != formatted_content {
            let end = utils::byte_offset_to_position(&original_content, original_content.len());

            // Update text_cache immediately so goto/hover use the formatted text
            {
                let mut text_cache = self.text_cache.write().await;
                let version = text_cache
                    .get(&uri.to_string())
                    .map(|(v, _)| *v)
                    .unwrap_or(0);
                text_cache.insert(uri.to_string().into(), (version, formatted_content.clone()));
            }

            let edit = TextEdit {
                range: Range {
                    start: Position::default(),
                    end,
                },
                new_text: formatted_content,
            };
            Ok(Some(vec![edit]))
        } else {
            Ok(None)
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.flush_project_cache_to_disk("didClose").await;
        let uri = params.text_document.uri.to_string();
        self.ast_cache.write().await.remove(&uri);
        self.text_cache.write().await.remove(&uri);
        self.completion_cache.write().await.remove(&uri);
        self.client
            .log_message(MessageType::INFO, "file closed, caches cleared.")
            .await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let s = config::parse_settings(&params.settings);
        self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "settings updated: inlayHints.parameters={}, inlayHints.gasEstimates={}, lint.enabled={}, lint.severity={:?}, lint.only={:?}, lint.exclude={:?}, fileOperations.templateOnCreate={}, fileOperations.updateImportsOnRename={}, fileOperations.updateImportsOnDelete={}, projectIndex.fullProjectScan={}, projectIndex.cacheMode={:?}, projectIndex.incrementalEditReindex={}",
                    s.inlay_hints.parameters, s.inlay_hints.gas_estimates, s.lint.enabled, s.lint.severity, s.lint.only, s.lint.exclude, s.file_operations.template_on_create, s.file_operations.update_imports_on_rename, s.file_operations.update_imports_on_delete, s.project_index.full_project_scan, s.project_index.cache_mode, s.project_index.incremental_edit_reindex,
                ),
            )
            .await;
        let mut settings = self.settings.write().await;
        *settings = s;

        // Refresh inlay hints so the editor re-requests them with new settings.
        let client = self.client.clone();
        tokio::spawn(async move {
            let _ = client.inlay_hint_refresh().await;
        });
    }
    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workdspace folders changed.")
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed.")
            .await;

        // Reload configs if foundry.toml or remappings.txt changed.
        for change in &params.changes {
            let path = match change.uri.to_file_path() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let filename = path.file_name().and_then(|n| n.to_str());

            if filename == Some("foundry.toml") {
                let lint_cfg = config::load_lint_config_from_toml(&path);
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "reloaded foundry.toml lint config: lint_on_build={}, ignore_patterns={}",
                            lint_cfg.lint_on_build,
                            lint_cfg.ignore_patterns.len()
                        ),
                    )
                    .await;
                let mut lc = self.lint_config.write().await;
                *lc = lint_cfg;

                let foundry_cfg = config::load_foundry_config_from_toml(&path);
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "reloaded foundry.toml project config: solc_version={:?}, remappings={}",
                            foundry_cfg.solc_version,
                            foundry_cfg.remappings.len()
                        ),
                    )
                    .await;
                if foundry_cfg.via_ir {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            "via_ir is enabled in foundry.toml — gas estimate inlay hints are disabled to avoid slow compilation",
                        )
                        .await;
                }
                let mut fc = self.foundry_config.write().await;
                *fc = foundry_cfg;
                break;
            }

            if filename == Some("remappings.txt") {
                self.client
                    .log_message(
                        MessageType::INFO,
                        "remappings.txt changed, config may need refresh",
                    )
                    .await;
                // Remappings from remappings.txt are resolved at solc invocation time
                // via `forge remappings`, so no cached state to update here.
            }
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let trigger_char = params
            .context
            .as_ref()
            .and_then(|ctx| ctx.trigger_character.as_deref());

        // Get source text — only needed for dot completions (to parse the line)
        let source_text = {
            let text_cache = self.text_cache.read().await;
            if let Some((_, text)) = text_cache.get(&uri.to_string()) {
                text.clone()
            } else {
                match uri.to_file_path() {
                    Ok(path) => std::fs::read_to_string(&path).unwrap_or_default(),
                    Err(_) => return Ok(None),
                }
            }
        };

        // Clone URI-specific cache (pointer copy, instant) and drop the lock immediately.
        let local_cached: Option<Arc<completion::CompletionCache>> = {
            let comp_cache = self.completion_cache.read().await;
            comp_cache.get(&uri.to_string()).cloned()
        };

        // Project-wide cache for global top-level symbol tail candidates.
        let root_cached: Option<Arc<completion::CompletionCache>> = {
            let root_key = self.project_cache_key().await;
            match root_key {
                Some(root_key) => {
                    let ast_cache = self.ast_cache.read().await;
                    ast_cache
                        .get(&root_key)
                        .map(|root_build| root_build.completion_cache.clone())
                }
                None => None,
            }
        };

        // Base cache remains per-file first; root cache is only a fallback.
        let cached = local_cached.or(root_cached.clone());

        if cached.is_none() {
            // Use pre-built completion cache from CachedBuild
            let ast_cache = self.ast_cache.clone();
            let completion_cache = self.completion_cache.clone();
            let uri_string = uri.to_string();
            tokio::spawn(async move {
                let cached_build = {
                    let cache = ast_cache.read().await;
                    match cache.get(&uri_string) {
                        Some(v) => v.clone(),
                        None => return,
                    }
                };
                completion_cache
                    .write()
                    .await
                    .insert(uri_string.into(), cached_build.completion_cache.clone());
            });
        }

        let cache_ref = cached.as_deref();

        // Look up the AST file_id for scope-aware resolution
        let file_id = {
            let uri_path = uri.to_file_path().ok();
            cache_ref.and_then(|c| {
                uri_path.as_ref().and_then(|p| {
                    let path_str = p.to_str()?;
                    c.path_to_file_id.get(path_str).copied()
                })
            })
        };

        let current_file_path = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()));

        // --- Import path completions ---
        // Use tree-sitter to determine whether the cursor is inside an import
        // string.  This is exact: it finds `import_directive > string` nodes
        // and checks if the cursor falls within the inner range (excluding
        // quotes).  This avoids false positives for arbitrary string literals
        // like `string memory s = "l`.
        //
        // For `"` / `'` trigger chars the LSP trigger position is the column
        // of the quote character itself; the inside of the string starts one
        // character to the right.
        let check_pos = if matches!(trigger_char, Some("\"") | Some("'")) {
            Position {
                line: position.line,
                character: position.character.saturating_add(1),
            }
        } else {
            position
        };

        // --- Assembly dialect completions ---
        // `assembly ("memory-safe") {}` — the only valid Solidity assembly
        // dialect is "memory-safe".  Fire exactly one completion item when
        // the cursor is inside the assembly_flags string.
        if let Some(asm_range) =
            links::ts_cursor_in_assembly_flags(source_text.as_bytes(), check_pos)
        {
            let text_edit = CompletionTextEdit::Edit(TextEdit {
                range: Range {
                    start: Position {
                        line: position.line,
                        character: asm_range.start.character,
                    },
                    end: Position {
                        line: position.line,
                        character: check_pos.character,
                    },
                },
                new_text: "memory-safe".to_string(),
            });
            let item = CompletionItem {
                label: "memory-safe".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("Solidity assembly dialect".to_string()),
                filter_text: Some("memory-safe".to_string()),
                text_edit: Some(text_edit),
                ..Default::default()
            };
            return Ok(Some(CompletionResponse::List(CompletionList {
                is_incomplete: false,
                items: vec![item],
            })));
        }

        // --- Import path completions ---
        // Use tree-sitter to determine whether the cursor is inside an import
        // string.  This is exact: it finds `import_directive > string` nodes
        // and checks if the cursor falls within the inner range (excluding
        // quotes).  This avoids false positives for arbitrary string literals
        // like `string memory s = "l`.
        if let Some(import_range) =
            links::ts_cursor_in_import_string(source_text.as_bytes(), check_pos)
        {
            if let Ok(current_file) = uri.to_file_path() {
                let foundry_cfg = self.foundry_config.read().await.clone();
                let project_root = foundry_cfg.root.clone();
                let remappings = crate::solc::resolve_remappings(&foundry_cfg).await;
                // Replace only the already-typed portion of the path so the
                // client inserts cleanly (no duplication).
                let typed_range = Some((
                    position.line,
                    import_range.start.character,
                    check_pos.character,
                ));
                let items = completion::all_sol_import_paths(
                    &current_file,
                    &project_root,
                    &remappings,
                    typed_range,
                );
                return Ok(Some(CompletionResponse::List(CompletionList {
                    is_incomplete: true,
                    items,
                })));
            }
            return Ok(None);
        }

        // A `"` or `'` trigger that is not inside an import string or assembly
        // flags string should never produce completions — return null so the
        // client does not show a spurious popup.
        if matches!(trigger_char, Some("\"") | Some("'")) {
            return Ok(None);
        }

        let tail_candidates = if trigger_char == Some(".") {
            vec![]
        } else {
            root_cached.as_deref().map_or_else(Vec::new, |c| {
                completion::top_level_importable_completion_candidates(
                    c,
                    current_file_path.as_deref(),
                    &source_text,
                )
            })
        };

        let result = completion::handle_completion_with_tail_candidates(
            cache_ref,
            &source_text,
            position,
            trigger_char,
            file_id,
            tail_candidates,
        );
        Ok(result)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/definition request")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let source_text = String::from_utf8_lossy(&source_bytes).to_string();

        // Extract the identifier name under the cursor for tree-sitter validation.
        let cursor_name = goto::cursor_context(&source_text, position).map(|ctx| ctx.name);

        // Determine if the file is dirty (unsaved edits since last build).
        // When dirty, AST byte offsets are stale so we prefer tree-sitter.
        // When clean, AST has proper semantic resolution (scoping, types).
        let (is_dirty, cached_build) = {
            let text_version = self
                .text_cache
                .read()
                .await
                .get(&uri.to_string())
                .map(|(v, _)| *v)
                .unwrap_or(0);
            let cb = self.get_or_fetch_build(&uri, &file_path, false).await;
            let build_version = cb.as_ref().map(|b| b.build_version).unwrap_or(0);
            (text_version > build_version, cb)
        };

        // Validate a tree-sitter result: read the target source and check that
        // the text at the location matches the cursor identifier. Tree-sitter
        // resolves by name so a mismatch means it landed on the wrong node.
        // AST results are NOT validated — the AST can legitimately resolve to a
        // different name (e.g. `.selector` → error declaration).
        let validate_ts = |loc: &Location| -> bool {
            let Some(ref name) = cursor_name else {
                return true; // can't validate, trust it
            };
            let target_src = if loc.uri == uri {
                Some(source_text.clone())
            } else {
                loc.uri
                    .to_file_path()
                    .ok()
                    .and_then(|p| std::fs::read_to_string(&p).ok())
            };
            match target_src {
                Some(src) => goto::validate_goto_target(&src, loc, name),
                None => true, // can't read target, trust it
            }
        };

        if is_dirty {
            self.client
                .log_message(MessageType::INFO, "file is dirty, trying tree-sitter first")
                .await;

            // DIRTY: tree-sitter first (validated) → AST fallback
            let ts_result = {
                let comp_cache = self.completion_cache.read().await;
                let text_cache = self.text_cache.read().await;
                if let Some(cc) = comp_cache.get(&uri.to_string()) {
                    goto::goto_definition_ts(&source_text, position, &uri, cc, &text_cache)
                } else {
                    None
                }
            };

            if let Some(location) = ts_result {
                if validate_ts(&location) {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "found definition (tree-sitter) at {}:{}",
                                location.uri, location.range.start.line
                            ),
                        )
                        .await;
                    return Ok(Some(GotoDefinitionResponse::from(location)));
                }
                self.client
                    .log_message(
                        MessageType::INFO,
                        "tree-sitter result failed validation, trying AST fallback",
                    )
                    .await;
            }

            // Tree-sitter failed or didn't validate — try name-based AST lookup.
            // Instead of matching by byte offset (which is stale on dirty files),
            // search cached AST nodes whose source text matches the cursor name
            // and follow their referencedDeclaration.
            if let Some(ref cb) = cached_build
                && let Some(ref name) = cursor_name
            {
                let byte_hint = goto::pos_to_bytes(&source_bytes, position);
                if let Some(location) = goto::goto_declaration_by_name(cb, &uri, name, byte_hint) {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "found definition (AST by name) at {}:{}",
                                location.uri, location.range.start.line
                            ),
                        )
                        .await;
                    return Ok(Some(GotoDefinitionResponse::from(location)));
                }
            }
        } else {
            // CLEAN: AST first → tree-sitter fallback (validated)
            if let Some(ref cb) = cached_build
                && let Some(location) =
                    goto::goto_declaration_cached(cb, &uri, position, &source_bytes)
            {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "found definition (AST) at {}:{}",
                            location.uri, location.range.start.line
                        ),
                    )
                    .await;
                return Ok(Some(GotoDefinitionResponse::from(location)));
            }

            // AST couldn't resolve — try tree-sitter fallback (validated)
            let ts_result = {
                let comp_cache = self.completion_cache.read().await;
                let text_cache = self.text_cache.read().await;
                if let Some(cc) = comp_cache.get(&uri.to_string()) {
                    goto::goto_definition_ts(&source_text, position, &uri, cc, &text_cache)
                } else {
                    None
                }
            };

            if let Some(location) = ts_result {
                if validate_ts(&location) {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "found definition (tree-sitter fallback) at {}:{}",
                                location.uri, location.range.start.line
                            ),
                        )
                        .await;
                    return Ok(Some(GotoDefinitionResponse::from(location)));
                }
                self.client
                    .log_message(MessageType::INFO, "tree-sitter fallback failed validation")
                    .await;
            }
        }

        self.client
            .log_message(MessageType::INFO, "no definition found")
            .await;
        Ok(None)
    }

    async fn goto_declaration(
        &self,
        params: request::GotoDeclarationParams,
    ) -> tower_lsp::jsonrpc::Result<Option<request::GotoDeclarationResponse>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/declaration request")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };

        if let Some(location) =
            goto::goto_declaration_cached(&cached_build, &uri, position, &source_bytes)
        {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "found declaration at {}:{}",
                        location.uri, location.range.start.line
                    ),
                )
                .await;
            Ok(Some(request::GotoDeclarationResponse::from(location)))
        } else {
            self.client
                .log_message(MessageType::INFO, "no declaration found")
                .await;
            Ok(None)
        }
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        self.client
            .log_message(MessageType::INFO, "Got a textDocument/references request")
            .await;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "Invalid file URI")
                    .await;
                return Ok(None);
            }
        };
        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        let file_build = self.get_or_fetch_build(&uri, &file_path, true).await;
        let file_build = match file_build {
            Some(cb) => cb,
            None => return Ok(None),
        };
        let mut project_build = self.ensure_project_cached_build().await;
        let current_abs = file_path.to_string_lossy().to_string();
        if self.use_solc
            && self.settings.read().await.project_index.full_project_scan
            && project_build
                .as_ref()
                .is_some_and(|b| !b.nodes.contains_key(current_abs.as_str()))
        {
            let foundry_config = self.foundry_config_for_file(&file_path).await;
            let remappings = crate::solc::resolve_remappings(&foundry_config).await;
            let changed = vec![PathBuf::from(&current_abs)];
            let cfg_for_plan = foundry_config.clone();
            let remappings_for_plan = remappings.clone();
            let affected_set = tokio::task::spawn_blocking(move || {
                compute_reverse_import_closure(&cfg_for_plan, &changed, &remappings_for_plan)
            })
            .await
            .ok()
            .unwrap_or_default();
            let mut affected_files: Vec<PathBuf> = affected_set.into_iter().collect();
            if affected_files.is_empty() {
                affected_files.push(PathBuf::from(&current_abs));
            }
            let text_cache_snapshot = self.text_cache.read().await.clone();
            match crate::solc::solc_project_index_scoped(
                &foundry_config,
                Some(&self.client),
                Some(&text_cache_snapshot),
                &affected_files,
            )
            .await
            {
                Ok(ast_data) => {
                    let scoped_build = Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                    if let Some(root_key) = self.project_cache_key().await {
                        let merged = {
                            let mut cache = self.ast_cache.write().await;
                            let merged = if let Some(existing) = cache.get(&root_key).cloned() {
                                let mut merged = (*existing).clone();
                                match merge_scoped_cached_build(
                                    &mut merged,
                                    (*scoped_build).clone(),
                                ) {
                                    Ok(_) => Arc::new(merged),
                                    Err(_) => scoped_build.clone(),
                                }
                            } else {
                                scoped_build.clone()
                            };
                            cache.insert(root_key.into(), merged.clone());
                            merged
                        };
                        project_build = Some(merged);
                    } else {
                        project_build = Some(scoped_build);
                    }
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "references warm-refresh: scoped reindex applied (affected={})",
                                affected_files.len()
                            ),
                        )
                        .await;
                }
                Err(e) => {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("references warm-refresh: scoped reindex failed: {e}"),
                        )
                        .await;
                }
            }
        }

        // Always resolve target/local references from the current file build.
        // This avoids stale/partial project-cache misses immediately after edits.
        let mut locations = references::goto_references_cached(
            &file_build,
            &uri,
            position,
            &source_bytes,
            None,
            params.context.include_declaration,
        );

        // Cross-file: resolve target from current file, then expand in project cache.
        if let Some((def_abs_path, def_byte_offset)) =
            references::resolve_target_location(&file_build, &uri, position, &source_bytes)
        {
            if let Some(project_build) = project_build {
                let other_locations = references::goto_references_for_target(
                    &project_build,
                    &def_abs_path,
                    def_byte_offset,
                    None,
                    params.context.include_declaration,
                );
                locations.extend(other_locations);
            }

            // Search sub-project caches for references in lib test files.
            // Each sub-cache has its own node ID space; byte_to_id matches
            // the target declaration by absolute file path + byte offset.
            let sub_caches = self.sub_caches.read().await;
            for sub_cache in sub_caches.iter() {
                let sub_locations = references::goto_references_for_target(
                    sub_cache,
                    &def_abs_path,
                    def_byte_offset,
                    None,
                    params.context.include_declaration,
                );
                locations.extend(sub_locations);
            }
        }

        // Deduplicate across all caches
        let mut seen = std::collections::HashSet::new();
        locations.retain(|loc| {
            seen.insert((
                loc.uri.clone(),
                loc.range.start.line,
                loc.range.start.character,
                loc.range.end.line,
                loc.range.end.character,
            ))
        });

        if locations.is_empty() {
            self.client
                .log_message(MessageType::INFO, "No references found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Found {} references", locations.len()),
                )
                .await;
            Ok(Some(locations))
        }
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<PrepareRenameResponse>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/prepareRename request")
            .await;

        let uri = params.text_document.uri;
        let position = params.position;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        if let Some(range) = rename::get_identifier_range(&source_bytes, position) {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "prepare rename range: {}:{}",
                        range.start.line, range.start.character
                    ),
                )
                .await;
            Ok(Some(PrepareRenameResponse::Range(range)))
        } else {
            self.client
                .log_message(MessageType::INFO, "no identifier found for prepare rename")
                .await;
            Ok(None)
        }
    }

    async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/rename request")
            .await;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let file_path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };
        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let current_identifier = match rename::get_identifier_at_position(&source_bytes, position) {
            Some(id) => id,
            None => {
                self.client
                    .log_message(MessageType::ERROR, "No identifier found at position")
                    .await;
                return Ok(None);
            }
        };

        if !utils::is_valid_solidity_identifier(&new_name) {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "new name is not a valid solidity identifier",
            ));
        }

        if new_name == current_identifier {
            self.client
                .log_message(
                    MessageType::INFO,
                    "new name is the same as current identifier",
                )
                .await;
            return Ok(None);
        }

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };
        let other_builds: Vec<Arc<goto::CachedBuild>> = {
            let cache = self.ast_cache.read().await;
            cache
                .iter()
                .filter(|(key, _)| key.as_str() != uri.to_string())
                .map(|(_, v)| v.clone())
                .collect()
        };
        let other_refs: Vec<&goto::CachedBuild> = other_builds.iter().map(|v| v.as_ref()).collect();

        // Build a map of URI → file content from the text_cache so rename
        // verification reads from in-memory buffers (unsaved edits) instead
        // of from disk.
        let text_buffers: HashMap<String, Vec<u8>> = {
            let text_cache = self.text_cache.read().await;
            text_cache
                .iter()
                .map(|(uri, (_, content))| (uri.to_string(), content.as_bytes().to_vec()))
                .collect()
        };

        match rename::rename_symbol(
            &cached_build,
            &uri,
            position,
            &source_bytes,
            new_name,
            &other_refs,
            &text_buffers,
        ) {
            Some(workspace_edit) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "created rename edit with {} file(s), {} total change(s)",
                            workspace_edit
                                .changes
                                .as_ref()
                                .map(|c| c.len())
                                .unwrap_or(0),
                            workspace_edit
                                .changes
                                .as_ref()
                                .map(|c| c.values().map(|v| v.len()).sum::<usize>())
                                .unwrap_or(0)
                        ),
                    )
                    .await;

                // Return the full WorkspaceEdit to the client so the editor
                // applies all changes (including cross-file renames) via the
                // LSP protocol. This keeps undo working and avoids writing
                // files behind the editor's back.
                Ok(Some(workspace_edit))
            }

            None => {
                self.client
                    .log_message(MessageType::INFO, "No locations found for renaming")
                    .await;
                Ok(None)
            }
        }
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<SymbolInformation>>> {
        self.client
            .log_message(MessageType::INFO, "got workspace/symbol request")
            .await;

        // Collect sources from open files in text_cache
        let files: Vec<(Url, String)> = {
            let cache = self.text_cache.read().await;
            cache
                .iter()
                .filter(|(uri_str, _)| uri_str.ends_with(".sol"))
                .filter_map(|(uri_str, (_, content))| {
                    Url::parse(uri_str).ok().map(|uri| (uri, content.clone()))
                })
                .collect()
        };

        let mut all_symbols = symbols::extract_workspace_symbols(&files);
        if !params.query.is_empty() {
            let query = params.query.to_lowercase();
            all_symbols.retain(|symbol| symbol.name.to_lowercase().contains(&query));
        }
        if all_symbols.is_empty() {
            self.client
                .log_message(MessageType::INFO, "No symbols found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} symbols", all_symbols.len()),
                )
                .await;
            Ok(Some(all_symbols))
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/documentSymbol request")
            .await;
        let uri = params.text_document.uri;
        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        // Read source from text_cache (open files) or disk
        let source = {
            let cache = self.text_cache.read().await;
            cache
                .get(&uri.to_string())
                .map(|(_, content)| content.clone())
        };
        let source = match source {
            Some(s) => s,
            None => match std::fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            },
        };

        let symbols = symbols::extract_document_symbols(&source);
        if symbols.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no document symbols found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} document symbols", symbols.len()),
                )
                .await;
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        self.client
            .log_message(
                MessageType::INFO,
                "got textDocument/documentHighlight request",
            )
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let highlights = highlight::document_highlights(&source, position);

        if highlights.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no document highlights found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} document highlights", highlights.len()),
                )
                .await;
            Ok(Some(highlights))
        }
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/hover request")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };

        let result = hover::hover_info(&cached_build, &uri, position, &source_bytes);

        if result.is_some() {
            self.client
                .log_message(MessageType::INFO, "hover info found")
                .await;
        } else {
            self.client
                .log_message(MessageType::INFO, "no hover info found")
                .await;
        }

        Ok(result)
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/signatureHelp request")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };

        let result = hover::signature_help(&cached_build, &source_bytes, position);

        Ok(result)
    }

    async fn document_link(
        &self,
        params: DocumentLinkParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<DocumentLink>>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/documentLink request")
            .await;

        let uri = params.text_document.uri;
        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };

        let result = links::document_links(&cached_build, &uri, &source_bytes);

        if result.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no document links found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} document links", result.len()),
                )
                .await;
            Ok(Some(result))
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(
                MessageType::INFO,
                "got textDocument/semanticTokens/full request",
            )
            .await;

        let uri = params.text_document.uri;
        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                // File not open in editor — try reading from disk
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let mut tokens = semantic_tokens::semantic_tokens_full(&source);

        // Generate a unique result_id and cache the tokens for delta requests
        let id = self.semantic_token_id.fetch_add(1, Ordering::Relaxed);
        let result_id = id.to_string();
        tokens.result_id = Some(result_id.clone());

        {
            let mut cache = self.semantic_token_cache.write().await;
            cache.insert(uri.to_string().into(), (result_id, tokens.data.clone()));
        }

        Ok(Some(SemanticTokensResult::Tokens(tokens)))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensRangeResult>> {
        self.client
            .log_message(
                MessageType::INFO,
                "got textDocument/semanticTokens/range request",
            )
            .await;

        let uri = params.text_document.uri;
        let range = params.range;
        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let tokens =
            semantic_tokens::semantic_tokens_range(&source, range.start.line, range.end.line);

        Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensFullDeltaResult>> {
        self.client
            .log_message(
                MessageType::INFO,
                "got textDocument/semanticTokens/full/delta request",
            )
            .await;

        let uri = params.text_document.uri;
        let previous_result_id = params.previous_result_id;

        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let mut new_tokens = semantic_tokens::semantic_tokens_full(&source);

        // Generate a new result_id
        let id = self.semantic_token_id.fetch_add(1, Ordering::Relaxed);
        let new_result_id = id.to_string();
        new_tokens.result_id = Some(new_result_id.clone());

        let uri_str = uri.to_string();

        // Look up the previous tokens by result_id
        let old_tokens = {
            let cache = self.semantic_token_cache.read().await;
            cache
                .get(&uri_str)
                .filter(|(rid, _)| *rid == previous_result_id)
                .map(|(_, tokens)| tokens.clone())
        };

        // Update the cache with the new tokens
        {
            let mut cache = self.semantic_token_cache.write().await;
            cache.insert(
                uri_str.into(),
                (new_result_id.clone(), new_tokens.data.clone()),
            );
        }

        match old_tokens {
            Some(old) => {
                // Compute delta
                let edits = semantic_tokens::compute_delta(&old, &new_tokens.data);
                Ok(Some(SemanticTokensFullDeltaResult::TokensDelta(
                    SemanticTokensDelta {
                        result_id: Some(new_result_id),
                        edits,
                    },
                )))
            }
            None => {
                // No cached previous — fall back to full response
                Ok(Some(SemanticTokensFullDeltaResult::Tokens(new_tokens)))
            }
        }
    }

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<FoldingRange>>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/foldingRange request")
            .await;

        let uri = params.text_document.uri;

        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let ranges = folding::folding_ranges(&source);

        if ranges.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no folding ranges found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} folding ranges", ranges.len()),
                )
                .await;
            Ok(Some(ranges))
        }
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<SelectionRange>>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/selectionRange request")
            .await;

        let uri = params.text_document.uri;

        let source = {
            let cache = self.text_cache.read().await;
            cache.get(&uri.to_string()).map(|(_, s)| s.clone())
        };

        let source = match source {
            Some(s) => s,
            None => {
                let file_path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => return Ok(None),
                };
                match std::fs::read_to_string(&file_path) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
        };

        let ranges = selection::selection_ranges(&source, &params.positions);

        if ranges.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no selection ranges found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} selection ranges", ranges.len()),
                )
                .await;
            Ok(Some(ranges))
        }
    }

    async fn inlay_hint(
        &self,
        params: InlayHintParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<InlayHint>>> {
        self.client
            .log_message(MessageType::INFO, "got textDocument/inlayHint request")
            .await;

        let uri = params.text_document.uri;
        let range = params.range;

        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                self.client
                    .log_message(MessageType::ERROR, "invalid file uri")
                    .await;
                return Ok(None);
            }
        };

        let source_bytes = match self.get_source_bytes(&uri, &file_path).await {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let cached_build = self.get_or_fetch_build(&uri, &file_path, false).await;
        let cached_build = match cached_build {
            Some(cb) => cb,
            None => return Ok(None),
        };

        let mut hints = inlay_hints::inlay_hints(&cached_build, &uri, range, &source_bytes);

        // Filter hints based on settings.
        let settings = self.settings.read().await;
        if !settings.inlay_hints.parameters {
            hints.retain(|h| h.kind != Some(InlayHintKind::PARAMETER));
        }
        if !settings.inlay_hints.gas_estimates {
            hints.retain(|h| h.kind != Some(InlayHintKind::TYPE));
        }

        if hints.is_empty() {
            self.client
                .log_message(MessageType::INFO, "no inlay hints found")
                .await;
            Ok(None)
        } else {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("found {} inlay hints", hints.len()),
                )
                .await;
            Ok(Some(hints))
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        use crate::code_actions::FixKind;

        let uri = &params.text_document.uri;

        // Resolve source text once — needed by all tree-sitter backed actions.
        let source: Option<String> = if let Ok(path) = uri.to_file_path() {
            self.get_source_bytes(uri, &path)
                .await
                .map(|b| String::from_utf8_lossy(&b).into_owned())
        } else {
            None
        };

        let db = &self.code_action_db;
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        for diag in &params.context.diagnostics {
            // ── forge-lint string codes ───────────────────────────────────────
            if let Some(NumberOrString::String(s)) = &diag.code {
                if s == "unused-import" {
                    if let Some(edit) = source.as_deref().and_then(|src| {
                        goto::code_action_edit(
                            src,
                            diag.range,
                            goto::CodeActionKind::DeleteNodeByKind {
                                node_kind: "import_directive",
                            },
                        )
                    }) {
                        let mut changes = HashMap::new();
                        changes.insert(uri.clone(), vec![edit]);
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove unused import".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes),
                                ..Default::default()
                            }),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));
                    }
                    continue;
                }
            }

            // Diagnostics from solc carry the error code as a string.
            let code: ErrorCode = match &diag.code {
                Some(NumberOrString::String(s)) => match s.parse() {
                    Ok(n) => ErrorCode(n),
                    Err(_) => continue,
                },
                _ => continue,
            };

            // ── JSON-driven path ──────────────────────────────────────────────
            if let Some(def) = db.get(&code) {
                // Build the TextEdit from the fix kind.
                let edit_opt: Option<TextEdit> = match &def.fix {
                    FixKind::Insert { text, anchor: _ } => {
                        // InsertAtFileStart is the only anchor for now.
                        goto::code_action_edit(
                            source.as_deref().unwrap_or(""),
                            diag.range,
                            goto::CodeActionKind::InsertAtFileStart { text },
                        )
                    }

                    FixKind::ReplaceToken {
                        replacement,
                        walk_to,
                    } => source.as_deref().and_then(|src| {
                        goto::code_action_edit(
                            src,
                            diag.range,
                            goto::CodeActionKind::ReplaceToken {
                                replacement,
                                walk_to: walk_to.as_deref(),
                            },
                        )
                    }),

                    FixKind::DeleteToken => source.as_deref().and_then(|src| {
                        goto::code_action_edit(src, diag.range, goto::CodeActionKind::DeleteToken)
                    }),

                    FixKind::DeleteNode { node_kind } => {
                        // Only variable_declaration_statement supported so far.
                        if node_kind == "variable_declaration_statement" {
                            source.as_deref().and_then(|src| {
                                goto::code_action_edit(
                                    src,
                                    diag.range,
                                    goto::CodeActionKind::DeleteLocalVar,
                                )
                            })
                        } else {
                            None
                        }
                    }

                    FixKind::DeleteChildNode {
                        walk_to,
                        child_kinds,
                    } => {
                        let ck: Vec<&str> = child_kinds.iter().map(|s| s.as_str()).collect();
                        source.as_deref().and_then(|src| {
                            goto::code_action_edit(
                                src,
                                diag.range,
                                goto::CodeActionKind::DeleteChildNode {
                                    walk_to,
                                    child_kinds: &ck,
                                },
                            )
                        })
                    }

                    FixKind::ReplaceChildNode {
                        walk_to,
                        child_kind,
                        replacement,
                    } => source.as_deref().and_then(|src| {
                        goto::code_action_edit(
                            src,
                            diag.range,
                            goto::CodeActionKind::ReplaceChildNode {
                                walk_to,
                                child_kind,
                                replacement,
                            },
                        )
                    }),

                    FixKind::InsertBeforeNode {
                        walk_to,
                        before_child,
                        text,
                    } => {
                        let bc: Vec<&str> = before_child.iter().map(|s| s.as_str()).collect();
                        source.as_deref().and_then(|src| {
                            goto::code_action_edit(
                                src,
                                diag.range,
                                goto::CodeActionKind::InsertBeforeNode {
                                    walk_to,
                                    before_child: &bc,
                                    text,
                                },
                            )
                        })
                    }

                    // Custom fixes are handled below.
                    FixKind::Custom => None,
                };

                if let Some(edit) = edit_opt {
                    let mut changes = HashMap::new();
                    changes.insert(uri.clone(), vec![edit]);
                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: def.title.clone(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diag.clone()]),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }),
                        is_preferred: Some(true),
                        ..Default::default()
                    }));
                    continue; // handled — skip custom fallback
                }

                // If it's not Custom and edit_opt was None, the TS lookup failed
                // (e.g. file not parseable). Nothing to emit for this diagnostic.
                if !matches!(def.fix, FixKind::Custom) {
                    continue;
                }
            }

            // ── Custom / hand-written fallback ────────────────────────────────
            // Only reaches here for codes with kind=custom or codes not in the DB.
            // Add bespoke arms here as needed.
            #[allow(clippy::match_single_binding)]
            match code {
                // 2018 — state mutability can be restricted to pure/view.
                // The replacement (pure vs view) is embedded in the diagnostic message.
                // 9456 — missing `override` specifier.
                // Needs a TS walk to find the right insertion point in the modifier list.
                _ => {}
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn will_rename_files(
        &self,
        params: RenameFilesParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/willRenameFiles: {} file(s)", params.files.len()),
            )
            .await;
        if !self
            .settings
            .read()
            .await
            .file_operations
            .update_imports_on_rename
        {
            self.client
                .log_message(
                    MessageType::INFO,
                    "willRenameFiles: updateImportsOnRename disabled",
                )
                .await;
            return Ok(None);
        }

        // ── Phase 1: discover source files (blocking I/O) ──────────────
        let config = self.foundry_config.read().await.clone();
        let project_root = config.root.clone();
        let source_files: Vec<String> = tokio::task::spawn_blocking(move || {
            crate::solc::discover_source_files(&config)
                .into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect()
        })
        .await
        .unwrap_or_default();

        if source_files.is_empty() {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "willRenameFiles: no source files found",
                )
                .await;
            return Ok(None);
        }

        // ── Phase 2: parse rename params & expand folders ──────────────
        let raw_renames: Vec<(std::path::PathBuf, std::path::PathBuf)> = params
            .files
            .iter()
            .filter_map(|fr| {
                let old_uri = Url::parse(&fr.old_uri).ok()?;
                let new_uri = Url::parse(&fr.new_uri).ok()?;
                let old_path = old_uri.to_file_path().ok()?;
                let new_path = new_uri.to_file_path().ok()?;
                Some((old_path, new_path))
            })
            .collect();

        let renames = file_operations::expand_folder_renames(&raw_renames, &source_files);

        if renames.is_empty() {
            return Ok(None);
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "willRenameFiles: {} rename(s) after folder expansion",
                    renames.len()
                ),
            )
            .await;

        // ── Phase 3: hydrate text_cache (blocking I/O) ─────────────────
        // Collect which files need reading from disk (not already in cache).
        let files_to_read: Vec<(String, String)> = {
            let tc = self.text_cache.read().await;
            source_files
                .iter()
                .filter_map(|fs_path| {
                    let uri = Url::from_file_path(fs_path).ok()?;
                    let uri_str = uri.to_string();
                    if tc.contains_key(&uri_str) {
                        None
                    } else {
                        Some((uri_str, fs_path.clone()))
                    }
                })
                .collect()
        };

        if !files_to_read.is_empty() {
            let loaded: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
                files_to_read
                    .into_iter()
                    .filter_map(|(uri_str, fs_path)| {
                        let content = std::fs::read_to_string(&fs_path).ok()?;
                        Some((uri_str, content))
                    })
                    .collect()
            })
            .await
            .unwrap_or_default();

            let mut tc = self.text_cache.write().await;
            for (uri_str, content) in loaded {
                tc.entry(uri_str.into()).or_insert((0, content));
            }
        }

        // ── Phase 4: compute edits (pure, no I/O) ──────────────────────
        // Build source-bytes provider that reads from the cache held behind
        // the Arc<RwLock>.  We hold a read guard only for the duration of
        // each lookup, not for the full computation.
        let text_cache = self.text_cache.clone();
        let result = {
            let tc = text_cache.read().await;
            let get_source_bytes = |fs_path: &str| -> Option<Vec<u8>> {
                let uri = Url::from_file_path(fs_path).ok()?;
                let (_, content) = tc.get(&uri.to_string())?;
                Some(content.as_bytes().to_vec())
            };

            file_operations::rename_imports(
                &source_files,
                &renames,
                &project_root,
                &get_source_bytes,
            )
        };

        // ── Phase 5: log diagnostics ───────────────────────────────────
        let stats = &result.stats;
        if stats.read_failures > 0 || stats.pathdiff_failures > 0 || stats.duplicate_renames > 0 {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!(
                        "willRenameFiles stats: read_failures={}, pathdiff_failures={}, \
                         duplicate_renames={}, no_parent={}, no_op_skips={}, dedup_skips={}",
                        stats.read_failures,
                        stats.pathdiff_failures,
                        stats.duplicate_renames,
                        stats.no_parent,
                        stats.no_op_skips,
                        stats.dedup_skips,
                    ),
                )
                .await;
        }

        let all_edits = result.edits;

        if all_edits.is_empty() {
            self.client
                .log_message(MessageType::INFO, "willRenameFiles: no import edits needed")
                .await;
            return Ok(None);
        }

        // ── Phase 6: patch own text_cache ──────────────────────────────
        {
            let mut tc = self.text_cache.write().await;
            let patched = file_operations::apply_edits_to_cache(&all_edits, &mut tc);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("willRenameFiles: patched {} cached file(s)", patched),
                )
                .await;
        }

        let total_edits: usize = all_edits.values().map(|v| v.len()).sum();
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "willRenameFiles: {} edit(s) across {} file(s)",
                    total_edits,
                    all_edits.len()
                ),
            )
            .await;

        Ok(Some(WorkspaceEdit {
            changes: Some(all_edits),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/didRenameFiles: {} file(s)", params.files.len()),
            )
            .await;
        self.project_cache_dirty.store(true, Ordering::Release);
        {
            let mut changed = self.project_cache_changed_files.write().await;
            for file in &params.files {
                if let Ok(old_uri) = Url::parse(&file.old_uri)
                    && let Ok(old_path) = old_uri.to_file_path()
                {
                    changed.insert(old_path.to_string_lossy().to_string());
                }
                if let Ok(new_uri) = Url::parse(&file.new_uri)
                    && let Ok(new_path) = new_uri.to_file_path()
                {
                    changed.insert(new_path.to_string_lossy().to_string());
                }
            }
        }

        // ── Phase 1: parse params & expand folder renames ──────────────
        let raw_uri_pairs: Vec<(Url, Url)> = params
            .files
            .iter()
            .filter_map(|fr| {
                let old_uri = Url::parse(&fr.old_uri).ok()?;
                let new_uri = Url::parse(&fr.new_uri).ok()?;
                Some((old_uri, new_uri))
            })
            .collect();

        let file_renames = {
            let tc = self.text_cache.read().await;
            let cache_paths: Vec<std::path::PathBuf> = tc
                .keys()
                .filter_map(|k| Url::parse(k).ok())
                .filter_map(|u| u.to_file_path().ok())
                .collect();
            drop(tc);

            // Include discovered project files so folder renames also migrate
            // entries that aren't currently present in text_cache.
            let cfg = self.foundry_config.read().await.clone();
            let discovered_paths =
                tokio::task::spawn_blocking(move || crate::solc::discover_source_files(&cfg))
                    .await
                    .unwrap_or_default();

            let mut all_paths: HashSet<std::path::PathBuf> = discovered_paths.into_iter().collect();
            all_paths.extend(cache_paths);
            let all_paths: Vec<std::path::PathBuf> = all_paths.into_iter().collect();

            file_operations::expand_folder_renames_from_paths(&raw_uri_pairs, &all_paths)
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "didRenameFiles: migrating {} cache entry/entries",
                    file_renames.len()
                ),
            )
            .await;

        // ── Phase 2: migrate per-file caches ───────────────────────────
        // Take a single write lock per cache type and do all migrations
        // in one pass (avoids repeated lock/unlock per file).
        {
            let mut tc = self.text_cache.write().await;
            for (old_key, new_key) in &file_renames {
                if let Some(entry) = tc.remove(old_key) {
                    tc.insert(new_key.clone().into(), entry);
                }
            }
        }
        {
            let mut ac = self.ast_cache.write().await;
            for (old_key, _) in &file_renames {
                ac.remove(old_key);
            }
        }
        {
            let mut cc = self.completion_cache.write().await;
            for (old_key, _) in &file_renames {
                cc.remove(old_key);
            }
        }

        // Invalidate the project index cache and rebuild so subsequent
        // willRenameFiles requests see the updated file layout.
        let root_key = self.project_cache_key().await;
        if let Some(ref key) = root_key {
            self.ast_cache.write().await.remove(key);
        }

        let foundry_config = self.foundry_config.read().await.clone();
        let ast_cache = self.ast_cache.clone();
        let client = self.client.clone();
        // Snapshot text_cache so the re-index uses in-memory content
        // (with updated import paths from willRenameFiles) rather than
        // reading from disk where files may not yet reflect the edits.
        let text_cache_snapshot = self.text_cache.read().await.clone();

        tokio::spawn(async move {
            let Some(cache_key) = root_key else {
                return;
            };
            match crate::solc::solc_project_index(
                &foundry_config,
                Some(&client),
                Some(&text_cache_snapshot),
            )
            .await
            {
                Ok(ast_data) => {
                    let cached_build = Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                    let source_count = cached_build.nodes.len();
                    ast_cache
                        .write()
                        .await
                        .insert(cache_key.into(), cached_build);
                    client
                        .log_message(
                            MessageType::INFO,
                            format!("didRenameFiles: re-indexed {} source files", source_count),
                        )
                        .await;
                }
                Err(e) => {
                    client
                        .log_message(
                            MessageType::WARNING,
                            format!("didRenameFiles: re-index failed: {e}"),
                        )
                        .await;
                }
            }
        });
    }

    async fn will_delete_files(
        &self,
        params: DeleteFilesParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/willDeleteFiles: {} file(s)", params.files.len()),
            )
            .await;
        if !update_imports_on_delete_enabled(&*self.settings.read().await) {
            self.client
                .log_message(
                    MessageType::INFO,
                    "willDeleteFiles: updateImportsOnDelete disabled",
                )
                .await;
            return Ok(None);
        }

        let config = self.foundry_config.read().await.clone();
        let project_root = config.root.clone();
        let source_files: Vec<String> = tokio::task::spawn_blocking(move || {
            crate::solc::discover_source_files(&config)
                .into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect()
        })
        .await
        .unwrap_or_default();

        if source_files.is_empty() {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "willDeleteFiles: no source files found",
                )
                .await;
            return Ok(None);
        }

        let raw_deletes: Vec<std::path::PathBuf> = params
            .files
            .iter()
            .filter_map(|fd| Url::parse(&fd.uri).ok())
            .filter_map(|u| u.to_file_path().ok())
            .collect();

        let deletes = file_operations::expand_folder_deletes(&raw_deletes, &source_files);
        if deletes.is_empty() {
            return Ok(None);
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "willDeleteFiles: {} delete target(s) after folder expansion",
                    deletes.len()
                ),
            )
            .await;

        let files_to_read: Vec<(String, String)> = {
            let tc = self.text_cache.read().await;
            source_files
                .iter()
                .filter_map(|fs_path| {
                    let uri = Url::from_file_path(fs_path).ok()?;
                    let uri_str = uri.to_string();
                    if tc.contains_key(&uri_str) {
                        None
                    } else {
                        Some((uri_str, fs_path.clone()))
                    }
                })
                .collect()
        };

        if !files_to_read.is_empty() {
            let loaded: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
                files_to_read
                    .into_iter()
                    .filter_map(|(uri_str, fs_path)| {
                        let content = std::fs::read_to_string(&fs_path).ok()?;
                        Some((uri_str, content))
                    })
                    .collect()
            })
            .await
            .unwrap_or_default();

            let mut tc = self.text_cache.write().await;
            for (uri_str, content) in loaded {
                tc.entry(uri_str.into()).or_insert((0, content));
            }
        }

        let result = {
            let tc = self.text_cache.read().await;
            let get_source_bytes = |fs_path: &str| -> Option<Vec<u8>> {
                let uri = Url::from_file_path(fs_path).ok()?;
                let (_, content) = tc.get(&uri.to_string())?;
                Some(content.as_bytes().to_vec())
            };

            file_operations::delete_imports(
                &source_files,
                &deletes,
                &project_root,
                &get_source_bytes,
            )
        };

        let stats = &result.stats;
        if stats.read_failures > 0
            || stats.statement_range_failures > 0
            || stats.duplicate_deletes > 0
        {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!(
                        "willDeleteFiles stats: read_failures={}, statement_range_failures={}, \
                         duplicate_deletes={}, no_parent={}, dedup_skips={}",
                        stats.read_failures,
                        stats.statement_range_failures,
                        stats.duplicate_deletes,
                        stats.no_parent,
                        stats.dedup_skips,
                    ),
                )
                .await;
        }

        let all_edits = result.edits;
        if all_edits.is_empty() {
            self.client
                .log_message(
                    MessageType::INFO,
                    "willDeleteFiles: no import-removal edits needed",
                )
                .await;
            return Ok(None);
        }

        {
            let mut tc = self.text_cache.write().await;
            let patched = file_operations::apply_edits_to_cache(&all_edits, &mut tc);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("willDeleteFiles: patched {} cached file(s)", patched),
                )
                .await;
        }

        let total_edits: usize = all_edits.values().map(|v| v.len()).sum();
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "willDeleteFiles: {} edit(s) across {} file(s)",
                    total_edits,
                    all_edits.len()
                ),
            )
            .await;

        Ok(Some(WorkspaceEdit {
            changes: Some(all_edits),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/didDeleteFiles: {} file(s)", params.files.len()),
            )
            .await;
        self.project_cache_dirty.store(true, Ordering::Release);
        {
            let mut changed = self.project_cache_changed_files.write().await;
            for file in &params.files {
                if let Ok(uri) = Url::parse(&file.uri)
                    && let Ok(path) = uri.to_file_path()
                {
                    changed.insert(path.to_string_lossy().to_string());
                }
            }
        }

        let raw_delete_uris: Vec<Url> = params
            .files
            .iter()
            .filter_map(|fd| Url::parse(&fd.uri).ok())
            .collect();

        let deleted_paths = {
            let tc = self.text_cache.read().await;
            let cache_paths: Vec<std::path::PathBuf> = tc
                .keys()
                .filter_map(|k| Url::parse(k).ok())
                .filter_map(|u| u.to_file_path().ok())
                .collect();
            drop(tc);

            let cfg = self.foundry_config.read().await.clone();
            let discovered_paths =
                tokio::task::spawn_blocking(move || crate::solc::discover_source_files(&cfg))
                    .await
                    .unwrap_or_default();

            let mut all_paths: HashSet<std::path::PathBuf> = discovered_paths.into_iter().collect();
            all_paths.extend(cache_paths);
            let all_paths: Vec<std::path::PathBuf> = all_paths.into_iter().collect();

            file_operations::expand_folder_deletes_from_paths(&raw_delete_uris, &all_paths)
        };

        let mut deleted_keys: HashSet<String> = HashSet::new();
        let mut deleted_uris: Vec<Url> = Vec::new();
        for path in deleted_paths {
            if let Ok(uri) = Url::from_file_path(&path) {
                deleted_keys.insert(uri.to_string());
                deleted_uris.push(uri);
            }
        }
        if deleted_keys.is_empty() {
            return;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "didDeleteFiles: deleting {} cache/diagnostic entry(ies)",
                    deleted_keys.len()
                ),
            )
            .await;

        for uri in &deleted_uris {
            self.client
                .publish_diagnostics(uri.clone(), vec![], None)
                .await;
        }

        let mut removed_text = 0usize;
        let mut removed_ast = 0usize;
        let mut removed_completion = 0usize;
        let mut removed_semantic = 0usize;
        let mut removed_pending_create = 0usize;
        {
            let mut tc = self.text_cache.write().await;
            for key in &deleted_keys {
                if tc.remove(key).is_some() {
                    removed_text += 1;
                }
            }
        }
        {
            let mut ac = self.ast_cache.write().await;
            for key in &deleted_keys {
                if ac.remove(key).is_some() {
                    removed_ast += 1;
                }
            }
        }
        {
            let mut cc = self.completion_cache.write().await;
            for key in &deleted_keys {
                if cc.remove(key).is_some() {
                    removed_completion += 1;
                }
            }
        }
        {
            let mut sc = self.semantic_token_cache.write().await;
            for key in &deleted_keys {
                if sc.remove(key).is_some() {
                    removed_semantic += 1;
                }
            }
        }
        {
            let mut pending = self.pending_create_scaffold.write().await;
            for key in &deleted_keys {
                if pending.remove(key) {
                    removed_pending_create += 1;
                }
            }
        }
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "didDeleteFiles: removed caches text={} ast={} completion={} semantic={} pendingCreate={}",
                    removed_text,
                    removed_ast,
                    removed_completion,
                    removed_semantic,
                    removed_pending_create,
                ),
            )
            .await;

        let root_key = self.project_cache_key().await;
        if let Some(ref key) = root_key {
            self.ast_cache.write().await.remove(key);
        }

        let foundry_config = self.foundry_config.read().await.clone();
        let ast_cache = self.ast_cache.clone();
        let client = self.client.clone();
        let text_cache_snapshot = self.text_cache.read().await.clone();

        tokio::spawn(async move {
            let Some(cache_key) = root_key else {
                return;
            };
            match crate::solc::solc_project_index(
                &foundry_config,
                Some(&client),
                Some(&text_cache_snapshot),
            )
            .await
            {
                Ok(ast_data) => {
                    let cached_build = Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                    let source_count = cached_build.nodes.len();
                    ast_cache
                        .write()
                        .await
                        .insert(cache_key.into(), cached_build);
                    client
                        .log_message(
                            MessageType::INFO,
                            format!("didDeleteFiles: re-indexed {} source files", source_count),
                        )
                        .await;
                }
                Err(e) => {
                    client
                        .log_message(
                            MessageType::WARNING,
                            format!("didDeleteFiles: re-index failed: {e}"),
                        )
                        .await;
                }
            }
        });
    }

    async fn will_create_files(
        &self,
        params: CreateFilesParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/willCreateFiles: {} file(s)", params.files.len()),
            )
            .await;
        if !self
            .settings
            .read()
            .await
            .file_operations
            .template_on_create
        {
            self.client
                .log_message(
                    MessageType::INFO,
                    "willCreateFiles: templateOnCreate disabled",
                )
                .await;
            return Ok(None);
        }
        self.client
            .log_message(
                MessageType::INFO,
                "willCreateFiles: skipping pre-create edits; scaffolding via didCreateFiles",
            )
            .await;
        Ok(None)
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("workspace/didCreateFiles: {} file(s)", params.files.len()),
            )
            .await;
        self.project_cache_dirty.store(true, Ordering::Release);
        {
            let mut changed = self.project_cache_changed_files.write().await;
            for file in &params.files {
                if let Ok(uri) = Url::parse(&file.uri)
                    && let Ok(path) = uri.to_file_path()
                {
                    changed.insert(path.to_string_lossy().to_string());
                }
            }
        }
        if !self
            .settings
            .read()
            .await
            .file_operations
            .template_on_create
        {
            self.client
                .log_message(
                    MessageType::INFO,
                    "didCreateFiles: templateOnCreate disabled",
                )
                .await;
            return;
        }

        let config = self.foundry_config.read().await;
        let solc_version = config.solc_version.clone();
        drop(config);

        // Generate scaffold and push via workspace/applyEdit for files that
        // are empty in both cache and on disk. This avoids prepending content
        // to already-populated files while keeping a fallback for clients that
        // don't apply willCreateFiles edits.
        let mut apply_edits: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        let mut staged_content: HashMap<String, String> = HashMap::new();
        let mut created_uris: Vec<String> = Vec::new();
        {
            let tc = self.text_cache.read().await;
            for file_create in &params.files {
                let uri = match Url::parse(&file_create.uri) {
                    Ok(u) => u,
                    Err(_) => continue,
                };
                let uri_str = uri.to_string();

                let open_has_content = tc
                    .get(&uri_str)
                    .map_or(false, |(_, c)| c.chars().any(|ch| !ch.is_whitespace()));
                let path = match uri.to_file_path() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let disk_has_content = std::fs::read_to_string(&path)
                    .map_or(false, |c| c.chars().any(|ch| !ch.is_whitespace()));

                // If an open buffer already has content, skip. If buffer is
                // open but empty, still apply scaffold to that buffer.
                if open_has_content {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "didCreateFiles: skip {} (open buffer already has content)",
                                uri_str
                            ),
                        )
                        .await;
                    continue;
                }

                // Also skip when the file already has content on disk.
                if disk_has_content {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "didCreateFiles: skip {} (disk file already has content)",
                                uri_str
                            ),
                        )
                        .await;
                    continue;
                }

                let content =
                    match file_operations::generate_scaffold(&uri, solc_version.as_deref()) {
                        Some(s) => s,
                        None => continue,
                    };

                staged_content.insert(uri_str, content.clone());
                created_uris.push(uri.to_string());

                apply_edits.entry(uri).or_default().push(TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    new_text: content,
                });
            }
        }

        if !apply_edits.is_empty() {
            {
                let mut pending = self.pending_create_scaffold.write().await;
                for uri in &created_uris {
                    pending.insert(uri.clone().into());
                }
            }

            let edit = WorkspaceEdit {
                changes: Some(apply_edits.clone()),
                document_changes: None,
                change_annotations: None,
            };
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "didCreateFiles: scaffolding {} empty file(s) via workspace/applyEdit",
                        apply_edits.len()
                    ),
                )
                .await;
            let apply_result = self.client.apply_edit(edit).await;
            let applied = apply_result.as_ref().is_ok_and(|r| r.applied);

            if applied {
                let mut tc = self.text_cache.write().await;
                for (uri_str, content) in staged_content {
                    tc.insert(uri_str.into(), (0, content));
                }
            } else {
                if let Ok(resp) = &apply_result {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!(
                                "didCreateFiles: applyEdit rejected (no disk fallback): {:?}",
                                resp.failure_reason
                            ),
                        )
                        .await;
                } else if let Err(e) = &apply_result {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("didCreateFiles: applyEdit failed (no disk fallback): {e}"),
                        )
                        .await;
                }
            }
        }

        // Refresh diagnostics for newly created files that now have in-memory
        // content (e.g. scaffold applied via willCreateFiles/didChange). This
        // clears stale diagnostics produced from the transient empty didOpen.
        for file_create in &params.files {
            let Ok(uri) = Url::parse(&file_create.uri) else {
                continue;
            };
            let (version, content) = {
                let tc = self.text_cache.read().await;
                match tc.get(&uri.to_string()) {
                    Some((v, c)) => (*v, c.clone()),
                    None => continue,
                }
            };
            if !content.chars().any(|ch| !ch.is_whitespace()) {
                continue;
            }
            self.on_change(TextDocumentItem {
                uri,
                version,
                text: content,
                language_id: "solidity".to_string(),
            })
            .await;
        }

        // Trigger background re-index so new symbols become discoverable.
        let root_key = self.project_cache_key().await;
        if let Some(ref key) = root_key {
            self.ast_cache.write().await.remove(key);
        }

        let foundry_config = self.foundry_config.read().await.clone();
        let ast_cache = self.ast_cache.clone();
        let client = self.client.clone();
        let text_cache_snapshot = self.text_cache.read().await.clone();

        tokio::spawn(async move {
            let Some(cache_key) = root_key else {
                return;
            };
            match crate::solc::solc_project_index(
                &foundry_config,
                Some(&client),
                Some(&text_cache_snapshot),
            )
            .await
            {
                Ok(ast_data) => {
                    let cached_build = Arc::new(crate::goto::CachedBuild::new(ast_data, 0));
                    let source_count = cached_build.nodes.len();
                    ast_cache
                        .write()
                        .await
                        .insert(cache_key.into(), cached_build);
                    client
                        .log_message(
                            MessageType::INFO,
                            format!("didCreateFiles: re-indexed {} source files", source_count),
                        )
                        .await;
                }
                Err(e) => {
                    client
                        .log_message(
                            MessageType::WARNING,
                            format!("didCreateFiles: re-index failed: {e}"),
                        )
                        .await;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        start_or_mark_project_cache_sync_pending, stop_project_cache_sync_worker_or_reclaim,
        take_project_cache_sync_pending, try_claim_project_cache_dirty,
        update_imports_on_delete_enabled,
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn update_imports_on_delete_enabled_defaults_true() {
        let s = crate::config::Settings::default();
        assert!(update_imports_on_delete_enabled(&s));
    }

    #[test]
    fn update_imports_on_delete_enabled_respects_false() {
        let mut s = crate::config::Settings::default();
        s.file_operations.update_imports_on_delete = false;
        assert!(!update_imports_on_delete_enabled(&s));
    }

    #[test]
    fn project_cache_sync_burst_only_first_starts_worker() {
        let pending = AtomicBool::new(false);
        let running = AtomicBool::new(false);

        assert!(start_or_mark_project_cache_sync_pending(&pending, &running));
        assert!(pending.load(Ordering::Acquire));
        assert!(running.load(Ordering::Acquire));

        // Subsequent save while running should only mark pending, not spawn.
        assert!(!start_or_mark_project_cache_sync_pending(
            &pending, &running
        ));
        assert!(pending.load(Ordering::Acquire));
        assert!(running.load(Ordering::Acquire));
    }

    #[test]
    fn project_cache_sync_take_pending_is_one_shot() {
        let pending = AtomicBool::new(true);
        assert!(take_project_cache_sync_pending(&pending));
        assert!(!pending.load(Ordering::Acquire));
        assert!(!take_project_cache_sync_pending(&pending));
    }

    #[test]
    fn project_cache_sync_worker_stop_or_reclaim_handles_race() {
        let pending = AtomicBool::new(false);
        let running = AtomicBool::new(true);

        // No new pending work: worker stops.
        assert!(!stop_project_cache_sync_worker_or_reclaim(
            &pending, &running
        ));
        assert!(!running.load(Ordering::Acquire));

        // Simulate a new save arriving right as worker tries to stop.
        pending.store(true, Ordering::Release);
        running.store(true, Ordering::Release);
        assert!(stop_project_cache_sync_worker_or_reclaim(
            &pending, &running
        ));
        assert!(running.load(Ordering::Acquire));
    }

    #[test]
    fn project_cache_dirty_claim_and_retry_cycle() {
        let dirty = AtomicBool::new(true);

        assert!(try_claim_project_cache_dirty(&dirty));
        assert!(!dirty.load(Ordering::Acquire));

        // Second claim without retry mark should fail.
        assert!(!try_claim_project_cache_dirty(&dirty));

        // Retry path marks dirty again.
        dirty.store(true, Ordering::Release);
        assert!(try_claim_project_cache_dirty(&dirty));
        assert!(!dirty.load(Ordering::Acquire));
    }
}
