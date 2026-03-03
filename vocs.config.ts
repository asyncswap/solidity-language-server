import { defineConfig } from "vocs";

export default defineConfig({
  title: "Solidity Language Server",
  description: "Documentation and benchmarks for solidity-language-server",
  rootDir: "docs",
  sidebar: [
    {
      text: "Documentation",
      items: [
        { text: "Quickstart", link: "/docs/quickstart" },
        { text: "Features", link: "/docs/features" },
      ]
    },
    {
      text: "Setup",
      items: [
        { text: "Overview", link: "/setup" },
        { text: "Neovim", link: "/setup/neovim" },
        { text: "VS Code", link: "/setup/vscode" },
        { text: "Helix", link: "/setup/helix" },
        { text: "Zed", link: "/setup/zed" },
        { text: "Sublime Text", link: "/setup/sublime" },
        { text: "Emacs", link: "/setup/emacs" },
        { text: "Vim", link: "/setup/vim" }
      ]
    },
    {
      text: "LSP Benchmarks",
      items: [
        { text: "Overview", link: "/benchmarks/overview" },
        { text: "Shop.sol", link: "/benchmarks/reports/shop" },
        { text: "Pool.sol", link: "/benchmarks/reports/pool" },
        { text: "PoolManager.t.sol", link: "/benchmarks/reports/poolmanager-t" }
      ]
    },
    {
      text: "Reference",
      items: [
        { text: "Index", link: "/reference" },
        { text: "Go To Definition", link: "/reference/goto" },
        { text: "References", link: "/reference/references" },
        { text: "Hover", link: "/reference/hover" },
        { text: "Completions", link: "/reference/completions" },
        { text: "Symbols", link: "/reference/symbols" },
        { text: "Inlay Hints", link: "/reference/inlay-hints" },
        { text: "Signature Help", link: "/reference/signature-help" },
        { text: "Will Rename Files", link: "/reference/will-rename-files" },
        { text: "Imports and References", link: "/reference/imports-references" },
        { text: "Diagnostics", link: "/reference/diagnostics" },
        { text: "Code Actions", link: "/reference/code-actions" },
        { text: "Caches", link: "/reference/caches" },
        { text: "Profiling", link: "/reference/profiling" }
      ]
    },
    {
      text: "Blog",
      items: [
        { text: "All Posts", link: "/blog" },
        { text: "v0.1.29 Release", link: "/blog/v0-1-29-release" },
        { text: "v0.1.28 Release", link: "/blog/v0-1-28-release" },
        { text: "v0.1.27 Release", link: "/blog/v0-1-27-release" },
        { text: "v0.1.26 Release", link: "/blog/v0-1-26-release" },
        { text: "v0.1.25 Release", link: "/blog/v0-1-25-release" },
        { text: "v0.1.17 Release", link: "/blog/v0-1-17-release" },
        { text: "v0.1.14 Release", link: "/blog/v0-1-14-release" }
      ]
    }
  ],
  topNav: [
    { text: "CHANGELOG", link: "/changelog" },
    { text: "GitHub", link: "https://github.com/mmsaki/solidity-language-server" },
    { text: "Crates.io", link: "https://crates.io/crates/solidity-language-server" }
  ]
});
