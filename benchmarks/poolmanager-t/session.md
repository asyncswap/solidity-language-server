# Session Log — v4-core / test/PoolManager.t.sol

## initialize

**Request:** `initialize` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (16.0ms, 9.8 MB) — ok

<details>
<summary>Summary: <code>"ok"</code></summary>

```json
"ok"
```
</details>

**forge** (22.5ms, 28.5 MB) — ok

<details>
<summary>Summary: <code>"ok"</code></summary>

```json
"ok"
```
</details>

---

## textDocument/diagnostic

**Request:** `textDocument/diagnostic` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (0.4ms, 10.6 MB) — {"message":"lsp server initialized.","ty...

<details>
<summary>Summary: <code>{ message: "lsp server initialized.", type: 3 }</code></summary>

```json
{
  "message": "lsp server initialized.",
  "type": 3
}
```
</details>

**forge** (0.2ms, 28.6 MB) — {"message":"solar initialized","type":3}

<details>
<summary>Summary: <code>{ message: "solar initialized", type: 3 }</code></summary>

```json
{
  "message": "solar initialized",
  "type": 3
}
```
</details>

---

## textDocument/semanticTokens/full/delta

**Request:** `textDocument/semanticTokens/full/delta` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (10.8ms, 10.5 MB) — delta

<details>
<summary>Summary: <code>{ edits: [], resultId: "3" }</code></summary>

```json
{
  "edits": [],
  "resultId": "3"
}
```
</details>

**forge** (28.2 MB) — No such method textDocument/semanticToke...

---

## textDocument/definition

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/definition",
  "params": {
    "position": {
      "character": 51,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (60.3ms, 67.8 MB) — `TickMath.sol:9`

<details>
<summary>Summary: <code>{ range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } }, uri: "file:///Users/meek/developer/as...</code></summary>

```json
{
  "range": {
    "end": {
      "character": 16,
      "line": 9
    },
    "start": {
      "character": 8,
      "line": 9
    }
  },
  "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
}
```
</details>

**forge** (0.1ms, 197.9 MB) — `TickMath.sol:9`

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } }, uri: "file:///Users/meek/de...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 16,
        "line": 9
      },
      "start": {
        "character": 8,
        "line": 9
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
  }
]
```
</details>

---

## textDocument/declaration

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/declaration",
  "params": {
    "position": {
      "character": 51,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (8.2ms, 67.6 MB) — `TickMath.sol:9`

<details>
<summary>Summary: <code>{ range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } }, uri: "file:///Users/meek/developer/as...</code></summary>

```json
{
  "range": {
    "end": {
      "character": 16,
      "line": 9
    },
    "start": {
      "character": 8,
      "line": 9
    }
  },
  "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
}
```
</details>

**forge** (0.1ms, 193.3 MB) — `TickMath.sol:9`

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } }, uri: "file:///Users/meek/de...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 16,
        "line": 9
      },
      "start": {
        "character": 8,
        "line": 9
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
  }
]
```
</details>

---

