# The Solidity Language Server

[![Crates.io](https://img.shields.io/crates/v/solidity-language-server)](https://crates.io/crates/solidity-language-server)
[![Tests](https://github.com/asyncswap/solidity-language-server/actions/workflows/test.yml/badge.svg)](https://github.com/asyncswap/solidity-language-server/actions/workflows/test.yml)
[![Release](https://github.com/asyncswap/solidity-language-server/actions/workflows/release.yml/badge.svg)](https://github.com/asyncswap/solidity-language-server/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/asyncswap/solidity-language-server)](https://github.com/asyncswap/solidity-language-server/releases/latest)
[![Telegram](https://img.shields.io/badge/Telegram-Join%20Chat-blue?logo=telegram)](https://t.me/+R1lW7xWJ55tlYzcx)

The fastest Solidity language server — optimized for low-latency go-to-definition/declaration, references, hover, completions, and file operations. See [benchmarks](https://github.com/mmsaki/lsp-bench).

## Install

```sh
curl -fsSL https://asyncswap.org/lsp/install.sh | sh
```

### Alternative: Install from Cargo

```sh
cargo install solidity-language-server
```

Or download directly from the [latest release](https://github.com/asyncswap/solidity-language-server/releases/latest) page.

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | solidity-language-server-aarch64-apple-darwin.tar.gz |
| macOS (Intel) | solidity-language-server-x86_64-apple-darwin.tar.gz |
| Windows | solidity-language-server-x86_64-pc-windows-msvc.zip |
| Linux | solidity-language-server-x86_64-unknown-linux-gnu.tar.gz |

## Benchmarks

Benchmarked against `v4-core` — `test/PoolManager.t.sol` (`v0.1.30`, p95 latency).

| Method | [mmsaki v0.1.30](https://github.com/asyncswap/solidity-language-server/releases/tag/v0.1.30) |
|--------|--------|
| initialize | 22.4ms ⚡ |
| textDocument/diagnostic | 2.8ms ⚡ |
| textDocument/semanticTokens/full/delta | 11.1ms ⚡ |
| textDocument/definition | 8.3ms ⚡ |
| textDocument/declaration | 1.6ms ⚡ |
| textDocument/hover | 7.3ms ⚡ |
| textDocument/references | 5.3ms ⚡ |
| textDocument/completion | 59.9ms ⚡ |
| textDocument/signatureHelp | 6.2ms ⚡ |
| textDocument/rename | 15.1ms ⚡ |
| textDocument/prepareRename | 0.2ms ⚡ |
| textDocument/documentSymbol | 7.7ms ⚡ |
| textDocument/documentHighlight | 8.4ms ⚡ |
| textDocument/documentLink | 2.6ms ⚡ |
| textDocument/formatting | 43.4ms ⚡ |
| textDocument/foldingRange | 10.4ms ⚡ |
| textDocument/selectionRange | 6.6ms ⚡ |
| textDocument/inlayHint | 11.4ms ⚡ |
| textDocument/semanticTokens/full | 18.5ms ⚡ |
| textDocument/semanticTokens/range | 8.9ms ⚡ |
| workspace/symbol | 7.2ms ⚡ |
| workspace/willRenameFiles | 240.6ms ⚡ |
| workspace/willCreateFiles | 1.6ms ⚡ |
| workspace/willDeleteFiles | 230.5ms ⚡ |
| workspace/executeCommand | 5.3ms ⚡ |
| textDocument/codeAction | 28.7ms ⚡ |

- Full benchmark report: [PoolManager.t.sol results](benchmarks/poolmanager-t/README.md)

## Docs

- [Documentation Site](https://solidity-language-server-docs.pages.dev) — canonical docs (Quickstart, setup, references, benchmarks)
- [Quickstart](https://solidity-language-server-docs.pages.dev/docs/quickstart)
- [Reference Index](https://solidity-language-server-docs.pages.dev/reference)
- [Neovim Setup](https://solidity-language-server-docs.pages.dev/setup/neovim)
- [FEATURES.md](FEATURES.md) — full LSP feature set and roadmap
- [CONTRIBUTING.md](CONTRIBUTING.md) — development setup, project structure, and how to contribute
- [CHANGELOG.md](CHANGELOG.md) — release history

## Editor Setup

Use the docs for complete editor-specific setup and config examples:

- [Neovim](https://solidity-language-server-docs.pages.dev/setup/neovim)
- [Helix](https://solidity-language-server-docs.pages.dev/setup/helix)
- [Zed](https://solidity-language-server-docs.pages.dev/setup/zed)
- [VS Code / Cursor](https://solidity-language-server-docs.pages.dev/setup/vscode)
- [Vim](https://solidity-language-server-docs.pages.dev/setup/vim)
- [Emacs](https://solidity-language-server-docs.pages.dev/setup/emacs)

## AI Agents

- [OpenCode](https://solidity-language-server-docs.pages.dev/agents/opencode) — direct LSP integration
- [Claude Code](https://solidity-language-server-docs.pages.dev/agents/claude) — LSP via [plugin](https://github.com/asyncswap/skills)
- [Codex](https://solidity-language-server-docs.pages.dev/agents/codex) — `AGENTS.md` + shell commands

Minimal LSP command:

```sh
solidity-language-server --stdio
```

## Verify Release Binaries

Release binaries are GPG-signed. Download `checksums-sha256.txt`, `checksums-sha256.txt.asc`, and [`public-key.asc`](public-key.asc) from the [release](https://github.com/asyncswap/solidity-language-server/releases/latest):

```sh
gpg --import public-key.asc
gpg --verify checksums-sha256.txt.asc checksums-sha256.txt
sha256sum -c checksums-sha256.txt
```
