# Imports and Navigation

This page is intentionally short and exists to prevent confusion between three related features:

1. **Go-to-definition on symbols** (`Identifier`, `MemberAccess`, etc.)
2. **Go-to-definition on import strings** (`import "./X.sol"`)
3. **Find references** (`textDocument/references`)

## What goes where

- Symbol goto and import-string goto are both handled in the goto implementation:
  - see [`goto.md`](./goto)
- Reference collection is handled in the references implementation:
  - see [`references.md`](./references)

## Important boundary

Import strings are treated as **navigation targets**, not symbol references.

That means:

- `import "./Pool.sol"` can navigate to the file via go-to-definition.
- `textDocument/references` does not return import-string literals as references for a declaration.

This boundary is deliberate in the current design so references remain declaration/usage based, while import paths remain file-navigation based.

## Import path completions

There is a third distinct behavior for import strings: **completions**.

When the cursor is inside an import string literal (e.g. `import "|"`), the server provides filesystem-based path completions — not symbol completions. This is handled in the completions pipeline, not in goto or references.

See [`completions.md`](./completions) for details on import path completion mode.
