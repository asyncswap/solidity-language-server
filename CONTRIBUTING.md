# Contributing

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Foundry](https://book.getfoundry.sh/getting-started/installation) (`forge` must be in your PATH)
- A Foundry project to test against (e.g. [Uniswap v4-core](https://github.com/Uniswap/v4-core))

## Setup

```sh
git clone https://github.com/mmsaki/solidity-language-server.git
cd solidity-language-server
cargo build --release
cargo test
```

## Project Structure

```
src/
├── main.rs              # CLI entry point, clap args, tracing setup
├── lib.rs               # Module declarations
├── lsp.rs               # LSP server, handler dispatch, capabilities
├── build.rs             # forge build + diagnostics
├── config.rs            # LSP settings deserialization
├── completion.rs        # Completion engine, chain resolution, using-for
├── file_operations.rs   # willCreate/willRename/willDelete file ops
├── folding.rs           # textDocument/foldingRange
├── goto.rs              # goto-definition, goto-declaration, AST node caching
├── highlight.rs         # textDocument/documentHighlight
├── hover.rs             # textDocument/hover, NatSpec, selectors, @inheritdoc
├── inlay_hints.rs       # textDocument/inlayHint
├── links.rs             # textDocument/documentLink
├── lint.rs              # forge lint diagnostics
├── project_cache.rs     # v2 on-disk shard cache, single-flight sync
├── references.rs        # find-references, Yul external refs
├── rename.rs            # prepare-rename, rename (including aliased imports)
├── runner.rs            # solc subprocess runner
├── selection.rs         # textDocument/selectionRange
├── semantic_tokens.rs   # textDocument/semanticTokens (full/range/delta)
├── solar_runner.rs      # experimental Solar parser backend
├── solc.rs              # solc invocation and output parsing
├── solc_ast/            # AST node types and visitor (12 sub-modules)
├── symbols.rs           # document/workspace symbols
├── types.rs             # shared type definitions
└── utils.rs             # shared utilities

docs/pages/          # Vocs docs content (quickstart, setup, reference, benchmarks)
build.rs             # Compile-time: git commit hash, OS, arch
```

## Running Tests

```sh
cargo test              # all tests (600 currently)
cargo test hover        # run hover tests only
cargo test completion   # run completion tests only
```

Tests use `poolmanager.json` (Uniswap v4 fixture) committed in the repo.

## Testing the LSP

Build and point your editor at the binary:

```sh
cargo build --release
# binary is at target/release/solidity-language-server
```

Or install locally:

```sh
cargo install --path .
```

Test against a real Foundry project — the LSP needs `foundry.toml` in the project root.

## Adding a New LSP Feature

1. **Create the module** — e.g. `src/feature.rs`
2. **Register in `src/lib.rs`** — add `pub mod feature;`
3. **Add capability in `src/lsp.rs`** — in the `initialize` handler's `ServerCapabilities`
4. **Add handler in `src/lsp.rs`** — implement the `LanguageServer` trait method
5. **Use `ast_cache`** — read from `self.ast_cache` instead of calling `self.compiler.ast()` directly
  6. **Write tests** — use `poolmanager.json` fixture, assert against known node IDs
7. **Add docs** — document behavior in `docs/pages/reference/*.md` (and update setup/quickstart pages if needed)

## AST Exploration

The compiler AST is the foundation. Use `jq` to explore:

```sh
# Find a node by ID
jq '.. | objects | select(.id == 1767)' poolmanager.json

# Find nodes by type
jq '[.. | objects | select(.nodeType == "FunctionDefinition")] | length' poolmanager.json

# Find nodes with a specific field
jq '.. | objects | select(.functionSelector != null) | {id, name, functionSelector}' poolmanager.json
```

See [documentation site](https://solidity-language-server-docs.pages.dev) and `docs/pages/reference/` for implementation-deep feature guides.

## Pull Requests

1. Create an issue describing the feature or bug
2. Branch from `main`: `git checkout -b feature/your-feature`
3. Make your changes with tests
4. Ensure `cargo test` passes and `cargo check` has no warnings
5. Open a PR referencing the issue

## Releases

Releases are automated. When a `v*` tag is pushed:

1. CI builds release binaries for macOS (arm64, x86_64), Linux (x86_64), and Windows (x86_64)
2. Generates SHA-256 checksums
3. GPG-signs the checksums
4. Creates a GitHub Release with all artifacts

Tag protection rules restrict who can push release tags.