## textDocument/hover

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/hover",
  "params": {
    "position": {
      "character": 29,
      "line": 96
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (6.9ms, 67.8 MB) — error PoolNotInitialized()

<details>
<summary>Summary: <code>{ contents: { kind: "markdown", value: "```solidity
error PoolNotInitialized()
```

Selector: `0x486aa307`

---
Throw...</code></summary>

```json
{
  "contents": {
    "kind": "markdown",
    "value": "```solidity\nerror PoolNotInitialized()\n```\n\nSelector: `0x486aa307`\n\n---\nThrown when trying to interact with a non-initialized pool"
  }
}
```
</details>

**forge** (0.1ms, 185.2 MB) — error PoolNotInitialized()

<details>
<summary>Summary: <code>{ contents: { kind: "markdown", value: "```solidity
error PoolNotInitialized()
```

Thrown when trying to interact wi...</code></summary>

```json
{
  "contents": {
    "kind": "markdown",
    "value": "```solidity\nerror PoolNotInitialized()\n```\n\nThrown when trying to interact with a non-initialized pool"
  },
  "range": {
    "end": {
      "character": 47,
      "line": 96
    },
    "start": {
      "character": 29,
      "line": 96
    }
  }
}
```
</details>

---

## textDocument/references

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/references",
  "params": {
    "context": {
      "includeDeclaration": true
    },
    "position": {
      "character": 32,
      "line": 102
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (23.7ms, 68.7 MB) — 67 references

<details>
<summary>Summary: <code>Array(67) [{ range: { end: { character: 28, line: 70 }, start: { character: 12, line: 70 } }, uri: "file:///Users/mee...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 28,
        "line": 70
      },
      "start": {
        "character": 12,
        "line": 70
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/utils/Deployers.sol"
  },
  {
    "range": {
      "end": {
        "character": 24,
        "line": 206
      },
      "start": {
        "character": 8,
        "line": 206
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/utils/Deployers.sol"
  },
  {
    "range": {
      "end": {
        "character": 44,
        "line": 822
      },
      "start": {
        "character": 28,
        "line": 822
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 48,
        "line": 102
      },
      "start": {
        "character": 32,
        "line": 102
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 62,
        "line": 107
      },
      "start": {
        "character": 46,
        "line": 107
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  "... 62 more (67 total)"
]
```
</details>

**forge** (0.3ms, 189.4 MB) — 67 references

<details>
<summary>Summary: <code>Array(67) [{ range: { end: { character: 62, line: 64 }, start: { character: 46, line: 64 } }, uri: "file:///Users/mee...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 62,
        "line": 64
      },
      "start": {
        "character": 46,
        "line": 64
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/NoDelegateCall.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 62,
        "line": 67
      },
      "start": {
        "character": 46,
        "line": 67
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/NoDelegateCall.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 62,
        "line": 97
      },
      "start": {
        "character": 46,
        "line": 97
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 48,
        "line": 102
      },
      "start": {
        "character": 32,
        "line": 102
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  {
    "range": {
      "end": {
        "character": 62,
        "line": 107
      },
      "start": {
        "character": 46,
        "line": 107
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  },
  "... 62 more (67 total)"
]
```
</details>

---

## textDocument/completion

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/completion",
  "params": {
    "context": {
      "triggerCharacter": ".",
      "triggerKind": 2
    },
    "position": {
      "character": 35,
      "line": 815
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (50.9ms, 66.7 MB) — 23 items (amount0, amount1, checkTicks)

<details>
<summary>Summary: <code>{ isIncomplete: false, items: Array(23) [{ detail: "function amount0(BalanceDelta balanceDelta) internal pure returns...</code></summary>

```json
{
  "isIncomplete": false,
  "items": [
    {
      "detail": "function amount0(BalanceDelta balanceDelta) internal pure returns (int128 _amount0)",
      "kind": 3,
      "label": "amount0"
    },
    {
      "detail": "function amount1(BalanceDelta balanceDelta) internal pure returns (int128 _amount1)",
      "kind": 3,
      "label": "amount1"
    },
    {
      "detail": "function checkTicks(int24 tickLower, int24 tickUpper) private pure",
      "kind": 3,
      "label": "checkTicks"
    },
    {
      "detail": "function initialize(struct Pool.State storage self, uint160 sqrtPriceX96, uint24 lpFee) internal returns (int24 tick)",
      "kind": 3,
      "label": "initialize"
    },
    {
      "detail": "function setProtocolFee(struct Pool.State storage self, uint24 protocolFee) internal",
      "kind": 3,
      "label": "setProtocolFee"
    },
    "... 18 more (23 total)"
  ]
}
```
</details>

**forge** (0.3ms, 200.1 MB) — 2 items (amount0, amount1)

<details>
<summary>Summary: <code>Array(2) [{ data: Array(6) [1, "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/t...", 60...</code></summary>

```json
[
  {
    "data": [
      1,
      "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/types/BalanceDelta.sol",
      60,
      13,
      60,
      "... 1 more (6 total)"
    ],
    "detail": "BalanceDeltaLibrary",
    "kind": 2,
    "label": "amount0"
  },
  {
    "data": [
      1,
      "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/types/BalanceDelta.sol",
      66,
      13,
      66,
      "... 1 more (6 total)"
    ],
    "detail": "BalanceDeltaLibrary",
    "kind": 2,
    "label": "amount1"
  }
]
```
</details>

---

## textDocument/signatureHelp

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/signatureHelp",
  "params": {
    "position": {
      "character": 37,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (6.1ms, 66.5 MB) — function bound(uint256 x, uint256 min, uint256 max...

<details>
<summary>Summary: <code>{ activeParameter: 0, activeSignature: 0, signatures: Array(1) [{ activeParameter: 0, label: "function bound(uint256 ...</code></summary>

```json
{
  "activeParameter": 0,
  "activeSignature": 0,
  "signatures": [
    {
      "activeParameter": 0,
      "label": "function bound(uint256 x, uint256 min, uint256 max) internal pure returns (uint256 result)",
      "parameters": [
        {
          "label": [
            15,
            24
          ]
        },
        {
          "label": [
            26,
            37
          ]
        },
        {
          "label": [
            39,
            50
          ]
        }
      ]
    }
  ]
}
```
</details>

**forge** (0.1ms, 197.3 MB) — function bound(uint256 x, uint256 min, uint256 max...

<details>
<summary>Summary: <code>{ activeParameter: 0, activeSignature: 0, signatures: Array(2) [{ label: "function bound(uint256 x, uint256 min, uint...</code></summary>

```json
{
  "activeParameter": 0,
  "activeSignature": 0,
  "signatures": [
    {
      "label": "function bound(uint256 x, uint256 min, uint256 max) internal pure returns (uint256 result)",
      "parameters": [
        {
          "label": "uint256 x"
        },
        {
          "label": "uint256 min"
        },
        {
          "label": "uint256 max"
        }
      ]
    },
    {
      "label": "function bound(int256 x, int256 min, int256 max) internal pure returns (int256 result)",
      "parameters": [
        {
          "label": "int256 x"
        },
        {
          "label": "int256 min"
        },
        {
          "label": "int256 max"
        }
      ]
    }
  ]
}
```
</details>

---

## textDocument/rename

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/rename",
  "params": {
    "newName": "__lsp_bench_rename__",
    "position": {
      "character": 18,
      "line": 254
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (23.3ms, 69.3 MB) — 9 edits in 1 files

<details>
<summary>Summary: <code>{ changes: { file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol: Array(9)...</code></summary>

```json
{
  "changes": {
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol": [
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 17,
            "line": 266
          },
          "start": {
            "character": 8,
            "line": 266
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 81,
            "line": 266
          },
          "start": {
            "character": 72,
            "line": 266
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 42,
            "line": 259
          },
          "start": {
            "character": 33,
            "line": 259
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 42,
            "line": 266
          },
          "start": {
            "character": 33,
            "line": 266
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 27,
            "line": 254
          },
          "start": {
            "character": 18,
            "line": 254
          }
        }
      },
      "... 4 more (9 total)"
    ]
  }
}
```
</details>

**forge** (0.7ms, 197.2 MB) — 9 edits in 1 files

<details>
<summary>Summary: <code>{ changes: { file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol: Array(9)...</code></summary>

```json
{
  "changes": {
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol": [
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 27,
            "line": 254
          },
          "start": {
            "character": 18,
            "line": 254
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 57,
            "line": 256
          },
          "start": {
            "character": 48,
            "line": 256
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 17,
            "line": 258
          },
          "start": {
            "character": 8,
            "line": 258
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 42,
            "line": 258
          },
          "start": {
            "character": 33,
            "line": 258
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 17,
            "line": 259
          },
          "start": {
            "character": 8,
            "line": 259
          }
        }
      },
      "... 4 more (9 total)"
    ]
  }
}
```
</details>

---

## textDocument/prepareRename

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/prepareRename",
  "params": {
    "position": {
      "character": 51,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (0.3ms, 67.3 MB) — ready (line 116)

<details>
<summary>Summary: <code>{ end: { character: 59, line: 116 }, start: { character: 51, line: 116 } }</code></summary>

```json
{
  "end": {
    "character": 59,
    "line": 116
  },
  "start": {
    "character": 51,
    "line": 116
  }
}
```
</details>

**forge** (187.6 MB) — cannot rename this symbol because worksp...

---

## textDocument/documentSymbol

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/documentSymbol",
  "params": {
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (7.3ms, 10.3 MB) — 35 symbols

<details>
<summary>Summary: <code>Array(35) [{ kind: 15, name: "pragma solidity ^0.8.20", range: { end: { character: 24, line: 1 }, start: { character:...</code></summary>

```json
[
  {
    "kind": 15,
    "name": "pragma solidity ^0.8.20",
    "range": {
      "end": {
        "character": 24,
        "line": 1
      },
      "start": {
        "character": 0,
        "line": 1
      }
    },
    "selectionRange": {
      "end": {
        "character": 24,
        "line": 1
      },
      "start": {
        "character": 0,
        "line": 1
      }
    }
  },
  {
    "kind": 2,
    "name": "import \"forge-std/Test.sol\"",
    "range": {
      "end": {
        "character": 40,
        "line": 3
      },
      "start": {
        "character": 0,
        "line": 3
      }
    },
    "selectionRange": {
      "end": {
        "character": 40,
        "line": 3
      },
      "start": {
        "character": 0,
        "line": 3
      }
    }
  },
  {
    "kind": 2,
    "name": "import \"../src/interfaces/IHooks.sol\"",
    "range": {
      "end": {
        "character": 52,
        "line": 4
      },
      "start": {
        "character": 0,
        "line": 4
      }
    },
    "selectionRange": {
      "end": {
        "character": 52,
        "line": 4
      },
      "start": {
        "character": 0,
        "line": 4
      }
    }
  },
  {
    "kind": 2,
    "name": "import \"../src/libraries/Hooks.sol\"",
    "range": {
      "end": {
        "character": 49,
        "line": 5
      },
      "start": {
        "character": 0,
        "line": 5
      }
    },
    "selectionRange": {
      "end": {
        "character": 49,
        "line": 5
      },
      "start": {
        "character": 0,
        "line": 5
      }
    }
  },
  {
    "kind": 2,
    "name": "import \"../src/interfaces/IPoolManager.sol\"",
    "range": {
      "end": {
        "character": 64,
        "line": 6
      },
      "start": {
        "character": 0,
        "line": 6
      }
    },
    "selectionRange": {
      "end": {
        "character": 64,
        "line": 6
      },
      "start": {
        "character": 0,
        "line": 6
      }
    }
  },
  "... 30 more (35 total)"
]
```
</details>

**forge** (0.7ms, 28.3 MB) — 258 symbols

<details>
<summary>Summary: <code>Array(258) [{ kind: 5, location: { range: { end: { character: 1, line: 1261 }, start: { character: 0, line: 37 } }, u...</code></summary>

```json
[
  {
    "kind": 5,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 1261
        },
        "start": {
          "character": 0,
          "line": 37
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    },
    "name": "PoolManagerTest"
  },
  {
    "containerName": "PoolManagerTest",
    "kind": 24,
    "location": {
      "range": {
        "end": {
          "character": 27,
          "line": 45
        },
        "start": {
          "character": 4,
          "line": 45
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    },
    "name": "UnlockCallback"
  },
  {
    "containerName": "PoolManagerTest",
    "kind": 24,
    "location": {
      "range": {
        "end": {
          "character": 62,
          "line": 46
        },
        "start": {
          "character": 4,
          "line": 46
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    },
    "name": "ProtocolFeeControllerUpdated"
  },
  {
    "containerName": "ProtocolFeeControllerUpdated",
    "kind": 13,
    "location": {
      "range": {
        "end": {
          "character": 60,
          "line": 46
        },
        "start": {
          "character": 39,
          "line": 46
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    },
    "name": "feeController"
  },
  {
    "containerName": "PoolManagerTest",
    "kind": 24,
    "location": {
      "range": {
        "end": {
          "character": 6,
          "line": 54
        },
        "start": {
          "character": 4,
          "line": 47
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    },
    "name": "ModifyLiquidity"
  },
  "... 253 more (258 total)"
]
```
</details>

---

## textDocument/documentHighlight

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/documentHighlight",
  "params": {
    "position": {
      "character": 51,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (7.4ms, 69.0 MB) — [{"kind":2,"range":{"end":{"character":1...

<details>
<summary>Summary: <code>Array(19) [{ kind: 2, range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } } }, { kind: 2, rang...</code></summary>

```json
[
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 16,
        "line": 9
      },
      "start": {
        "character": 8,
        "line": 9
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 59,
        "line": 116
      },
      "start": {
        "character": 51,
        "line": 116
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 84,
        "line": 116
      },
      "start": {
        "character": 76,
        "line": 116
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 59,
        "line": 132
      },
      "start": {
        "character": 51,
        "line": 132
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 84,
        "line": 132
      },
      "start": {
        "character": 76,
        "line": 132
      }
    }
  },
  "... 14 more (19 total)"
]
```
</details>

**forge** (0.1ms, 192.1 MB) — [{"kind":2,"range":{"end":{"character":1...

<details>
<summary>Summary: <code>Array(19) [{ kind: 2, range: { end: { character: 16, line: 9 }, start: { character: 8, line: 9 } } }, { kind: 2, rang...</code></summary>

```json
[
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 16,
        "line": 9
      },
      "start": {
        "character": 8,
        "line": 9
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 59,
        "line": 116
      },
      "start": {
        "character": 51,
        "line": 116
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 84,
        "line": 116
      },
      "start": {
        "character": 76,
        "line": 116
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 59,
        "line": 132
      },
      "start": {
        "character": 51,
        "line": 132
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 84,
        "line": 132
      },
      "start": {
        "character": 76,
        "line": 132
      }
    }
  },
  "... 14 more (19 total)"
]
```
</details>

---

## textDocument/documentLink

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/documentLink",
  "params": {
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (1.7ms, 67.0 MB) — 33 links

<details>
<summary>Summary: <code>Array(33) [{ range: { end: { character: 38, line: 3 }, start: { character: 20, line: 3 } }, target: "file:///Users/me...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 38,
        "line": 3
      },
      "start": {
        "character": 20,
        "line": 3
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Test.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Test.sol"
  },
  {
    "range": {
      "end": {
        "character": 50,
        "line": 4
      },
      "start": {
        "character": 22,
        "line": 4
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IHooks.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IHooks.sol"
  },
  {
    "range": {
      "end": {
        "character": 47,
        "line": 5
      },
      "start": {
        "character": 21,
        "line": 5
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/Hooks.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/Hooks.sol"
  },
  {
    "range": {
      "end": {
        "character": 62,
        "line": 6
      },
      "start": {
        "character": 28,
        "line": 6
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IPoolManager.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IPoolManager.sol"
  },
  {
    "range": {
      "end": {
        "character": 64,
        "line": 7
      },
      "start": {
        "character": 29,
        "line": 7
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IProtocolFees.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IProtocolFees.sol"
  },
  "... 28 more (33 total)"
]
```
</details>

**forge** (0.1ms, 191.2 MB) — 33 links

<details>
<summary>Summary: <code>Array(33) [{ range: { end: { character: 39, line: 3 }, start: { character: 19, line: 3 } }, target: "file:///Users/me...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 39,
        "line": 3
      },
      "start": {
        "character": 19,
        "line": 3
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Test.sol"
  },
  {
    "range": {
      "end": {
        "character": 51,
        "line": 4
      },
      "start": {
        "character": 21,
        "line": 4
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IHooks.sol"
  },
  {
    "range": {
      "end": {
        "character": 48,
        "line": 5
      },
      "start": {
        "character": 20,
        "line": 5
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/Hooks.sol"
  },
  {
    "range": {
      "end": {
        "character": 63,
        "line": 6
      },
      "start": {
        "character": 27,
        "line": 6
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IPoolManager.sol"
  },
  {
    "range": {
      "end": {
        "character": 65,
        "line": 7
      },
      "start": {
        "character": 28,
        "line": 7
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/interfaces/IProtocolFees.sol"
  },
  "... 28 more (33 total)"
]
```
</details>

---

## textDocument/formatting

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/formatting",
  "params": {
    "options": {
      "insertSpaces": true,
      "tabSize": 4
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (32.4ms, 10.4 MB) — 1 edits

<details>
<summary>Summary: <code>Array(1) [{ newText: "// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test...", range: { end...</code></summary>

```json
[
  {
    "newText": "// SPDX-License-Identifier: UNLICENSED\npragma solidity ^0.8.20;\n\nimport {Test} from \"forge-std/Test.sol\";\nimport {IHooks} from \"../src/interfaces/IHooks.sol\";\nimport {Hooks} from \"../src/libraries/Hooks.sol\";\nimport {IPoolManager} from \"../src/interfaces/IPoolManager.sol\";\nimport {IProtocolFees} from \"../src/interfaces/IProtocolFees.sol\";\nimport {PoolManager} from \"../src/PoolManager.sol\";\nimport {TickMath} from \"../src/libraries/TickMath.sol\";\nimport {Pool} from \"../src/libraries/Pool.sol\";\nimport {Deployers} from \"./utils/Deployers.sol\";\nimport {Currency, CurrencyLibrary} from \"../src/types/Currency.sol\";\nimport {MockHooks} from \"../src/test/MockHooks.sol\";\nimport {MockContract} from \"../src/test/MockContract.sol\";\nimport {EmptyTestHooks} from \"../src/test/EmptyTestHooks.sol\";\nimport {PoolKey} from \"../src/types/PoolKey.sol\";\nimport {ModifyLiquidityParams, SwapParams} from \"../src/types/PoolOperation.sol\";\nimport {PoolModifyLiquidityTest} from \"../src/test/PoolModifyLiquidityTest.sol\";\nimport {BalanceDelta, BalanceDeltaLibrary} from \"../src/types/BalanceDelta.sol\";\nimport {PoolSwapTest} from \"../src/test/PoolSwapTest.sol\";\nimport {TestInvalidERC20} from \"../src/test/TestInvalidERC20.sol\";\nimport {PoolEmptyUnlockTest} from \"../src/test/PoolEmptyUnlockTest.sol\";\nimport {Action} from \"../src/test/PoolNestedActionsTest.sol\";\nimport {PoolId} from \"../src/types/PoolId.sol\";\nimport {LPFeeLibrary} from \"../src/libraries/LPFeeLibrary.sol\";\nimport {Position} from \"../src/libraries/Position.sol\";\nimport {Constants} from \"./utils/Constants.sol\";\nimport {SafeCast} from \"../src/libraries/SafeCast.sol\";\nimport {AmountHelpers} from \"./utils/AmountHelpers.sol\";\nimport {ProtocolFeeLibrary} from \"../src/libraries/ProtocolFeeLibrary.sol\";\nimport {IProtocolFees} from \"../src/interfaces/IProtocolFees.sol\";\nimport {StateLibrary} from \"../src/libraries/StateLibrary.sol\";\nimport {TransientStateLibrary} from \"../src/libraries/TransientStateLibrary.sol\";\nimport {Actions} from \"../src/test/ActionsRouter.sol\";\nimport {CustomRevert} from \"../src/libraries/CustomRevert.sol\";\n\ncontract PoolManagerTest is Test, Deployers {\n    using Hooks for IHooks;\n    using LPFeeLibrary for uint24;\n    using SafeCast for *;\n    using ProtocolFeeLibrary for uint24;\n    using StateLibrary for IPoolManager;\n    using TransientStateLibrary for IPoolManager;\n\n    event UnlockCallback();\n    event ProtocolFeeControllerUpdated(address feeController);\n    event ModifyLiquidity(\n        PoolId indexed poolId,\n        address indexed sender,\n        int24 tickLower,\n        int24 tickUpper,\n        int256 liquidityDelta,\n        bytes32 salt\n    );\n    event Swap(\n        PoolId indexed poolId,\n        address indexed sender,\n        int128 amount0,\n        int128 amount1,\n        uint160 sqrtPriceX96,\n        uint128 liquidity,\n        int24 tick,\n        uint24 fee\n    );\n\n    event Donate(PoolId indexed id, address indexed sender, uint256 amount0, uint256 amount1);\n\n    event Transfer(\n        address caller, address indexed sender, address indexed receiver, uint256 indexed id, uint256 amount\n    );\n\n    PoolEmptyUnlockTest emptyUnlockRouter;\n\n    uint24 constant MAX_PROTOCOL_FEE_BOTH_TOKENS = (1000 << 12) | 1000; // 1000 1000\n\n    address recipientAddress = makeAddr(\"recipientAddress\");\n\n    function setUp() public {\n        initializeManagerRoutersAndPoolsWithLiq(IHooks(address(0)));\n\n        emptyUnlockRouter = new PoolEmptyUnlockTest(manager);\n    }\n\n    function test_bytecodeSize() public {\n        vm.snapshotValue(\"poolManager bytecode size\", address(manager).code.length);\n    }\n\n    function test_initcodeHash() public {\n        vm.snapshotValue(\n            \"poolManager initcode hash (without constructor params, as uint256)\",\n            uint256(keccak256(type(PoolManager).creationCode))\n        );\n    }\n\n    function test_addLiquidity_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        modifyLiquidityRouter.modifyLiquidity(uninitializedKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.modifyLiquidity(uninitializedKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        modifyLiquidityRouter.modifyLiquidity(uninitializedKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsForNativeTokensIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            nativeKey.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsForNativeTokensIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            nativeKey.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsWithHooksIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        address payable mockAddr =\n            payable(address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPool(currency0, currency1, IHooks(mockAddr), 3000, sqrtPriceX96);\n\n        BalanceDelta balanceDelta = modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeAddLiquidity.selector;\n        bytes memory beforeParams = abi.encode(address(modifyLiquidityRouter), key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        bytes32 afterSelector = MockHooks.afterAddLiquidity.selector;\n        bytes memory afterParams = abi.encode(\n            address(modifyLiquidityRouter),\n            key,\n            LIQUIDITY_PARAMS,\n            balanceDelta,\n            BalanceDeltaLibrary.ZERO_DELTA,\n            ZERO_BYTES\n        );\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_removeLiquidity_succeedsWithHooksIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        address payable mockAddr =\n            payable(address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPool(currency0, currency1, IHooks(mockAddr), 3000, sqrtPriceX96);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        BalanceDelta balanceDelta = modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeRemoveLiquidity.selector;\n        bytes memory beforeParams = abi.encode(address(modifyLiquidityRouter), key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        bytes32 afterSelector = MockHooks.afterRemoveLiquidity.selector;\n        bytes memory afterParams = abi.encode(\n            address(modifyLiquidityRouter),\n            key,\n            REMOVE_LIQUIDITY_PARAMS,\n            balanceDelta,\n            BalanceDeltaLibrary.ZERO_DELTA,\n            ZERO_BYTES\n        );\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_addLiquidity_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterAddLiquidity.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeAddLiquidity hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        // Fail at afterAddLiquidity hook.\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, mockHooks.beforeAddLiquidity.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterRemoveLiquidity.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeRemoveLiquidity hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        // Fail at afterRemoveLiquidity hook.\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, mockHooks.beforeRemoveLiquidity.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, mockHooks.beforeAddLiquidity.selector);\n        mockHooks.setReturnValue(mockHooks.afterAddLiquidity.selector, mockHooks.afterAddLiquidity.selector);\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, mockHooks.beforeRemoveLiquidity.selector);\n        mockHooks.setReturnValue(mockHooks.afterRemoveLiquidity.selector, mockHooks.afterRemoveLiquidity.selector);\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            REMOVE_LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_6909() public {\n        // convert test tokens into ERC6909 claims\n        claimsRouter.deposit(currency0, address(this), 10_000e18);\n        claimsRouter.deposit(currency1, address(this), 10_000e18);\n        assertEq(manager.balanceOf(address(this), currency0.toId()), 10_000e18);\n        assertEq(manager.balanceOf(address(this), currency1.toId()), 10_000e18);\n\n        uint256 currency0BalanceBefore = currency0.balanceOfSelf();\n        uint256 currency1BalanceBefore = currency1.balanceOfSelf();\n        uint256 currency0PMBalanceBefore = currency0.balanceOf(address(manager));\n        uint256 currency1PMBalanceBefore = currency1.balanceOf(address(manager));\n\n        // allow liquidity router to burn our 6909 tokens\n        manager.setOperator(address(modifyLiquidityRouter), true);\n\n        // add liquidity with 6909: settleUsingBurn=true, takeClaims=true (unused)\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES, true, true);\n\n        assertLt(manager.balanceOf(address(this), currency0.toId()), 10_000e18);\n        assertLt(manager.balanceOf(address(this), currency1.toId()), 10_000e18);\n\n        // ERC20s are unspent\n        assertEq(currency0.balanceOfSelf(), currency0BalanceBefore);\n        assertEq(currency1.balanceOfSelf(), currency1BalanceBefore);\n\n        // PoolManager did not receive net-new ERC20s\n        assertEq(currency0.balanceOf(address(manager)), currency0PMBalanceBefore);\n        assertEq(currency1.balanceOf(address(manager)), currency1PMBalanceBefore);\n    }\n\n    function test_removeLiquidity_6909() public {\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        assertEq(manager.balanceOf(address(this), currency0.toId()), 0);\n        assertEq(manager.balanceOf(address(this), currency1.toId()), 0);\n\n        uint256 currency0BalanceBefore = currency0.balanceOfSelf();\n        uint256 currency1BalanceBefore = currency1.balanceOfSelf();\n        uint256 currency0PMBalanceBefore = currency0.balanceOf(address(manager));\n        uint256 currency1PMBalanceBefore = currency1.balanceOf(address(manager));\n\n        // remove liquidity as 6909: settleUsingBurn=true (unused), takeClaims=true\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES, true, true);\n\n        assertTrue(manager.balanceOf(address(this), currency0.toId()) > 0);\n        assertTrue(manager.balanceOf(address(this), currency1.toId()) > 0);\n\n        // ERC20s are unspent\n        assertEq(currency0.balanceOfSelf(), currency0BalanceBefore);\n        assertEq(currency1.balanceOfSelf(), currency1BalanceBefore);\n\n        // PoolManager did lose ERC-20s\n        assertEq(currency0.balanceOf(address(manager)), currency0PMBalanceBefore);\n        assertEq(currency1.balanceOf(address(manager)), currency1PMBalanceBefore);\n    }\n\n    function test_addLiquidity_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple addLiquidity\");\n    }\n\n    function test_addLiquidity_secondAdditionSameRange_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple addLiquidity second addition same range\");\n    }\n\n    function test_removeLiquidity_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        // add some liquidity to remove\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n\n        uniqueParams.liquidityDelta *= -1;\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple removeLiquidity\");\n    }\n\n    function test_removeLiquidity_someLiquidityRemains_gas() public {\n        // add double the liquidity to remove\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n\n        uniqueParams.liquidityDelta /= -2;\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple removeLiquidity some liquidity remains\");\n    }\n\n    function test_addLiquidity_succeeds() public {\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeeds() public {\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_withNative_gas() public {\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"addLiquidity with native token\");\n    }\n\n    function test_removeLiquidity_withNative_gas() public {\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"removeLiquidity with native token\");\n    }\n\n    function test_addLiquidity_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"addLiquidity with empty hook\");\n    }\n\n    function test_removeLiquidity_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"removeLiquidity with empty hook\");\n    }\n\n    function test_swap_failsIfNotInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        key.fee = 100;\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: sqrtPriceX96});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsIfInitialized() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(\n            key.toId(), address(swapRouter), int128(-100), int128(98), 79228162514264329749955861424, 1e18, -1, 3000\n        );\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.swap(key, SWAP_PARAMS, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithNativeTokensIfInitialized() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(\n            nativeKey.toId(),\n            address(swapRouter),\n            int128(-100),\n            int128(98),\n            79228162514264329749955861424,\n            1e18,\n            -1,\n            3000\n        );\n\n        swapRouter.swap{value: 100}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithHooksIfInitialized() public {\n        address payable mockAddr = payable(address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, IHooks(mockAddr), 3000, SQRT_PRICE_1_1);\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        BalanceDelta balanceDelta = swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeSwap.selector;\n        bytes memory beforeParams = abi.encode(address(swapRouter), key, SWAP_PARAMS, ZERO_BYTES);\n\n        bytes32 afterSelector = MockHooks.afterSwap.selector;\n        bytes memory afterParams = abi.encode(address(swapRouter), key, SWAP_PARAMS, balanceDelta, ZERO_BYTES);\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_swap_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: 10, sqrtPriceLimitX96: SQRT_PRICE_1_2});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterSwap.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeSwap hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n\n        // Fail at afterSwap hook.\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, mockHooks.beforeSwap.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: -10, sqrtPriceLimitX96: SQRT_PRICE_1_2});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, mockHooks.beforeSwap.selector);\n        mockHooks.setReturnValue(mockHooks.afterSwap.selector, mockHooks.afterSwap.selector);\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(key.toId(), address(swapRouter), -10, 8, 79228162514264336880490487708, 1e18, -1, 100);\n\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeeds() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_gas() public {\n        swapRouterNoChecks.swap(key, SWAP_PARAMS);\n        vm.snapshotGasLastCall(\"simple swap\");\n    }\n\n    function test_swap_withNative_succeeds() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap{value: 100}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_withNative_gas() public {\n        swapRouterNoChecks.swap{value: 100}(nativeKey, SWAP_PARAMS);\n        vm.snapshotGasLastCall(\"simple swap with native\");\n    }\n\n    function test_swap_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap with hooks\");\n    }\n\n    function test_swap_mint6909IfOutputNotTaken_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(0), address(this), CurrencyLibrary.toId(currency1), 98);\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap mint output as 6909\");\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(currency1));\n        assertEq(erc6909Balance, 98);\n    }\n\n    function test_swap_mint6909IfNativeOutputNotTaken_gas() public {\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_2_1});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(0), address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 98\n        );\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap mint native output as 6909\");\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 98);\n    }\n\n    function test_swap_burn6909AsInput_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(0), address(this), CurrencyLibrary.toId(currency1), 98);\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), uint256(uint160(Currency.unwrap(currency1))));\n        assertEq(erc6909Balance, 98);\n\n        // give permission for swapRouter to burn the 6909s\n        manager.setOperator(address(swapRouter), true);\n\n        // swap from currency1 to currency0 again, using 6909s as input tokens\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: 25, sqrtPriceLimitX96: SQRT_PRICE_4_1});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: true});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(this), address(0), CurrencyLibrary.toId(currency1), 27);\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap burn 6909 for input\");\n\n        erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(currency1));\n        assertEq(erc6909Balance, 71);\n    }\n\n    function test_swap_burnNative6909AsInput_gas() public {\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_2_1});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(0), address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 98\n        );\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 98);\n\n        // give permission for swapRouter to burn the 6909s\n        manager.setOperator(address(swapRouter), true);\n\n        // swap from currency0 to currency1, using 6909s as input tokens\n        params = SwapParams({zeroForOne: true, amountSpecified: 25, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: true});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(this), address(0), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 27\n        );\n        // don't have to send in native currency since burning 6909 for input\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap burn native 6909 for input\");\n\n        erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 71);\n    }\n\n    function test_swap_againstLiquidity_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap against liquidity\");\n    }\n\n    function test_swap_againstLiqWithNative_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap{value: 1 ether}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n\n        swapRouter.swap{value: 1 ether}(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap against liquidity with native token\");\n    }\n\n    function test_swap_accruesProtocolFees(uint16 protocolFee0, uint16 protocolFee1, int256 amountSpecified) public {\n        protocolFee0 = uint16(bound(protocolFee0, 0, 1000));\n        protocolFee1 = uint16(bound(protocolFee1, 0, 1000));\n        vm.assume(amountSpecified != 0);\n\n        uint24 protocolFee = (uint24(protocolFee1) << 12) | uint24(protocolFee0);\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, protocolFee);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, protocolFee);\n\n        // Add liquidity - Fees dont accrue for positive liquidity delta.\n        ModifyLiquidityParams memory params = LIQUIDITY_PARAMS;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n\n        // Remove liquidity - Fees dont accrue for negative liquidity delta.\n        params.liquidityDelta = -LIQUIDITY_PARAMS.liquidityDelta;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n\n        // Now re-add the liquidity to test swap\n        params.liquidityDelta = LIQUIDITY_PARAMS.liquidityDelta;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        SwapParams memory swapParams = SwapParams(false, amountSpecified, TickMath.MAX_SQRT_PRICE - 1);\n        BalanceDelta delta = swapRouter.swap(key, swapParams, PoolSwapTest.TestSettings(false, false), ZERO_BYTES);\n        uint256 expectedProtocolFee =\n            uint256(uint128(-delta.amount1())) * protocolFee1 / ProtocolFeeLibrary.PIPS_DENOMINATOR;\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), expectedProtocolFee);\n    }\n\n    function test_donate_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        donateRouter.donate(uninitializedKey, 100, 100, ZERO_BYTES);\n    }\n\n    function test_donate_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.donate(key, 100, 100, ZERO_BYTES);\n    }\n\n    function test_donate_failsIfNoLiquidity(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        (key,) = initPool(currency0, currency1, IHooks(address(0)), 100, sqrtPriceX96);\n\n        vm.expectRevert(Pool.NoLiquidityToReceiveFees.selector);\n        donateRouter.donate(key, 100, 100, ZERO_BYTES);\n    }\n\n    // test successful donation if pool has liquidity\n    function test_donate_succeedsWhenPoolHasLiquidity() public {\n        (uint256 feeGrowthGlobal0X128, uint256 feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(key.toId());\n        assertEq(feeGrowthGlobal0X128, 0);\n        assertEq(feeGrowthGlobal1X128, 0);\n\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"donate gas with 2 tokens\");\n\n        (feeGrowthGlobal0X128, feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(key.toId());\n        assertEq(feeGrowthGlobal0X128, 34028236692093846346337);\n        assertEq(feeGrowthGlobal1X128, 68056473384187692692674);\n    }\n\n    function test_donate_succeedsForNativeTokensWhenPoolHasLiquidity() public {\n        (uint256 feeGrowthGlobal0X128, uint256 feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(nativeKey.toId());\n        assertEq(feeGrowthGlobal0X128, 0);\n        assertEq(feeGrowthGlobal1X128, 0);\n\n        donateRouter.donate{value: 100}(nativeKey, 100, 200, ZERO_BYTES);\n\n        (feeGrowthGlobal0X128, feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(nativeKey.toId());\n        assertEq(feeGrowthGlobal0X128, 34028236692093846346337);\n        assertEq(feeGrowthGlobal1X128, 68056473384187692692674);\n    }\n\n    function test_donate_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_DONATE_FLAG | Hooks.AFTER_DONATE_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterDonate.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeDonate hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n\n        // Fail at afterDonate hook.\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, mockHooks.beforeDonate.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n    }\n\n    function test_donate_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_DONATE_FLAG | Hooks.AFTER_DONATE_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, mockHooks.beforeDonate.selector);\n        mockHooks.setReturnValue(mockHooks.afterDonate.selector, mockHooks.afterDonate.selector);\n\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n    }\n\n    function test_donate_OneToken_gas() public {\n        donateRouter.donate(key, 100, 0, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"donate gas with 1 token\");\n    }\n\n    function test_fuzz_donate_emits_event(uint256 amount0, uint256 amount1) public {\n        amount0 = bound(amount0, 0, uint256(int256(type(int128).max)));\n        amount1 = bound(amount1, 0, uint256(int256(type(int128).max)));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit Donate(key.toId(), address(donateRouter), uint256(amount0), uint256(amount1));\n        donateRouter.donate(key, amount0, amount1, ZERO_BYTES);\n    }\n\n    function test_take_failsWithNoLiquidity() public {\n        deployFreshManagerAndRouters();\n\n        vm.expectRevert();\n        takeRouter.take(key, 100, 0);\n    }\n\n    function test_take_failsWithInvalidTokensThatDoNotReturnTrueOnTransfer() public {\n        TestInvalidERC20 invalidToken = new TestInvalidERC20(2 ** 255);\n        Currency invalidCurrency = Currency.wrap(address(invalidToken));\n        invalidToken.approve(address(modifyLiquidityRouter), type(uint256).max);\n        invalidToken.approve(address(takeRouter), type(uint256).max);\n\n        bool currency0Invalid = invalidCurrency < currency0;\n\n        (key,) = initPoolAndAddLiquidity(\n            (currency0Invalid ? invalidCurrency : currency0),\n            (currency0Invalid ? currency0 : invalidCurrency),\n            IHooks(address(0)),\n            3000,\n            SQRT_PRICE_1_1\n        );\n\n        (uint256 amount0, uint256 amount1) = currency0Invalid ? (1, 0) : (0, 1);\n        vm.expectRevert(\n            abi.encodeWithSelector(\n                CustomRevert.WrappedError.selector,\n                address(invalidToken),\n                TestInvalidERC20.transfer.selector,\n                abi.encode(bytes32(0)),\n                abi.encodeWithSelector(CurrencyLibrary.ERC20TransferFailed.selector)\n            )\n        );\n        takeRouter.take(key, amount0, amount1);\n\n        // should not revert when non zero amount passed in for valid currency\n        // assertions inside takeRouter because it takes then settles\n        (amount0, amount1) = currency0Invalid ? (0, 1) : (1, 0);\n        takeRouter.take(key, amount0, amount1);\n    }\n\n    function test_take_succeedsWithPoolWithLiquidity() public {\n        takeRouter.take(key, 1, 1); // assertions inside takeRouter because it takes then settles\n    }\n\n    function test_take_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.take(key.currency0, address(this), 1);\n    }\n\n    function test_take_succeedsWithPoolWithLiquidityWithNativeToken() public {\n        takeRouter.take{value: 1}(nativeKey, 1, 1); // assertions inside takeRouter because it takes then settles\n    }\n\n    function test_settle_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.settle();\n    }\n\n    function test_settle_revertsSendingNative_withTokenSynced() public {\n        Actions[] memory actions = new Actions[](2);\n        bytes[] memory params = new bytes[](2);\n\n        actions[0] = Actions.SYNC;\n        params[0] = abi.encode(key.currency0);\n\n        // Revert with NonzeroNativeValue\n        actions[1] = Actions.SETTLE_NATIVE;\n        params[1] = abi.encode(1);\n\n        vm.expectRevert(IPoolManager.NonzeroNativeValue.selector);\n        actionsRouter.executeActions{value: 1}(actions, params);\n    }\n\n    function test_mint_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.mint(address(this), key.currency0.toId(), 1);\n    }\n\n    function test_burn_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.burn(address(this), key.currency0.toId(), 1);\n    }\n\n    function test_collectProtocolFees_locked_revertsWithProtocolFeeCurrencySynced() public noIsolate {\n        manager.setProtocolFeeController(address(this));\n        // currency1 is never native\n        manager.sync(key.currency1);\n        assertEq(Currency.unwrap(key.currency1), Currency.unwrap(manager.getSyncedCurrency()));\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        manager.collectProtocolFees(address(this), key.currency1, 1);\n    }\n\n    function test_sync_locked_collectProtocolFees_unlocked_revertsWithProtocolFeeCurrencySynced() public noIsolate {\n        manager.setProtocolFeeController(address(actionsRouter));\n        manager.sync(key.currency1);\n        assertEq(Currency.unwrap(key.currency1), Currency.unwrap(manager.getSyncedCurrency()));\n\n        Actions[] memory actions = new Actions[](1);\n        bytes[] memory params = new bytes[](1);\n\n        actions[0] = Actions.COLLECT_PROTOCOL_FEES;\n        params[0] = abi.encode(address(this), key.currency1, 1);\n\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        actionsRouter.executeActions(actions, params);\n    }\n\n    function test_collectProtocolFees_unlocked_revertsWithProtocolFeeCurrencySynced() public {\n        manager.setProtocolFeeController(address(actionsRouter));\n\n        Actions[] memory actions = new Actions[](2);\n        bytes[] memory params = new bytes[](2);\n\n        actions[0] = Actions.SYNC;\n        params[0] = abi.encode(key.currency1);\n\n        actions[1] = Actions.COLLECT_PROTOCOL_FEES;\n        params[1] = abi.encode(address(this), key.currency1, 1);\n\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        actionsRouter.executeActions(actions, params);\n    }\n\n    function test_collectProtocolFees_ERC20_accumulateFees_gas() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(currency0.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency0, expectedFees);\n        vm.snapshotGasLastCall(\"erc20 collect protocol fees\");\n        assertEq(currency0.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n    }\n\n    function test_collectProtocolFees_ERC20_accumulateFees_exactOutput() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key, SwapParams(true, 10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(currency0.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency0, expectedFees);\n        assertEq(currency0.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n    }\n\n    function test_collectProtocolFees_ERC20_returnsAllFeesIf0IsProvidedAsParameter() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key,\n            SwapParams(false, -10000, TickMath.MAX_SQRT_PRICE - 1),\n            PoolSwapTest.TestSettings(false, false),\n            ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), expectedFees);\n        assertEq(currency1.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency1, 0);\n        assertEq(currency1.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n    }\n\n    function test_collectProtocolFees_nativeToken_accumulateFees_gas() public {\n        uint256 expectedFees = 10;\n        Currency nativeCurrency = CurrencyLibrary.ADDRESS_ZERO;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(nativeKey, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap{value: 10000}(\n            nativeKey, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, nativeCurrency, expectedFees);\n        vm.snapshotGasLastCall(\"native collect protocol fees\");\n        assertEq(nativeCurrency.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), 0);\n    }\n\n    function test_collectProtocolFees_nativeToken_returnsAllFeesIf0IsProvidedAsParameter() public {\n        uint256 expectedFees = 10;\n        Currency nativeCurrency = CurrencyLibrary.ADDRESS_ZERO;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(nativeKey, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap{value: 10000}(\n            nativeKey, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, nativeCurrency, 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), 0);\n    }\n\n    function test_unlock_EmitsCorrectId() public {\n        vm.expectEmit(false, false, false, true);\n        emit UnlockCallback();\n        emptyUnlockRouter.unlock();\n    }\n\n    Action[] _actions;\n\n    function test_unlock_cannotBeCalledTwiceByCaller() public {\n        _actions = [Action.NESTED_SELF_UNLOCK];\n        nestedActionRouter.unlock(abi.encode(_actions));\n    }\n\n    function test_unlock_cannotBeCalledTwiceByDifferentCallers() public {\n        _actions = [Action.NESTED_EXECUTOR_UNLOCK];\n        nestedActionRouter.unlock(abi.encode(_actions));\n    }\n\n    // function testExtsloadForPoolPrice() public {\n    //     IPoolManager.key = IPoolManager.PoolKey({\n    //         currency0: currency0,\n    //         currency1: currency1,\n    //         fee: 100,\n    //         hooks: IHooks(address(0)),\n    //         tickSpacing: 10\n    //     });\n    //     manager.initialize(key, SQRT_PRICE_1_1);\n\n    //     PoolId poolId = key.toId();\n    //     bytes32 slot0Bytes = manager.extsload(keccak256(abi.encode(poolId, POOL_SLOT)));\n    //     vm.snapshotGasLastCall(\"poolExtsloadSlot0\");\n\n    //     uint160 sqrtPriceX96Extsload;\n    //     assembly {\n    //         sqrtPriceX96Extsload := and(slot0Bytes, sub(shl(160, 1), 1))\n    //     }\n    //     (uint160 sqrtPriceX96Slot0,,,,,) = manager.getSlot0(poolId);\n\n    //     // assert that extsload loads the correct storage slot which matches the true slot0\n    //     assertEq(sqrtPriceX96Extsload, sqrtPriceX96Slot0);\n    // }\n\n    // function testExtsloadMultipleSlots() public {\n    //     IPoolManager.key = IPoolManager.PoolKey({\n    //         currency0: currency0,\n    //         currency1: currency1,\n    //         fee: 100,\n    //         hooks: IHooks(address(0)),\n    //         tickSpacing: 10\n    //     });\n    //     manager.initialize(key, SQRT_PRICE_1_1);\n\n    //     // populate feeGrowthGlobalX128 struct w/ modify + swap\n    //     modifyLiquidityRouter.modifyLiquidity(key, ModifyLiquidityParams(-120, 120, 5 ether, 0));\n    //     swapRouter.swap(\n    //         key,\n    //         SwapParams(false, 1 ether, TickMath.MAX_SQRT_PRICE - 1),\n    //         PoolSwapTest.TestSettings(true, true)\n    //     );\n    //     swapRouter.swap(\n    //         key,\n    //         SwapParams(true, 5 ether, TickMath.MIN_SQRT_PRICE + 1),\n    //         PoolSwapTest.TestSettings(true, true)\n    //     );\n\n    //     PoolId poolId = key.toId();\n    //     bytes memory value = manager.extsload(bytes32(uint256(keccak256(abi.encode(poolId, POOL_SLOT))) + 1), 2);\n    //     vm.snapshotGasLastCall(\"poolExtsloadTickInfoStruct\");\n\n    //     uint256 feeGrowthGlobal0X128Extsload;\n    //     uint256 feeGrowthGlobal1X128Extsload;\n    //     assembly {\n    //         feeGrowthGlobal0X128Extsload := and(mload(add(value, 0x20)), sub(shl(256, 1), 1))\n    //         feeGrowthGlobal1X128Extsload := and(mload(add(value, 0x40)), sub(shl(256, 1), 1))\n    //     }\n\n    //     assertEq(feeGrowthGlobal0X128Extsload, 408361710565269213475534193967158);\n    //     assertEq(feeGrowthGlobal1X128Extsload, 204793365386061595215803889394593);\n    // }\n\n    function test_getPosition() public view {\n        (uint128 liquidity,,) = manager.getPositionInfo(key.toId(), address(modifyLiquidityRouter), -120, 120, 0);\n        assert(LIQUIDITY_PARAMS.liquidityDelta > 0);\n        assertEq(liquidity, uint128(uint256(LIQUIDITY_PARAMS.liquidityDelta)));\n    }\n\n    function supportsInterface(bytes4) external pure returns (bool) {\n        return true;\n    }\n}\n",
    "range": {
      "end": {
        "character": 0,
        "line": 1272
      },
      "start": {
        "character": 0,
        "line": 0
      }
    }
  }
]
```
</details>

**forge** (44.5ms, 28.4 MB) — 1 edits

<details>
<summary>Summary: <code>Array(1) [{ newText: "// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test...", range: { end...</code></summary>

```json
[
  {
    "newText": "// SPDX-License-Identifier: UNLICENSED\npragma solidity ^0.8.20;\n\nimport {Test} from \"forge-std/Test.sol\";\nimport {IHooks} from \"../src/interfaces/IHooks.sol\";\nimport {Hooks} from \"../src/libraries/Hooks.sol\";\nimport {IPoolManager} from \"../src/interfaces/IPoolManager.sol\";\nimport {IProtocolFees} from \"../src/interfaces/IProtocolFees.sol\";\nimport {PoolManager} from \"../src/PoolManager.sol\";\nimport {TickMath} from \"../src/libraries/TickMath.sol\";\nimport {Pool} from \"../src/libraries/Pool.sol\";\nimport {Deployers} from \"./utils/Deployers.sol\";\nimport {Currency, CurrencyLibrary} from \"../src/types/Currency.sol\";\nimport {MockHooks} from \"../src/test/MockHooks.sol\";\nimport {MockContract} from \"../src/test/MockContract.sol\";\nimport {EmptyTestHooks} from \"../src/test/EmptyTestHooks.sol\";\nimport {PoolKey} from \"../src/types/PoolKey.sol\";\nimport {ModifyLiquidityParams, SwapParams} from \"../src/types/PoolOperation.sol\";\nimport {PoolModifyLiquidityTest} from \"../src/test/PoolModifyLiquidityTest.sol\";\nimport {BalanceDelta, BalanceDeltaLibrary} from \"../src/types/BalanceDelta.sol\";\nimport {PoolSwapTest} from \"../src/test/PoolSwapTest.sol\";\nimport {TestInvalidERC20} from \"../src/test/TestInvalidERC20.sol\";\nimport {PoolEmptyUnlockTest} from \"../src/test/PoolEmptyUnlockTest.sol\";\nimport {Action} from \"../src/test/PoolNestedActionsTest.sol\";\nimport {PoolId} from \"../src/types/PoolId.sol\";\nimport {LPFeeLibrary} from \"../src/libraries/LPFeeLibrary.sol\";\nimport {Position} from \"../src/libraries/Position.sol\";\nimport {Constants} from \"./utils/Constants.sol\";\nimport {SafeCast} from \"../src/libraries/SafeCast.sol\";\nimport {AmountHelpers} from \"./utils/AmountHelpers.sol\";\nimport {ProtocolFeeLibrary} from \"../src/libraries/ProtocolFeeLibrary.sol\";\nimport {IProtocolFees} from \"../src/interfaces/IProtocolFees.sol\";\nimport {StateLibrary} from \"../src/libraries/StateLibrary.sol\";\nimport {TransientStateLibrary} from \"../src/libraries/TransientStateLibrary.sol\";\nimport {Actions} from \"../src/test/ActionsRouter.sol\";\nimport {CustomRevert} from \"../src/libraries/CustomRevert.sol\";\n\ncontract PoolManagerTest is Test, Deployers {\n    using Hooks for IHooks;\n    using LPFeeLibrary for uint24;\n    using SafeCast for *;\n    using ProtocolFeeLibrary for uint24;\n    using StateLibrary for IPoolManager;\n    using TransientStateLibrary for IPoolManager;\n\n    event UnlockCallback();\n    event ProtocolFeeControllerUpdated(address feeController);\n    event ModifyLiquidity(\n        PoolId indexed poolId,\n        address indexed sender,\n        int24 tickLower,\n        int24 tickUpper,\n        int256 liquidityDelta,\n        bytes32 salt\n    );\n    event Swap(\n        PoolId indexed poolId,\n        address indexed sender,\n        int128 amount0,\n        int128 amount1,\n        uint160 sqrtPriceX96,\n        uint128 liquidity,\n        int24 tick,\n        uint24 fee\n    );\n\n    event Donate(PoolId indexed id, address indexed sender, uint256 amount0, uint256 amount1);\n\n    event Transfer(\n        address caller, address indexed sender, address indexed receiver, uint256 indexed id, uint256 amount\n    );\n\n    PoolEmptyUnlockTest emptyUnlockRouter;\n\n    uint24 constant MAX_PROTOCOL_FEE_BOTH_TOKENS = (1000 << 12) | 1000; // 1000 1000\n\n    address recipientAddress = makeAddr(\"recipientAddress\");\n\n    function setUp() public {\n        initializeManagerRoutersAndPoolsWithLiq(IHooks(address(0)));\n\n        emptyUnlockRouter = new PoolEmptyUnlockTest(manager);\n    }\n\n    function test_bytecodeSize() public {\n        vm.snapshotValue(\"poolManager bytecode size\", address(manager).code.length);\n    }\n\n    function test_initcodeHash() public {\n        vm.snapshotValue(\n            \"poolManager initcode hash (without constructor params, as uint256)\",\n            uint256(keccak256(type(PoolManager).creationCode))\n        );\n    }\n\n    function test_addLiquidity_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        modifyLiquidityRouter.modifyLiquidity(uninitializedKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.modifyLiquidity(uninitializedKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        modifyLiquidityRouter.modifyLiquidity(uninitializedKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsForNativeTokensIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            nativeKey.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsForNativeTokensIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            nativeKey.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsWithHooksIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        address payable mockAddr =\n            payable(address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPool(currency0, currency1, IHooks(mockAddr), 3000, sqrtPriceX96);\n\n        BalanceDelta balanceDelta = modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeAddLiquidity.selector;\n        bytes memory beforeParams = abi.encode(address(modifyLiquidityRouter), key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        bytes32 afterSelector = MockHooks.afterAddLiquidity.selector;\n        bytes memory afterParams = abi.encode(\n            address(modifyLiquidityRouter),\n            key,\n            LIQUIDITY_PARAMS,\n            balanceDelta,\n            BalanceDeltaLibrary.ZERO_DELTA,\n            ZERO_BYTES\n        );\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_removeLiquidity_succeedsWithHooksIfInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        address payable mockAddr =\n            payable(address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPool(currency0, currency1, IHooks(mockAddr), 3000, sqrtPriceX96);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        BalanceDelta balanceDelta = modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeRemoveLiquidity.selector;\n        bytes memory beforeParams = abi.encode(address(modifyLiquidityRouter), key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        bytes32 afterSelector = MockHooks.afterRemoveLiquidity.selector;\n        bytes memory afterParams = abi.encode(\n            address(modifyLiquidityRouter),\n            key,\n            REMOVE_LIQUIDITY_PARAMS,\n            balanceDelta,\n            BalanceDeltaLibrary.ZERO_DELTA,\n            ZERO_BYTES\n        );\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_addLiquidity_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterAddLiquidity.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeAddLiquidity hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        // Fail at afterAddLiquidity hook.\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, mockHooks.beforeAddLiquidity.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterRemoveLiquidity.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeRemoveLiquidity hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        // Fail at afterRemoveLiquidity hook.\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, mockHooks.beforeRemoveLiquidity.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeAddLiquidity.selector, mockHooks.beforeAddLiquidity.selector);\n        mockHooks.setReturnValue(mockHooks.afterAddLiquidity.selector, mockHooks.afterAddLiquidity.selector);\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            LIQUIDITY_PARAMS.tickLower,\n            LIQUIDITY_PARAMS.tickUpper,\n            LIQUIDITY_PARAMS.liquidityDelta,\n            LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        mockHooks.setReturnValue(mockHooks.beforeRemoveLiquidity.selector, mockHooks.beforeRemoveLiquidity.selector);\n        mockHooks.setReturnValue(mockHooks.afterRemoveLiquidity.selector, mockHooks.afterRemoveLiquidity.selector);\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit ModifyLiquidity(\n            key.toId(),\n            address(modifyLiquidityRouter),\n            REMOVE_LIQUIDITY_PARAMS.tickLower,\n            REMOVE_LIQUIDITY_PARAMS.tickUpper,\n            REMOVE_LIQUIDITY_PARAMS.liquidityDelta,\n            REMOVE_LIQUIDITY_PARAMS.salt\n        );\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_6909() public {\n        // convert test tokens into ERC6909 claims\n        claimsRouter.deposit(currency0, address(this), 10_000e18);\n        claimsRouter.deposit(currency1, address(this), 10_000e18);\n        assertEq(manager.balanceOf(address(this), currency0.toId()), 10_000e18);\n        assertEq(manager.balanceOf(address(this), currency1.toId()), 10_000e18);\n\n        uint256 currency0BalanceBefore = currency0.balanceOfSelf();\n        uint256 currency1BalanceBefore = currency1.balanceOfSelf();\n        uint256 currency0PMBalanceBefore = currency0.balanceOf(address(manager));\n        uint256 currency1PMBalanceBefore = currency1.balanceOf(address(manager));\n\n        // allow liquidity router to burn our 6909 tokens\n        manager.setOperator(address(modifyLiquidityRouter), true);\n\n        // add liquidity with 6909: settleUsingBurn=true, takeClaims=true (unused)\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES, true, true);\n\n        assertLt(manager.balanceOf(address(this), currency0.toId()), 10_000e18);\n        assertLt(manager.balanceOf(address(this), currency1.toId()), 10_000e18);\n\n        // ERC20s are unspent\n        assertEq(currency0.balanceOfSelf(), currency0BalanceBefore);\n        assertEq(currency1.balanceOfSelf(), currency1BalanceBefore);\n\n        // PoolManager did not receive net-new ERC20s\n        assertEq(currency0.balanceOf(address(manager)), currency0PMBalanceBefore);\n        assertEq(currency1.balanceOf(address(manager)), currency1PMBalanceBefore);\n    }\n\n    function test_removeLiquidity_6909() public {\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        assertEq(manager.balanceOf(address(this), currency0.toId()), 0);\n        assertEq(manager.balanceOf(address(this), currency1.toId()), 0);\n\n        uint256 currency0BalanceBefore = currency0.balanceOfSelf();\n        uint256 currency1BalanceBefore = currency1.balanceOfSelf();\n        uint256 currency0PMBalanceBefore = currency0.balanceOf(address(manager));\n        uint256 currency1PMBalanceBefore = currency1.balanceOf(address(manager));\n\n        // remove liquidity as 6909: settleUsingBurn=true (unused), takeClaims=true\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES, true, true);\n\n        assertTrue(manager.balanceOf(address(this), currency0.toId()) > 0);\n        assertTrue(manager.balanceOf(address(this), currency1.toId()) > 0);\n\n        // ERC20s are unspent\n        assertEq(currency0.balanceOfSelf(), currency0BalanceBefore);\n        assertEq(currency1.balanceOfSelf(), currency1BalanceBefore);\n\n        // PoolManager did lose ERC-20s\n        assertEq(currency0.balanceOf(address(manager)), currency0PMBalanceBefore);\n        assertEq(currency1.balanceOf(address(manager)), currency1PMBalanceBefore);\n    }\n\n    function test_addLiquidity_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple addLiquidity\");\n    }\n\n    function test_addLiquidity_secondAdditionSameRange_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple addLiquidity second addition same range\");\n    }\n\n    function test_removeLiquidity_gas() public {\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        // add some liquidity to remove\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n\n        uniqueParams.liquidityDelta *= -1;\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple removeLiquidity\");\n    }\n\n    function test_removeLiquidity_someLiquidityRemains_gas() public {\n        // add double the liquidity to remove\n        ModifyLiquidityParams memory uniqueParams =\n            ModifyLiquidityParams({tickLower: -300, tickUpper: -180, liquidityDelta: 1e18, salt: 0});\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n\n        uniqueParams.liquidityDelta /= -2;\n        modifyLiquidityNoChecks.modifyLiquidity(key, uniqueParams, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"simple removeLiquidity some liquidity remains\");\n    }\n\n    function test_addLiquidity_succeeds() public {\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_removeLiquidity_succeeds() public {\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n    }\n\n    function test_addLiquidity_withNative_gas() public {\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"addLiquidity with native token\");\n    }\n\n    function test_removeLiquidity_withNative_gas() public {\n        modifyLiquidityRouter.modifyLiquidity{value: 1 ether}(nativeKey, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"removeLiquidity with native token\");\n    }\n\n    function test_addLiquidity_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"addLiquidity with empty hook\");\n    }\n\n    function test_removeLiquidity_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPool(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n        modifyLiquidityRouter.modifyLiquidity(key, LIQUIDITY_PARAMS, ZERO_BYTES);\n\n        modifyLiquidityRouter.modifyLiquidity(key, REMOVE_LIQUIDITY_PARAMS, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"removeLiquidity with empty hook\");\n    }\n\n    function test_swap_failsIfNotInitialized(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        key.fee = 100;\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: sqrtPriceX96});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsIfInitialized() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(\n            key.toId(), address(swapRouter), int128(-100), int128(98), 79228162514264329749955861424, 1e18, -1, 3000\n        );\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.swap(key, SWAP_PARAMS, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithNativeTokensIfInitialized() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(\n            nativeKey.toId(),\n            address(swapRouter),\n            int128(-100),\n            int128(98),\n            79228162514264329749955861424,\n            1e18,\n            -1,\n            3000\n        );\n\n        swapRouter.swap{value: 100}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithHooksIfInitialized() public {\n        address payable mockAddr = payable(address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG)));\n        address payable hookAddr = payable(Constants.ALL_HOOKS);\n\n        vm.etch(hookAddr, vm.getDeployedCode(\"EmptyTestHooks.sol:EmptyTestHooks\"));\n        MockContract mockContract = new MockContract();\n        vm.etch(mockAddr, address(mockContract).code);\n\n        MockContract(mockAddr).setImplementation(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, IHooks(mockAddr), 3000, SQRT_PRICE_1_1);\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        BalanceDelta balanceDelta = swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        bytes32 beforeSelector = MockHooks.beforeSwap.selector;\n        bytes memory beforeParams = abi.encode(address(swapRouter), key, SWAP_PARAMS, ZERO_BYTES);\n\n        bytes32 afterSelector = MockHooks.afterSwap.selector;\n        bytes memory afterParams = abi.encode(address(swapRouter), key, SWAP_PARAMS, balanceDelta, ZERO_BYTES);\n\n        assertEq(MockContract(mockAddr).timesCalledSelector(beforeSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(beforeSelector, beforeParams));\n        assertEq(MockContract(mockAddr).timesCalledSelector(afterSelector), 1);\n        assertTrue(MockContract(mockAddr).calledWithSelector(afterSelector, afterParams));\n    }\n\n    function test_swap_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: 10, sqrtPriceLimitX96: SQRT_PRICE_1_2});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterSwap.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeSwap hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n\n        // Fail at afterSwap hook.\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, mockHooks.beforeSwap.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: -10, sqrtPriceLimitX96: SQRT_PRICE_1_2});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        mockHooks.setReturnValue(mockHooks.beforeSwap.selector, mockHooks.beforeSwap.selector);\n        mockHooks.setReturnValue(mockHooks.afterSwap.selector, mockHooks.afterSwap.selector);\n\n        vm.expectEmit(true, true, true, true);\n        emit Swap(key.toId(), address(swapRouter), -10, 8, 79228162514264336880490487708, 1e18, -1, 100);\n\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_succeeds() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_gas() public {\n        swapRouterNoChecks.swap(key, SWAP_PARAMS);\n        vm.snapshotGasLastCall(\"simple swap\");\n    }\n\n    function test_swap_withNative_succeeds() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap{value: 100}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n    }\n\n    function test_swap_withNative_gas() public {\n        swapRouterNoChecks.swap{value: 100}(nativeKey, SWAP_PARAMS);\n        vm.snapshotGasLastCall(\"simple swap with native\");\n    }\n\n    function test_swap_withHooks_gas() public {\n        address allHooksAddr = Constants.ALL_HOOKS;\n\n        MockHooks impl = new MockHooks();\n        vm.etch(allHooksAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(allHooksAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 3000, SQRT_PRICE_1_1);\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory swapParams =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, swapParams, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap with hooks\");\n    }\n\n    function test_swap_mint6909IfOutputNotTaken_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(0), address(this), CurrencyLibrary.toId(currency1), 98);\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap mint output as 6909\");\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(currency1));\n        assertEq(erc6909Balance, 98);\n    }\n\n    function test_swap_mint6909IfNativeOutputNotTaken_gas() public {\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_2_1});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(0), address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 98\n        );\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap mint native output as 6909\");\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 98);\n    }\n\n    function test_swap_burn6909AsInput_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(0), address(this), CurrencyLibrary.toId(currency1), 98);\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), uint256(uint160(Currency.unwrap(currency1))));\n        assertEq(erc6909Balance, 98);\n\n        // give permission for swapRouter to burn the 6909s\n        manager.setOperator(address(swapRouter), true);\n\n        // swap from currency1 to currency0 again, using 6909s as input tokens\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: 25, sqrtPriceLimitX96: SQRT_PRICE_4_1});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: true});\n\n        vm.expectEmit();\n        emit Transfer(address(swapRouter), address(this), address(0), CurrencyLibrary.toId(currency1), 27);\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap burn 6909 for input\");\n\n        erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(currency1));\n        assertEq(erc6909Balance, 71);\n    }\n\n    function test_swap_burnNative6909AsInput_gas() public {\n        SwapParams memory params =\n            SwapParams({zeroForOne: false, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_2_1});\n\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: true, settleUsingBurn: false});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(0), address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 98\n        );\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n\n        uint256 erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 98);\n\n        // give permission for swapRouter to burn the 6909s\n        manager.setOperator(address(swapRouter), true);\n\n        // swap from currency0 to currency1, using 6909s as input tokens\n        params = SwapParams({zeroForOne: true, amountSpecified: 25, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n        testSettings = PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: true});\n\n        vm.expectEmit();\n        emit Transfer(\n            address(swapRouter), address(this), address(0), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO), 27\n        );\n        // don't have to send in native currency since burning 6909 for input\n        swapRouter.swap(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap burn native 6909 for input\");\n\n        erc6909Balance = manager.balanceOf(address(this), CurrencyLibrary.toId(CurrencyLibrary.ADDRESS_ZERO));\n        assertEq(erc6909Balance, 71);\n    }\n\n    function test_swap_againstLiquidity_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap(key, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n\n        swapRouter.swap(key, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap against liquidity\");\n    }\n\n    function test_swap_againstLiqWithNative_gas() public {\n        PoolSwapTest.TestSettings memory testSettings =\n            PoolSwapTest.TestSettings({takeClaims: false, settleUsingBurn: false});\n\n        swapRouter.swap{value: 1 ether}(nativeKey, SWAP_PARAMS, testSettings, ZERO_BYTES);\n\n        SwapParams memory params =\n            SwapParams({zeroForOne: true, amountSpecified: -100, sqrtPriceLimitX96: SQRT_PRICE_1_4});\n\n        swapRouter.swap{value: 1 ether}(nativeKey, params, testSettings, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"swap against liquidity with native token\");\n    }\n\n    function test_swap_accruesProtocolFees(uint16 protocolFee0, uint16 protocolFee1, int256 amountSpecified) public {\n        protocolFee0 = uint16(bound(protocolFee0, 0, 1000));\n        protocolFee1 = uint16(bound(protocolFee1, 0, 1000));\n        vm.assume(amountSpecified != 0);\n\n        uint24 protocolFee = (uint24(protocolFee1) << 12) | uint24(protocolFee0);\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, protocolFee);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, protocolFee);\n\n        // Add liquidity - Fees dont accrue for positive liquidity delta.\n        ModifyLiquidityParams memory params = LIQUIDITY_PARAMS;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n\n        // Remove liquidity - Fees dont accrue for negative liquidity delta.\n        params.liquidityDelta = -LIQUIDITY_PARAMS.liquidityDelta;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n\n        // Now re-add the liquidity to test swap\n        params.liquidityDelta = LIQUIDITY_PARAMS.liquidityDelta;\n        modifyLiquidityRouter.modifyLiquidity(key, params, ZERO_BYTES);\n\n        SwapParams memory swapParams = SwapParams(false, amountSpecified, TickMath.MAX_SQRT_PRICE - 1);\n        BalanceDelta delta = swapRouter.swap(key, swapParams, PoolSwapTest.TestSettings(false, false), ZERO_BYTES);\n        uint256 expectedProtocolFee =\n            uint256(uint128(-delta.amount1())) * protocolFee1 / ProtocolFeeLibrary.PIPS_DENOMINATOR;\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), expectedProtocolFee);\n    }\n\n    function test_donate_failsIfNotInitialized() public {\n        vm.expectRevert(Pool.PoolNotInitialized.selector);\n        donateRouter.donate(uninitializedKey, 100, 100, ZERO_BYTES);\n    }\n\n    function test_donate_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.donate(key, 100, 100, ZERO_BYTES);\n    }\n\n    function test_donate_failsIfNoLiquidity(uint160 sqrtPriceX96) public {\n        sqrtPriceX96 = uint160(bound(sqrtPriceX96, TickMath.MIN_SQRT_PRICE, TickMath.MAX_SQRT_PRICE - 1));\n\n        (key,) = initPool(currency0, currency1, IHooks(address(0)), 100, sqrtPriceX96);\n\n        vm.expectRevert(Pool.NoLiquidityToReceiveFees.selector);\n        donateRouter.donate(key, 100, 100, ZERO_BYTES);\n    }\n\n    // test successful donation if pool has liquidity\n    function test_donate_succeedsWhenPoolHasLiquidity() public {\n        (uint256 feeGrowthGlobal0X128, uint256 feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(key.toId());\n        assertEq(feeGrowthGlobal0X128, 0);\n        assertEq(feeGrowthGlobal1X128, 0);\n\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"donate gas with 2 tokens\");\n\n        (feeGrowthGlobal0X128, feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(key.toId());\n        assertEq(feeGrowthGlobal0X128, 34028236692093846346337);\n        assertEq(feeGrowthGlobal1X128, 68056473384187692692674);\n    }\n\n    function test_donate_succeedsForNativeTokensWhenPoolHasLiquidity() public {\n        (uint256 feeGrowthGlobal0X128, uint256 feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(nativeKey.toId());\n        assertEq(feeGrowthGlobal0X128, 0);\n        assertEq(feeGrowthGlobal1X128, 0);\n\n        donateRouter.donate{value: 100}(nativeKey, 100, 200, ZERO_BYTES);\n\n        (feeGrowthGlobal0X128, feeGrowthGlobal1X128) = manager.getFeeGrowthGlobals(nativeKey.toId());\n        assertEq(feeGrowthGlobal0X128, 34028236692093846346337);\n        assertEq(feeGrowthGlobal1X128, 68056473384187692692674);\n    }\n\n    function test_donate_failsWithIncorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_DONATE_FLAG | Hooks.AFTER_DONATE_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, bytes4(0xdeadbeef));\n        mockHooks.setReturnValue(mockHooks.afterDonate.selector, bytes4(0xdeadbeef));\n\n        // Fails at beforeDonate hook.\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n\n        // Fail at afterDonate hook.\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, mockHooks.beforeDonate.selector);\n        vm.expectRevert(Hooks.InvalidHookResponse.selector);\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n    }\n\n    function test_donate_succeedsWithCorrectSelectors() public {\n        address hookAddr = address(uint160(Hooks.BEFORE_DONATE_FLAG | Hooks.AFTER_DONATE_FLAG));\n\n        MockHooks impl = new MockHooks();\n        vm.etch(hookAddr, address(impl).code);\n        MockHooks mockHooks = MockHooks(hookAddr);\n\n        (key,) = initPoolAndAddLiquidity(currency0, currency1, mockHooks, 100, SQRT_PRICE_1_1);\n\n        mockHooks.setReturnValue(mockHooks.beforeDonate.selector, mockHooks.beforeDonate.selector);\n        mockHooks.setReturnValue(mockHooks.afterDonate.selector, mockHooks.afterDonate.selector);\n\n        donateRouter.donate(key, 100, 200, ZERO_BYTES);\n    }\n\n    function test_donate_OneToken_gas() public {\n        donateRouter.donate(key, 100, 0, ZERO_BYTES);\n        vm.snapshotGasLastCall(\"donate gas with 1 token\");\n    }\n\n    function test_fuzz_donate_emits_event(uint256 amount0, uint256 amount1) public {\n        amount0 = bound(amount0, 0, uint256(int256(type(int128).max)));\n        amount1 = bound(amount1, 0, uint256(int256(type(int128).max)));\n\n        vm.expectEmit(true, true, false, true, address(manager));\n        emit Donate(key.toId(), address(donateRouter), uint256(amount0), uint256(amount1));\n        donateRouter.donate(key, amount0, amount1, ZERO_BYTES);\n    }\n\n    function test_take_failsWithNoLiquidity() public {\n        deployFreshManagerAndRouters();\n\n        vm.expectRevert();\n        takeRouter.take(key, 100, 0);\n    }\n\n    function test_take_failsWithInvalidTokensThatDoNotReturnTrueOnTransfer() public {\n        TestInvalidERC20 invalidToken = new TestInvalidERC20(2 ** 255);\n        Currency invalidCurrency = Currency.wrap(address(invalidToken));\n        invalidToken.approve(address(modifyLiquidityRouter), type(uint256).max);\n        invalidToken.approve(address(takeRouter), type(uint256).max);\n\n        bool currency0Invalid = invalidCurrency < currency0;\n\n        (key,) = initPoolAndAddLiquidity(\n            (currency0Invalid ? invalidCurrency : currency0),\n            (currency0Invalid ? currency0 : invalidCurrency),\n            IHooks(address(0)),\n            3000,\n            SQRT_PRICE_1_1\n        );\n\n        (uint256 amount0, uint256 amount1) = currency0Invalid ? (1, 0) : (0, 1);\n        vm.expectRevert(\n            abi.encodeWithSelector(\n                CustomRevert.WrappedError.selector,\n                address(invalidToken),\n                TestInvalidERC20.transfer.selector,\n                abi.encode(bytes32(0)),\n                abi.encodeWithSelector(CurrencyLibrary.ERC20TransferFailed.selector)\n            )\n        );\n        takeRouter.take(key, amount0, amount1);\n\n        // should not revert when non zero amount passed in for valid currency\n        // assertions inside takeRouter because it takes then settles\n        (amount0, amount1) = currency0Invalid ? (0, 1) : (1, 0);\n        takeRouter.take(key, amount0, amount1);\n    }\n\n    function test_take_succeedsWithPoolWithLiquidity() public {\n        takeRouter.take(key, 1, 1); // assertions inside takeRouter because it takes then settles\n    }\n\n    function test_take_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.take(key.currency0, address(this), 1);\n    }\n\n    function test_take_succeedsWithPoolWithLiquidityWithNativeToken() public {\n        takeRouter.take{value: 1}(nativeKey, 1, 1); // assertions inside takeRouter because it takes then settles\n    }\n\n    function test_settle_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.settle();\n    }\n\n    function test_settle_revertsSendingNative_withTokenSynced() public {\n        Actions[] memory actions = new Actions[](2);\n        bytes[] memory params = new bytes[](2);\n\n        actions[0] = Actions.SYNC;\n        params[0] = abi.encode(key.currency0);\n\n        // Revert with NonzeroNativeValue\n        actions[1] = Actions.SETTLE_NATIVE;\n        params[1] = abi.encode(1);\n\n        vm.expectRevert(IPoolManager.NonzeroNativeValue.selector);\n        actionsRouter.executeActions{value: 1}(actions, params);\n    }\n\n    function test_mint_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.mint(address(this), key.currency0.toId(), 1);\n    }\n\n    function test_burn_failsIfLocked() public {\n        vm.expectRevert(IPoolManager.ManagerLocked.selector);\n        manager.burn(address(this), key.currency0.toId(), 1);\n    }\n\n    function test_collectProtocolFees_locked_revertsWithProtocolFeeCurrencySynced() public noIsolate {\n        manager.setProtocolFeeController(address(this));\n        // currency1 is never native\n        manager.sync(key.currency1);\n        assertEq(Currency.unwrap(key.currency1), Currency.unwrap(manager.getSyncedCurrency()));\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        manager.collectProtocolFees(address(this), key.currency1, 1);\n    }\n\n    function test_sync_locked_collectProtocolFees_unlocked_revertsWithProtocolFeeCurrencySynced() public noIsolate {\n        manager.setProtocolFeeController(address(actionsRouter));\n        manager.sync(key.currency1);\n        assertEq(Currency.unwrap(key.currency1), Currency.unwrap(manager.getSyncedCurrency()));\n\n        Actions[] memory actions = new Actions[](1);\n        bytes[] memory params = new bytes[](1);\n\n        actions[0] = Actions.COLLECT_PROTOCOL_FEES;\n        params[0] = abi.encode(address(this), key.currency1, 1);\n\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        actionsRouter.executeActions(actions, params);\n    }\n\n    function test_collectProtocolFees_unlocked_revertsWithProtocolFeeCurrencySynced() public {\n        manager.setProtocolFeeController(address(actionsRouter));\n\n        Actions[] memory actions = new Actions[](2);\n        bytes[] memory params = new bytes[](2);\n\n        actions[0] = Actions.SYNC;\n        params[0] = abi.encode(key.currency1);\n\n        actions[1] = Actions.COLLECT_PROTOCOL_FEES;\n        params[1] = abi.encode(address(this), key.currency1, 1);\n\n        vm.expectRevert(IProtocolFees.ProtocolFeeCurrencySynced.selector);\n        actionsRouter.executeActions(actions, params);\n    }\n\n    function test_collectProtocolFees_ERC20_accumulateFees_gas() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(currency0.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency0, expectedFees);\n        vm.snapshotGasLastCall(\"erc20 collect protocol fees\");\n        assertEq(currency0.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n    }\n\n    function test_collectProtocolFees_ERC20_accumulateFees_exactOutput() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key, SwapParams(true, 10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(currency0.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency0, expectedFees);\n        assertEq(currency0.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n    }\n\n    function test_collectProtocolFees_ERC20_returnsAllFeesIf0IsProvidedAsParameter() public {\n        uint256 expectedFees = 10;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(key, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(key.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap(\n            key,\n            SwapParams(false, -10000, TickMath.MAX_SQRT_PRICE - 1),\n            PoolSwapTest.TestSettings(false, false),\n            ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(currency0), 0);\n        assertEq(manager.protocolFeesAccrued(currency1), expectedFees);\n        assertEq(currency1.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, currency1, 0);\n        assertEq(currency1.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n    }\n\n    function test_collectProtocolFees_nativeToken_accumulateFees_gas() public {\n        uint256 expectedFees = 10;\n        Currency nativeCurrency = CurrencyLibrary.ADDRESS_ZERO;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(nativeKey, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap{value: 10000}(\n            nativeKey, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, nativeCurrency, expectedFees);\n        vm.snapshotGasLastCall(\"native collect protocol fees\");\n        assertEq(nativeCurrency.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), 0);\n    }\n\n    function test_collectProtocolFees_nativeToken_returnsAllFeesIf0IsProvidedAsParameter() public {\n        uint256 expectedFees = 10;\n        Currency nativeCurrency = CurrencyLibrary.ADDRESS_ZERO;\n\n        (,, uint24 slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, 0);\n\n        vm.prank(feeController);\n        manager.setProtocolFee(nativeKey, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        (,, slot0ProtocolFee,) = manager.getSlot0(nativeKey.toId());\n        assertEq(slot0ProtocolFee, MAX_PROTOCOL_FEE_BOTH_TOKENS);\n\n        swapRouter.swap{value: 10000}(\n            nativeKey, SwapParams(true, -10000, SQRT_PRICE_1_2), PoolSwapTest.TestSettings(false, false), ZERO_BYTES\n        );\n\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), expectedFees);\n        assertEq(manager.protocolFeesAccrued(currency1), 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), 0);\n        vm.prank(feeController);\n        manager.collectProtocolFees(recipientAddress, nativeCurrency, 0);\n        assertEq(nativeCurrency.balanceOf(recipientAddress), expectedFees);\n        assertEq(manager.protocolFeesAccrued(nativeCurrency), 0);\n    }\n\n    function test_unlock_EmitsCorrectId() public {\n        vm.expectEmit(false, false, false, true);\n        emit UnlockCallback();\n        emptyUnlockRouter.unlock();\n    }\n\n    Action[] _actions;\n\n    function test_unlock_cannotBeCalledTwiceByCaller() public {\n        _actions = [Action.NESTED_SELF_UNLOCK];\n        nestedActionRouter.unlock(abi.encode(_actions));\n    }\n\n    function test_unlock_cannotBeCalledTwiceByDifferentCallers() public {\n        _actions = [Action.NESTED_EXECUTOR_UNLOCK];\n        nestedActionRouter.unlock(abi.encode(_actions));\n    }\n\n    // function testExtsloadForPoolPrice() public {\n    //     IPoolManager.key = IPoolManager.PoolKey({\n    //         currency0: currency0,\n    //         currency1: currency1,\n    //         fee: 100,\n    //         hooks: IHooks(address(0)),\n    //         tickSpacing: 10\n    //     });\n    //     manager.initialize(key, SQRT_PRICE_1_1);\n\n    //     PoolId poolId = key.toId();\n    //     bytes32 slot0Bytes = manager.extsload(keccak256(abi.encode(poolId, POOL_SLOT)));\n    //     vm.snapshotGasLastCall(\"poolExtsloadSlot0\");\n\n    //     uint160 sqrtPriceX96Extsload;\n    //     assembly {\n    //         sqrtPriceX96Extsload := and(slot0Bytes, sub(shl(160, 1), 1))\n    //     }\n    //     (uint160 sqrtPriceX96Slot0,,,,,) = manager.getSlot0(poolId);\n\n    //     // assert that extsload loads the correct storage slot which matches the true slot0\n    //     assertEq(sqrtPriceX96Extsload, sqrtPriceX96Slot0);\n    // }\n\n    // function testExtsloadMultipleSlots() public {\n    //     IPoolManager.key = IPoolManager.PoolKey({\n    //         currency0: currency0,\n    //         currency1: currency1,\n    //         fee: 100,\n    //         hooks: IHooks(address(0)),\n    //         tickSpacing: 10\n    //     });\n    //     manager.initialize(key, SQRT_PRICE_1_1);\n\n    //     // populate feeGrowthGlobalX128 struct w/ modify + swap\n    //     modifyLiquidityRouter.modifyLiquidity(key, ModifyLiquidityParams(-120, 120, 5 ether, 0));\n    //     swapRouter.swap(\n    //         key,\n    //         SwapParams(false, 1 ether, TickMath.MAX_SQRT_PRICE - 1),\n    //         PoolSwapTest.TestSettings(true, true)\n    //     );\n    //     swapRouter.swap(\n    //         key,\n    //         SwapParams(true, 5 ether, TickMath.MIN_SQRT_PRICE + 1),\n    //         PoolSwapTest.TestSettings(true, true)\n    //     );\n\n    //     PoolId poolId = key.toId();\n    //     bytes memory value = manager.extsload(bytes32(uint256(keccak256(abi.encode(poolId, POOL_SLOT))) + 1), 2);\n    //     vm.snapshotGasLastCall(\"poolExtsloadTickInfoStruct\");\n\n    //     uint256 feeGrowthGlobal0X128Extsload;\n    //     uint256 feeGrowthGlobal1X128Extsload;\n    //     assembly {\n    //         feeGrowthGlobal0X128Extsload := and(mload(add(value, 0x20)), sub(shl(256, 1), 1))\n    //         feeGrowthGlobal1X128Extsload := and(mload(add(value, 0x40)), sub(shl(256, 1), 1))\n    //     }\n\n    //     assertEq(feeGrowthGlobal0X128Extsload, 408361710565269213475534193967158);\n    //     assertEq(feeGrowthGlobal1X128Extsload, 204793365386061595215803889394593);\n    // }\n\n    function test_getPosition() public view {\n        (uint128 liquidity,,) = manager.getPositionInfo(key.toId(), address(modifyLiquidityRouter), -120, 120, 0);\n        assert(LIQUIDITY_PARAMS.liquidityDelta > 0);\n        assertEq(liquidity, uint128(uint256(LIQUIDITY_PARAMS.liquidityDelta)));\n    }\n\n    function supportsInterface(bytes4) external pure returns (bool) {\n        return true;\n    }\n}\n\n",
    "range": {
      "end": {
        "character": 0,
        "line": 1272
      },
      "start": {
        "character": 0,
        "line": 0
      }
    }
  }
]
```
</details>

---

## textDocument/foldingRange

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/foldingRange",
  "params": {
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (10.1ms, 10.4 MB) — [{"endCharacter":1,"endLine":1261,"start...

<details>
<summary>Summary: <code>Array(93) [{ endCharacter: 1, endLine: 1261, startCharacter: 44, startLine: 37 }, { endCharacter: 6, endLine: 54, sta...</code></summary>

```json
[
  {
    "endCharacter": 1,
    "endLine": 1261,
    "startCharacter": 44,
    "startLine": 37
  },
  {
    "endCharacter": 6,
    "endLine": 54,
    "startCharacter": 4,
    "startLine": 47
  },
  {
    "endCharacter": 6,
    "endLine": 64,
    "startCharacter": 4,
    "startLine": 55
  },
  {
    "endCharacter": 6,
    "endLine": 70,
    "startCharacter": 4,
    "startLine": 68
  },
  {
    "endCharacter": 5,
    "endLine": 82,
    "startCharacter": 28,
    "startLine": 78
  },
  "... 88 more (93 total)"
]
```
</details>

**forge** (0.7ms, 28.2 MB) — [{"endCharacter":63,"endLine":35,"kind":...

<details>
<summary>Summary: <code>Array(94) [{ endCharacter: 63, endLine: 35, kind: "imports", startCharacter: 0, startLine: 3 }, { endCharacter: 1, en...</code></summary>

```json
[
  {
    "endCharacter": 63,
    "endLine": 35,
    "kind": "imports",
    "startCharacter": 0,
    "startLine": 3
  },
  {
    "endCharacter": 1,
    "endLine": 1261,
    "startCharacter": 0,
    "startLine": 37
  },
  {
    "endCharacter": 6,
    "endLine": 54,
    "startCharacter": 4,
    "startLine": 47
  },
  {
    "endCharacter": 6,
    "endLine": 64,
    "startCharacter": 4,
    "startLine": 55
  },
  {
    "endCharacter": 6,
    "endLine": 70,
    "startCharacter": 4,
    "startLine": 68
  },
  "... 89 more (94 total)"
]
```
</details>

---

## textDocument/selectionRange

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/selectionRange",
  "params": {
    "positions": [
      {
        "character": 51,
        "line": 116
      }
    ],
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (8.8ms, 10.3 MB) — [{"parent":{"parent":{"parent":{"parent"...

<details>
<summary>Summary: <code>Array(1) [{ parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { range...</code></summary>

```json
[
  {
    "parent": {
      "parent": {
        "parent": {
          "parent": {
            "parent": {
              "parent": {
                "parent": {
                  "parent": {
                    "parent": {
                      "parent": {
                        "range": {
                          "end": {
                            "character": 0,
                            "line": 1262
                          },
                          "start": {
                            "character": 0,
                            "line": 0
                          }
                        }
                      },
                      "range": {
                        "end": {
                          "character": 1,
                          "line": 1261
                        },
                        "start": {
                          "character": 0,
                          "line": 37
                        }
                      }
                    },
                    "range": {
                      "end": {
                        "character": 1,
                        "line": 1261
                      },
                      "start": {
                        "character": 44,
                        "line": 37
                      }
                    }
                  },
                  "range": {
                    "end": {
                      "character": 5,
                      "line": 129
                    },
                    "start": {
                      "character": 4,
                      "line": 115
                    }
                  }
                },
                "range": {
                  "end": {
                    "character": 5,
                    "line": 129
                  },
                  "start": {
                    "character": 82,
                    "line": 115
                  }
                }
              },
              "range": {
                "end": {
                  "character": 106,
                  "line": 116
                },
                "start": {
                  "character": 8,
                  "line": 116
                }
              }
            },
            "range": {
              "end": {
                "character": 105,
                "line": 116
              },
              "start": {
                "character": 8,
                "line": 116
              }
            }
          },
          "range": {
            "end": {
              "character": 105,
              "line": 116
            },
            "start": {
              "character": 23,
              "line": 116
            }
          }
        },
        "range": {
          "end": {
            "character": 104,
            "line": 116
          },
          "start": {
            "character": 31,
            "line": 116
          }
        }
      },
      "range": {
        "end": {
          "character": 74,
          "line": 116
        },
        "start": {
          "character": 51,
          "line": 116
        }
      }
    },
    "range": {
      "end": {
        "character": 59,
        "line": 116
      },
      "start": {
        "character": 51,
        "line": 116
      }
    }
  }
]
```
</details>

**forge** (0.1ms, 24.7 MB) — [{"parent":{"parent":{"parent":{"parent"...

<details>
<summary>Summary: <code>Array(1) [{ parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { parent: { paren...</code></summary>

```json
[
  {
    "parent": {
      "parent": {
        "parent": {
          "parent": {
            "parent": {
              "parent": {
                "parent": {
                  "parent": {
                    "parent": {
                      "parent": {
                        "parent": {
                          "range": {
                            "end": {
                              "character": 0,
                              "line": 1262
                            },
                            "start": {
                              "character": 0,
                              "line": 0
                            }
                          }
                        },
                        "range": {
                          "end": {
                            "character": 1,
                            "line": 1261
                          },
                          "start": {
                            "character": 0,
                            "line": 37
                          }
                        }
                      },
                      "range": {
                        "end": {
                          "character": 5,
                          "line": 129
                        },
                        "start": {
                          "character": 4,
                          "line": 115
                        }
                      }
                    },
                    "range": {
                      "end": {
                        "character": 5,
                        "line": 129
                      },
                      "start": {
                        "character": 82,
                        "line": 115
                      }
                    }
                  },
                  "range": {
                    "end": {
                      "character": 106,
                      "line": 116
                    },
                    "start": {
                      "character": 8,
                      "line": 116
                    }
                  }
                },
                "range": {
                  "end": {
                    "character": 105,
                    "line": 116
                  },
                  "start": {
                    "character": 8,
                    "line": 116
                  }
                }
              },
              "range": {
                "end": {
                  "character": 105,
                  "line": 116
                },
                "start": {
                  "character": 23,
                  "line": 116
                }
              }
            },
            "range": {
              "end": {
                "character": 105,
                "line": 116
              },
              "start": {
                "character": 30,
                "line": 116
              }
            }
          },
          "range": {
            "end": {
              "character": 104,
              "line": 116
            },
            "start": {
              "character": 31,
              "line": 116
            }
          }
        },
        "range": {
          "end": {
            "character": 104,
            "line": 116
          },
          "start": {
            "character": 36,
            "line": 116
          }
        }
      },
      "range": {
        "end": {
          "character": 74,
          "line": 116
        },
        "start": {
          "character": 51,
          "line": 116
        }
      }
    },
    "range": {
      "end": {
        "character": 59,
        "line": 116
      },
      "start": {
        "character": 51,
        "line": 116
      }
    }
  }
]
```
</details>

---

## textDocument/inlayHint

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/inlayHint",
  "params": {
    "range": {
      "end": {
        "character": 0,
        "line": 9999
      },
      "start": {
        "character": 0,
        "line": 0
      }
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (9.2ms, 69.1 MB) — 1082 hints (name:, hooks:, _manager:)

<details>
<summary>Summary: <code>Array(1082) [{ kind: 2, label: "name:", paddingRight: true, position: { character: 40, line: 76 } }, { kind: 2, label...</code></summary>

```json
[
  {
    "kind": 2,
    "label": "name:",
    "paddingRight": true,
    "position": {
      "character": 40,
      "line": 76
    }
  },
  {
    "kind": 2,
    "label": "hooks:",
    "paddingRight": true,
    "position": {
      "character": 48,
      "line": 79
    }
  },
  {
    "kind": 2,
    "label": "_manager:",
    "paddingRight": true,
    "position": {
      "character": 52,
      "line": 81
    }
  },
  {
    "kind": 2,
    "label": "name:",
    "paddingRight": true,
    "position": {
      "character": 25,
      "line": 85
    }
  },
  {
    "kind": 2,
    "label": "value:",
    "paddingRight": true,
    "position": {
      "character": 54,
      "line": 85
    }
  },
  "... 1077 more (1082 total)"
]
```
</details>

**forge** (2.2ms, 186.7 MB) — 1271 hints (name:, : address, hooks:)

<details>
<summary>Summary: <code>Array(1271) [{ kind: 2, label: "name:", paddingLeft: false, paddingRight: true, position: { character: 40, line: 76 }...</code></summary>

```json
[
  {
    "kind": 2,
    "label": "name:",
    "paddingLeft": false,
    "paddingRight": true,
    "position": {
      "character": 40,
      "line": 76
    }
  },
  {
    "kind": 1,
    "label": ": address",
    "paddingLeft": true,
    "paddingRight": false,
    "position": {
      "character": 59,
      "line": 76
    }
  },
  {
    "kind": 2,
    "label": "hooks:",
    "paddingLeft": false,
    "paddingRight": true,
    "position": {
      "character": 48,
      "line": 79
    }
  },
  {
    "kind": 2,
    "label": "_manager:",
    "paddingLeft": false,
    "paddingRight": true,
    "position": {
      "character": 52,
      "line": 81
    }
  },
  {
    "kind": 1,
    "label": ": contract PoolEmptyUnlockTest",
    "paddingLeft": true,
    "paddingRight": false,
    "position": {
      "character": 60,
      "line": 81
    }
  },
  "... 1266 more (1271 total)"
]
```
</details>

---

## textDocument/semanticTokens/full

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/semanticTokens/full",
  "params": {
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (14.0ms, 10.3 MB) — 1512 tokens

<details>
<summary>Summary: <code>{ data: Array(7560) [0, 0, 38, ... 7557 more], resultId: "2" }</code></summary>

```json
{
  "data": [
    0,
    0,
    38,
    14,
    0,
    "... 7555 more (7560 total)"
  ],
  "resultId": "2"
}
```
</details>

**forge** (28.2 MB) — No such method textDocument/semanticToke...

---

## textDocument/semanticTokens/range

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/semanticTokens/range",
  "params": {
    "range": {
      "end": {
        "character": 0,
        "line": 300
      },
      "start": {
        "character": 0,
        "line": 0
      }
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (10.3ms, 10.5 MB) — 417 tokens

<details>
<summary>Summary: <code>{ data: Array(2085) [0, 0, 38, ... 2082 more] }</code></summary>

```json
{
  "data": [
    0,
    0,
    38,
    14,
    0,
    "... 2080 more (2085 total)"
  ]
}
```
</details>

**forge** (28.2 MB) — No such method textDocument/semanticToke...

---

## workspace/symbol

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "workspace/symbol",
  "params": {
    "query": ""
  }
}
```

**Responses:**

**mmsaki** (72.1ms, 67.4 MB) — 1797 symbols

<details>
<summary>Summary: <code>Array(1797) [{ kind: 5, location: { range: { end: { character: 1, line: 89 }, start: { character: 0, line: 9 } }, uri...</code></summary>

```json
[
  {
    "kind": 5,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 89
        },
        "start": {
          "character": 0,
          "line": 9
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ERC6909.sol"
    },
    "name": "ERC6909"
  },
  {
    "containerName": "ERC6909",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 93,
          "line": 14
        },
        "start": {
          "character": 4,
          "line": 14
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ERC6909.sol"
    },
    "name": "isOperator"
  },
  {
    "containerName": "ERC6909",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 86,
          "line": 16
        },
        "start": {
          "character": 4,
          "line": 16
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ERC6909.sol"
    },
    "name": "balanceOf"
  },
  {
    "containerName": "ERC6909",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 113,
          "line": 18
        },
        "start": {
          "character": 4,
          "line": 18
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ERC6909.sol"
    },
    "name": "allowance"
  },
  {
    "containerName": "ERC6909",
    "kind": 12,
    "location": {
      "range": {
        "end": {
          "character": 5,
          "line": 32
        },
        "start": {
          "character": 4,
          "line": 24
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ERC6909.sol"
    },
    "name": "transfer"
  },
  "... 1792 more (1797 total)"
]
```
</details>

**forge** (34.1ms, 185.3 MB) — 15657 symbols

<details>
<summary>Summary: <code>Array(15657) [{ kind: 5, location: { range: { end: { character: 1, line: 28 }, start: { character: 0, line: 6 } }, ur...</code></summary>

```json
[
  {
    "kind": 5,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 28
        },
        "start": {
          "character": 0,
          "line": 6
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Base.sol"
    },
    "name": "CommonBase"
  },
  {
    "containerName": "CommonBase",
    "kind": 14,
    "location": {
      "range": {
        "end": {
          "character": 99,
          "line": 8
        },
        "start": {
          "character": 4,
          "line": 8
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Base.sol"
    },
    "name": "VM_ADDRESS"
  },
  {
    "containerName": "CommonBase",
    "kind": 14,
    "location": {
      "range": {
        "end": {
          "character": 83,
          "line": 10
        },
        "start": {
          "character": 4,
          "line": 10
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Base.sol"
    },
    "name": "CONSOLE"
  },
  {
    "containerName": "CommonBase",
    "kind": 14,
    "location": {
      "range": {
        "end": {
          "character": 91,
          "line": 12
        },
        "start": {
          "character": 4,
          "line": 12
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Base.sol"
    },
    "name": "CREATE2_FACTORY"
  },
  {
    "containerName": "CommonBase",
    "kind": 14,
    "location": {
      "range": {
        "end": {
          "character": 110,
          "line": 14
        },
        "start": {
          "character": 4,
          "line": 14
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Base.sol"
    },
    "name": "DEFAULT_SENDER"
  },
  "... 15652 more (15657 total)"
]
```
</details>

---

## workspace/willRenameFiles

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "workspace/willRenameFiles",
  "params": {
    "files": [
      {
        "newUri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/__lsp_bench_renamed__.sol",
        "oldUri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
      }
    ]
  }
}
```

**Responses:**

**mmsaki** (227.7ms, 67.9 MB) — 12 edits in 12 files

<details>
<summary>Summary: <code>{ changes: { file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/PoolManager.sol: Array(1) [{...</code></summary>

```json
{
  "changes": {
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/PoolManager.sol": [
      {
        "newText": "\"./libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 41,
            "line": 4
          },
          "start": {
            "character": 19,
            "line": 4
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ProtocolFees.sol": [
      {
        "newText": "\"./libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 41,
            "line": 10
          },
          "start": {
            "character": 19,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/Fuzzers.sol": [
      {
        "newText": "\"../libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 42,
            "line": 11
          },
          "start": {
            "character": 19,
            "line": 11
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/ProtocolFeesImplementation.sol": [
      {
        "newText": "\"../libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 42,
            "line": 7
          },
          "start": {
            "character": 19,
            "line": 7
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/ProxyPoolManager.sol": [
      {
        "newText": "\"../libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 42,
            "line": 4
          },
          "start": {
            "character": 19,
            "line": 4
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/TickOverflowSafetyEchidnaTest.sol": [
      {
        "newText": "\"../libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 42,
            "line": 3
          },
          "start": {
            "character": 19,
            "line": 3
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/DynamicFees.t.sol": [
      {
        "newText": "\"../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 46,
            "line": 19
          },
          "start": {
            "character": 19,
            "line": 19
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol": [
      {
        "newText": "\"../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 46,
            "line": 10
          },
          "start": {
            "character": 19,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManagerInitialize.t.sol": [
      {
        "newText": "\"../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 46,
            "line": 10
          },
          "start": {
            "character": 19,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/Tick.t.sol": [
      {
        "newText": "\"../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 46,
            "line": 6
          },
          "start": {
            "character": 19,
            "line": 6
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/libraries/Pool.t.sol": [
      {
        "newText": "\"../../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 49,
            "line": 5
          },
          "start": {
            "character": 19,
            "line": 5
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/libraries/StateLibrary.t.sol": [
      {
        "newText": "\"../../src/libraries/Pools.sol\"",
        "range": {
          "end": {
            "character": 49,
            "line": 15
          },
          "start": {
            "character": 19,
            "line": 15
          }
        }
      }
    ]
  }
}
```
</details>

**forge** (1.1ms, 180.4 MB) — null (valid)

---

## workspace/willCreateFiles

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "workspace/willCreateFiles",
  "params": {
    "files": [
      {
        "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/__lsp_bench_created__.sol"
      }
    ]
  }
}
```

**Responses:**

**mmsaki** (0.2ms, 67.0 MB) — null (valid)

**forge** (0.1ms, 185.8 MB) — null (valid)

---

## workspace/willDeleteFiles

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "workspace/willDeleteFiles",
  "params": {
    "files": [
      {
        "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
      }
    ]
  }
}
```

**Responses:**

**mmsaki** (226.0ms, 66.9 MB) — {"changes":{"file:///Users/meek/develope...

<details>
<summary>Summary: <code>{ changes: { file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/PoolManager.sol: Array(1) [{...</code></summary>

```json
{
  "changes": {
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/PoolManager.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 5
          },
          "start": {
            "character": 0,
            "line": 4
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/ProtocolFees.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 11
          },
          "start": {
            "character": 0,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/Fuzzers.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 12
          },
          "start": {
            "character": 0,
            "line": 11
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/ProtocolFeesImplementation.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 8
          },
          "start": {
            "character": 0,
            "line": 7
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/ProxyPoolManager.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 5
          },
          "start": {
            "character": 0,
            "line": 4
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/TickOverflowSafetyEchidnaTest.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 4
          },
          "start": {
            "character": 0,
            "line": 3
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/DynamicFees.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 20
          },
          "start": {
            "character": 0,
            "line": 19
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 11
          },
          "start": {
            "character": 0,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManagerInitialize.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 11
          },
          "start": {
            "character": 0,
            "line": 10
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/Tick.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 7
          },
          "start": {
            "character": 0,
            "line": 6
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/libraries/Pool.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 6
          },
          "start": {
            "character": 0,
            "line": 5
          }
        }
      }
    ],
    "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/libraries/StateLibrary.t.sol": [
      {
        "newText": "",
        "range": {
          "end": {
            "character": 0,
            "line": 16
          },
          "start": {
            "character": 0,
            "line": 15
          }
        }
      }
    ]
  }
}
```
</details>

**forge** (0.4ms, 201.6 MB) — null (valid)

---

## workspace/executeCommand

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "workspace/executeCommand",
  "params": {
    "arguments": [],
    "command": "solidity.reindex"
  }
}
```

**Responses:**

**mmsaki** (0.1ms, 10.5 MB) — {"success":true}

<details>
<summary>Summary: <code>{ success: true }</code></summary>

```json
{
  "success": true
}
```
</details>

**forge** (28.7 MB) — unknown command `solidity.reindex`

---

## textDocument/prepareCallHierarchy

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/prepareCallHierarchy",
  "params": {
    "position": {
      "character": 51,
      "line": 116
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (0.5ms, 67.9 MB) — 1 items: `TickMath`

<details>
<summary>Summary: <code>Array(1) [{ data: { nodeId: 12216 }, detail: "library TickMath", kind: 5, name: "TickMath", range: { end: { character...</code></summary>

```json
[
  {
    "data": {
      "nodeId": 12216
    },
    "detail": "library TickMath",
    "kind": 5,
    "name": "TickMath",
    "range": {
      "end": {
        "character": 1,
        "line": 237
      },
      "start": {
        "character": 0,
        "line": 9
      }
    },
    "selectionRange": {
      "end": {
        "character": 16,
        "line": 9
      },
      "start": {
        "character": 8,
        "line": 9
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
  }
]
```
</details>

**forge** (0.1ms, 197.1 MB) — 1 items: `test_addLiquidity_succeedsIfInitialized`

<details>
<summary>Summary: <code>Array(1) [{ data: Array(6) [2, "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/...", 11...</code></summary>

```json
[
  {
    "data": [
      2,
      "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol",
      115,
      13,
      115,
      "... 1 more (6 total)"
    ],
    "detail": "PoolManagerTest",
    "kind": 6,
    "name": "test_addLiquidity_succeedsIfInitialized",
    "range": {
      "end": {
        "character": 5,
        "line": 129
      },
      "start": {
        "character": 4,
        "line": 115
      }
    },
    "selectionRange": {
      "end": {
        "character": 52,
        "line": 115
      },
      "start": {
        "character": 13,
        "line": 115
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/PoolManager.t.sol"
  }
]
```
</details>

---

## textDocument/codeAction

**Request:** `textDocument/codeAction` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (402.5 MB) — timeout

**forge** (0.1ms, 199.3 MB) — []

`[empty]` `[]`

---

## callHierarchy/incomingCalls

**Request:** `callHierarchy/incomingCalls` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (29.4ms, 67.1 MB) — 63 items: `getGasCostOfGetTickAtSqrtPrice`, `boundTicks`, `MIN_SQRT_PRICE`, ... (+60)

<details>
<summary>Summary: <code>Array(63) [{ from: { data: { nodeId: 61446 }, kind: 12, name: "getGasCostOfGetTickAtSqrtPrice", range: { end: { chara...</code></summary>

```json
[
  {
    "from": {
      "data": {
        "nodeId": 61446
      },
      "kind": 12,
      "name": "getGasCostOfGetTickAtSqrtPrice",
      "range": {
        "end": {
          "character": 5,
          "line": 24
        },
        "start": {
          "character": 4,
          "line": 20
        }
      },
      "selectionRange": {
        "end": {
          "character": 43,
          "line": 20
        },
        "start": {
          "character": 13,
          "line": 20
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/TickMathTest.sol"
    },
    "fromRanges": [
      {
        "end": {
          "character": 16,
          "line": 22
        },
        "start": {
          "character": 8,
          "line": 22
        }
      }
    ]
  },
  {
    "from": {
      "data": {
        "nodeId": 52741
      },
      "kind": 12,
      "name": "boundTicks",
      "range": {
        "end": {
          "character": 5,
          "line": 117
        },
        "start": {
          "character": 4,
          "line": 89
        }
      },
      "selectionRange": {
        "end": {
          "character": 23,
          "line": 89
        },
        "start": {
          "character": 13,
          "line": 89
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/Fuzzers.sol"
    },
    "fromRanges": [
      {
        "end": {
          "character": 31,
          "line": 93
        },
        "start": {
          "character": 23,
          "line": 93
        }
      },
      {
        "end": {
          "character": 31,
          "line": 94
        },
        "start": {
          "character": 23,
          "line": 94
        }
      },
      {
        "end": {
          "character": 31,
          "line": 100
        },
        "start": {
          "character": 23,
          "line": 100
        }
      },
      {
        "end": {
          "character": 31,
          "line": 101
        },
        "start": {
          "character": 23,
          "line": 101
        }
      },
      {
        "end": {
          "character": 37,
          "line": 112
        },
        "start": {
          "character": 29,
          "line": 112
        }
      },
      "... 4 more (9 total)"
    ]
  },
  {
    "from": {
      "data": {
        "nodeId": 61455
      },
      "kind": 12,
      "name": "MIN_SQRT_PRICE",
      "range": {
        "end": {
          "character": 5,
          "line": 28
        },
        "start": {
          "character": 4,
          "line": 26
        }
      },
      "selectionRange": {
        "end": {
          "character": 27,
          "line": 26
        },
        "start": {
          "character": 13,
          "line": 26
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/TickMathTest.sol"
    },
    "fromRanges": [
      {
        "end": {
          "character": 23,
          "line": 27
        },
        "start": {
          "character": 15,
          "line": 27
        }
      }
    ]
  },
  {
    "from": {
      "data": {
        "nodeId": 83129
      },
      "kind": 12,
      "name": "getMinTick",
      "range": {
        "end": {
          "character": 5,
          "line": 98
        },
        "start": {
          "character": 4,
          "line": 96
        }
      },
      "selectionRange": {
        "end": {
          "character": 23,
          "line": 96
        },
        "start": {
          "character": 13,
          "line": 96
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/test/Tick.t.sol"
    },
    "fromRanges": [
      {
        "end": {
          "character": 24,
          "line": 97
        },
        "start": {
          "character": 16,
          "line": 97
        }
      }
    ]
  },
  {
    "from": {
      "data": {
        "nodeId": 61385
      },
      "kind": 12,
      "name": "getSqrtPriceAtTick",
      "range": {
        "end": {
          "character": 5,
          "line": 8
        },
        "start": {
          "character": 4,
          "line": 6
        }
      },
      "selectionRange": {
        "end": {
          "character": 31,
          "line": 6
        },
        "start": {
          "character": 13,
          "line": 6
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/TickMathTest.sol"
    },
    "fromRanges": [
      {
        "end": {
          "character": 23,
          "line": 7
        },
        "start": {
          "character": 15,
          "line": 7
        }
      }
    ]
  },
  "... 58 more (63 total)"
]
```
</details>

**forge** (0.1ms, 199.6 MB) — 0 items

`[empty]` `[]`

---

## callHierarchy/outgoingCalls

**Request:** `callHierarchy/outgoingCalls` at `test/PoolManager.t.sol:116:51`

**Responses:**

**mmsaki** (5.9ms, 67.5 MB) — 4 items: `revertWith`, `revertWith`, `mostSignificantBit`, ... (+1)

<details>
<summary>Summary: <code>Array(4) [{ fromRanges: Array(1) [{ end: { character: 84, line: 69 }, start: { character: 74, line: 69 } }], to: { da...</code></summary>

```json
[
  {
    "fromRanges": [
      {
        "end": {
          "character": 84,
          "line": 69
        },
        "start": {
          "character": 74,
          "line": 69
        }
      }
    ],
    "to": {
      "data": {
        "nodeId": 631
      },
      "detail": "function revertWith(bytes4 selector, int24 value) internal pure",
      "kind": 12,
      "name": "revertWith",
      "range": {
        "end": {
          "character": 5,
          "line": 36
        },
        "start": {
          "character": 4,
          "line": 30
        }
      },
      "selectionRange": {
        "end": {
          "character": 23,
          "line": 30
        },
        "start": {
          "character": 13,
          "line": 30
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/CustomRevert.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 52,
          "line": 127
        },
        "start": {
          "character": 42,
          "line": 127
        }
      }
    ],
    "to": {
      "data": {
        "nodeId": 641
      },
      "detail": "function revertWith(bytes4 selector, uint160 value) internal pure",
      "kind": 12,
      "name": "revertWith",
      "range": {
        "end": {
          "character": 5,
          "line": 45
        },
        "start": {
          "character": 4,
          "line": 39
        }
      },
      "selectionRange": {
        "end": {
          "character": 23,
          "line": 39
        },
        "start": {
          "character": 13,
          "line": 39
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/CustomRevert.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 52,
          "line": 133
        },
        "start": {
          "character": 34,
          "line": 133
        }
      }
    ],
    "to": {
      "data": {
        "nodeId": 572
      },
      "detail": "function mostSignificantBit(uint256 x) internal pure returns (uint8 r)",
      "kind": 12,
      "name": "mostSignificantBit",
      "range": {
        "end": {
          "character": 5,
          "line": 24
        },
        "start": {
          "character": 4,
          "line": 11
        }
      },
      "selectionRange": {
        "end": {
          "character": 31,
          "line": 11
        },
        "start": {
          "character": 13,
          "line": 11
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/BitMath.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 67,
          "line": 234
        },
        "start": {
          "character": 49,
          "line": 234
        }
      }
    ],
    "to": {
      "data": {
        "nodeId": 412
      },
      "detail": "function getSqrtPriceAtTick(int24 tick) internal pure returns (uint160 sqrtPriceX96)",
      "kind": 12,
      "name": "getSqrtPriceAtTick",
      "range": {
        "end": {
          "character": 5,
          "line": 113
        },
        "start": {
          "character": 4,
          "line": 56
        }
      },
      "selectionRange": {
        "end": {
          "character": 31,
          "line": 56
        },
        "start": {
          "character": 13,
          "line": 56
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/libraries/TickMath.sol"
    }
  }
]
```
</details>

**forge** (0.2ms, 185.8 MB) — 4 items: `bound`, `expectEmit`, `modifyLiquidity`, ... (+1)

<details>
<summary>Summary: <code>Array(4) [{ fromRanges: Array(1) [{ end: { character: 36, line: 116 }, start: { character: 31, line: 116 } }], to: { ...</code></summary>

```json
[
  {
    "fromRanges": [
      {
        "end": {
          "character": 36,
          "line": 116
        },
        "start": {
          "character": 31,
          "line": 116
        }
      }
    ],
    "to": {
      "data": [
        2,
        "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/StdUtils.sol",
        59,
        13,
        59,
        "... 1 more (6 total)"
      ],
      "detail": "StdUtils",
      "kind": 6,
      "name": "bound",
      "range": {
        "end": {
          "character": 5,
          "line": 62
        },
        "start": {
          "character": 4,
          "line": 59
        }
      },
      "selectionRange": {
        "end": {
          "character": 18,
          "line": 59
        },
        "start": {
          "character": 13,
          "line": 59
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/StdUtils.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 21,
          "line": 118
        },
        "start": {
          "character": 11,
          "line": 118
        }
      }
    ],
    "to": {
      "data": [
        2,
        "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Vm.sol",
        2025,
        13,
        2025,
        "... 1 more (6 total)"
      ],
      "detail": "Vm",
      "kind": 6,
      "name": "expectEmit",
      "range": {
        "end": {
          "character": 17,
          "line": 2026
        },
        "start": {
          "character": 4,
          "line": 2025
        }
      },
      "selectionRange": {
        "end": {
          "character": 23,
          "line": 2025
        },
        "start": {
          "character": 13,
          "line": 2025
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/lib/forge-std/src/Vm.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 45,
          "line": 128
        },
        "start": {
          "character": 30,
          "line": 128
        }
      }
    ],
    "to": {
      "data": [
        2,
        "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/PoolModifyLiquidityTest.sol",
        32,
        13,
        32,
        "... 1 more (6 total)"
      ],
      "detail": "PoolModifyLiquidityTest",
      "kind": 6,
      "name": "modifyLiquidity",
      "range": {
        "end": {
          "character": 5,
          "line": 38
        },
        "start": {
          "character": 4,
          "line": 32
        }
      },
      "selectionRange": {
        "end": {
          "character": 28,
          "line": 32
        },
        "start": {
          "character": 13,
          "line": 32
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/test/PoolModifyLiquidityTest.sol"
    }
  },
  {
    "fromRanges": [
      {
        "end": {
          "character": 20,
          "line": 120
        },
        "start": {
          "character": 16,
          "line": 120
        }
      }
    ],
    "to": {
      "data": [
        2,
        "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/types/PoolId.sol",
        10,
        13,
        10,
        "... 1 more (6 total)"
      ],
      "detail": "PoolIdLibrary",
      "kind": 6,
      "name": "toId",
      "range": {
        "end": {
          "character": 5,
          "line": 15
        },
        "start": {
          "character": 4,
          "line": 10
        }
      },
      "selectionRange": {
        "end": {
          "character": 17,
          "line": 10
        },
        "start": {
          "character": 13,
          "line": 10
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/v4-core/src/types/PoolId.sol"
    }
  }
]
```
</details>

---
