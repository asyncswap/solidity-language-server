# The Solidity Language Server

[![Crates.io](https://img.shields.io/crates/v/solidity-language-server)](https://crates.io/crates/solidity-language-server)
[![Tests](https://github.com/mmsaki/solidity-language-server/actions/workflows/test.yml/badge.svg)](https://github.com/mmsaki/solidity-language-server/actions/workflows/test.yml)
[![Release](https://github.com/mmsaki/solidity-language-server/actions/workflows/release.yml/badge.svg)](https://github.com/mmsaki/solidity-language-server/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/mmsaki/solidity-language-server)](https://github.com/mmsaki/solidity-language-server/releases/latest)
[![Telegram](https://img.shields.io/badge/Telegram-Join%20Chat-blue?logo=telegram)](https://t.me/+R1lW7xWJ55tlYzcx)

The fastest Solidity language server — optimized for low-latency go-to-definition/declaration, references, hover, completions, and file operations. See [benchmarks](https://github.com/mmsaki/lsp-bench).

## Install

```sh
cargo install solidity-language-server
```

Or download a pre-built binary from the [latest release](https://github.com/mmsaki/solidity-language-server/releases/latest).

## Benchmarks

Benchmarked against `v4-core` — `test/PoolManager.t.sol` (`v0.1.26`, p95 latency).

| Method | [mmsaki v0.1.26](https://github.com/mmsaki/solidity-language-server/releases/tag/v0.1.26) |
|--------|--------|
| initialize | 20.7ms ⚡ |
| textDocument/diagnostic | 2.2s ⚡ |
| textDocument/semanticTokens/full/delta | 9.8ms ⚡ |
| textDocument/definition | 6.7ms ⚡ |
| textDocument/declaration | 0.8ms ⚡ |
| textDocument/hover | 6.8ms ⚡ |
| textDocument/references | 4.9ms ⚡ |
| textDocument/completion | 0.2ms ⚡ |
| textDocument/signatureHelp | 6.0ms ⚡ |
| textDocument/rename | 6.5ms ⚡ |
| textDocument/prepareRename | 0.2ms ⚡ |
| textDocument/documentSymbol | 6.4ms ⚡ |
| textDocument/documentHighlight | 7.6ms ⚡ |
| textDocument/documentLink | 1.6ms ⚡ |
| textDocument/formatting | 19.3ms ⚡ |
| textDocument/foldingRange | 7.3ms ⚡ |
| textDocument/selectionRange | 6.0ms ⚡ |
| textDocument/inlayHint | 16.1ms ⚡ |
| textDocument/semanticTokens/full | 10.1ms ⚡ |
| textDocument/semanticTokens/range | 6.8ms ⚡ |
| workspace/symbol | 6.1ms ⚡ |
| workspace/willRenameFiles | 90.4ms ⚡ |
| workspace/willCreateFiles | 0.2ms |
| workspace/willDeleteFiles | 81.8ms ⚡ |

- Full benchmark reports:
- [Shop.sol results](benchmarks/shop/README.md)
- [Pool.sol results](benchmarks/pool/README.md)
- [PoolManager.t.sol results](benchmarks/poolmanager-t/README.md)

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

Minimal LSP command:

```sh
solidity-language-server --stdio
```

## Verify Release Binaries

Release binaries are GPG-signed. Download `checksums-sha256.txt`, `checksums-sha256.txt.asc`, and [`public-key.asc`](public-key.asc) from the [release](https://github.com/mmsaki/solidity-language-server/releases/latest):

```sh
gpg --import public-key.asc
gpg --verify checksums-sha256.txt.asc checksums-sha256.txt
sha256sum -c checksums-sha256.txt
```
