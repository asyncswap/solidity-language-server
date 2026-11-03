# Session Log — example / Shop.sol

## initialize

**Request:** `initialize` at `Shop.sol:130:26`

**Responses:**

**mmsaki** (12.5ms, 10.9 MB) — ok

<details>
<summary>Summary: <code>"ok"</code></summary>

```json
"ok"
```
</details>

**forge** (19.0ms, 25.1 MB) — ok

<details>
<summary>Summary: <code>"ok"</code></summary>

```json
"ok"
```
</details>

---

## textDocument/diagnostic

**Request:** `textDocument/diagnostic` at `Shop.sol:130:26`

**Responses:**

**mmsaki** (0.2ms, 12.0 MB) — {"message":"lsp server initialized.","ty...

<details>
<summary>Summary: <code>{ message: "lsp server initialized.", type: 3 }</code></summary>

```json
{
  "message": "lsp server initialized.",
  "type": 3
}
```
</details>

**forge** (0.2ms, 25.3 MB) — {"message":"solar initialized","type":3}

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

## textDocument/definition

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/definition",
  "params": {
    "position": {
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (6.4ms, 20.4 MB) — `Shop.sol:41`

<details>
<summary>Summary: <code>{ range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } }, uri: "file:///Users/meek/developer...</code></summary>

```json
{
  "range": {
    "end": {
      "character": 19,
      "line": 41
    },
    "start": {
      "character": 13,
      "line": 41
    }
  },
  "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
}
```
</details>

**forge** (0.1ms, 33.3 MB) — `Shop.sol:41`

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } }, uri: "file:///Users/meek...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (1.3ms, 20.3 MB) — `Shop.sol:41`

<details>
<summary>Summary: <code>{ range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } }, uri: "file:///Users/meek/developer...</code></summary>

```json
{
  "range": {
    "end": {
      "character": 19,
      "line": 41
    },
    "start": {
      "character": 13,
      "line": 41
    }
  },
  "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
}
```
</details>

**forge** (0.1ms, 33.2 MB) — `Shop.sol:41`

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } }, uri: "file:///Users/meek...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (1.0ms, 20.4 MB) — function addTax(uint256 amount, uint16 tax, uint16...

<details>
<summary>Summary: <code>{ contents: { kind: "markdown", value: "```solidity
function addTax(uint256 amount, uint16 tax, uint16 base) internal...</code></summary>

```json
{
  "contents": {
    "kind": "markdown",
    "value": "```solidity\nfunction addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256)\n```\n\n---\nCalculates the total amount with tax applied.\n\n**Parameters:**\n- `amount` — The base amount before tax.\n- `tax` — The tax numerator.\n- `base` — The tax denominator.\n\n**Returns:**\n- `The` — total amount including tax."
  }
}
```
</details>

**forge** (0.1ms, 33.2 MB) — function addTax(uint256 amount, uint16 tax, uint16...

<details>
<summary>Summary: <code>{ contents: { kind: "markdown", value: "```solidity
function addTax(uint256 amount, uint16 tax, uint16 base) internal...</code></summary>

```json
{
  "contents": {
    "kind": "markdown",
    "value": "```solidity\nfunction addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256)\n```\n\nCalculates the total amount with tax applied.\n\n**@param**\n\n- `amount`: The base amount before tax.\n\n- `tax`: The tax numerator.\n\n- `base`: The tax denominator.\n\n**@return**\n\n- The total amount including tax."
  },
  "range": {
    "end": {
      "character": 44,
      "line": 136
    },
    "start": {
      "character": 38,
      "line": 136
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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (2.5ms, 20.4 MB) — 2 references

<details>
<summary>Summary: <code>Array(2) [{ range: { end: { character: 44, line: 136 }, start: { character: 38, line: 136 } }, uri: "file:///Users/me...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 44,
        "line": 136
      },
      "start": {
        "character": 38,
        "line": 136
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
  },
  {
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
  }
]
```
</details>

**forge** (0.1ms, 33.3 MB) — 2 references

<details>
<summary>Summary: <code>Array(2) [{ range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } }, uri: "file:///Users/meek...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
  },
  {
    "range": {
      "end": {
        "character": 44,
        "line": 136
      },
      "start": {
        "character": 38,
        "line": 136
      }
    },
    "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
  }
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
      "character": 38,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (13.7ms, 20.3 MB) — 2 items (addTax, getRefund)

<details>
<summary>Summary: <code>{ isIncomplete: false, items: Array(2) [{ detail: "function addTax(uint256 amount, uint16 tax, uint16 base) internal ...</code></summary>

```json
{
  "isIncomplete": false,
  "items": [
    {
      "detail": "function addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256)",
      "kind": 3,
      "label": "addTax"
    },
    {
      "detail": "function getRefund(uint256 amount, uint16 rate, uint16 base) internal pure returns (uint256)",
      "kind": 3,
      "label": "getRefund"
    }
  ]
}
```
</details>

**forge** (0.1ms, 33.1 MB) — 2 items (addTax, getRefund)

<details>
<summary>Summary: <code>Array(2) [{ data: Array(6) [1, "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol", 41...</code></summary>

```json
[
  {
    "data": [
      1,
      "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol",
      41,
      13,
      41,
      "... 1 more (6 total)"
    ],
    "detail": "Transaction",
    "kind": 2,
    "label": "addTax"
  },
  {
    "data": [
      1,
      "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol",
      50,
      13,
      50,
      "... 1 more (6 total)"
    ],
    "detail": "Transaction",
    "kind": 2,
    "label": "getRefund"
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
      "character": 45,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (0.9ms, 20.3 MB) — function addTax(uint256 amount, uint16 tax, uint16...

<details>
<summary>Summary: <code>{ activeParameter: 1, activeSignature: 0, signatures: Array(1) [{ activeParameter: 1, label: "function addTax(uint256...</code></summary>

```json
{
  "activeParameter": 1,
  "activeSignature": 0,
  "signatures": [
    {
      "activeParameter": 1,
      "label": "function addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256)",
      "parameters": [
        {
          "label": [
            16,
            30
          ]
        },
        {
          "label": [
            32,
            42
          ]
        },
        {
          "label": [
            44,
            55
          ]
        }
      ]
    }
  ]
}
```
</details>

**forge** (0.1ms, 33.4 MB) — function addTax(uint16 tax, uint16 base) internal ...

<details>
<summary>Summary: <code>{ activeParameter: 0, activeSignature: 0, signatures: Array(1) [{ documentation: "Calculates the total amount with ta...</code></summary>

```json
{
  "activeParameter": 0,
  "activeSignature": 0,
  "signatures": [
    {
      "documentation": "Calculates the total amount with tax applied.",
      "label": "function addTax(uint16 tax, uint16 base) internal pure returns (uint256)",
      "parameters": [
        {
          "documentation": "The tax numerator.",
          "label": "uint16 tax"
        },
        {
          "documentation": "The tax denominator.",
          "label": "uint16 base"
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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (2.5ms, 20.3 MB) — 2 edits in 1 files

<details>
<summary>Summary: <code>{ changes: { file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol: Array(2) [{ newText: "...</code></summary>

```json
{
  "changes": {
    "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol": [
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 44,
            "line": 136
          },
          "start": {
            "character": 38,
            "line": 136
          }
        }
      },
      {
        "newText": "__lsp_bench_rename__",
        "range": {
          "end": {
            "character": 19,
            "line": 41
          },
          "start": {
            "character": 13,
            "line": 41
          }
        }
      }
    ]
  }
}
```
</details>

**forge** (33.5 MB) — cannot rename this symbol because worksp...

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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (0.2ms, 20.5 MB) — ready (line 136)

<details>
<summary>Summary: <code>{ end: { character: 44, line: 136 }, start: { character: 38, line: 136 } }</code></summary>

```json
{
  "end": {
    "character": 44,
    "line": 136
  },
  "start": {
    "character": 38,
    "line": 136
  }
}
```
</details>

**forge** (33.3 MB) — cannot rename this symbol because worksp...

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
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (2.1ms, 10.9 MB) — 4 symbols

<details>
<summary>Summary: <code>Array(4) [{ kind: 15, name: "pragma solidity ^0.8.0", range: { end: { character: 23, line: 1 }, start: { character: 0...</code></summary>

```json
[
  {
    "kind": 15,
    "name": "pragma solidity ^0.8.0",
    "range": {
      "end": {
        "character": 23,
        "line": 1
      },
      "start": {
        "character": 0,
        "line": 1
      }
    },
    "selectionRange": {
      "end": {
        "character": 23,
        "line": 1
      },
      "start": {
        "character": 0,
        "line": 1
      }
    }
  },
  {
    "children": [
      {
        "children": [
          {
            "kind": 8,
            "name": "buyer",
            "range": {
              "end": {
                "character": 22,
                "line": 29
              },
              "start": {
                "character": 8,
                "line": 29
              }
            },
            "selectionRange": {
              "end": {
                "character": 21,
                "line": 29
              },
              "start": {
                "character": 16,
                "line": 29
              }
            }
          },
          {
            "kind": 8,
            "name": "nonce",
            "range": {
              "end": {
                "character": 22,
                "line": 30
              },
              "start": {
                "character": 8,
                "line": 30
              }
            },
            "selectionRange": {
              "end": {
                "character": 21,
                "line": 30
              },
              "start": {
                "character": 16,
                "line": 30
              }
            }
          },
          {
            "kind": 8,
            "name": "amount",
            "range": {
              "end": {
                "character": 23,
                "line": 31
              },
              "start": {
                "character": 8,
                "line": 31
              }
            },
            "selectionRange": {
              "end": {
                "character": 22,
                "line": 31
              },
              "start": {
                "character": 16,
                "line": 31
              }
            }
          },
          {
            "kind": 8,
            "name": "date",
            "range": {
              "end": {
                "character": 21,
                "line": 32
              },
              "start": {
                "character": 8,
                "line": 32
              }
            },
            "selectionRange": {
              "end": {
                "character": 20,
                "line": 32
              },
              "start": {
                "character": 16,
                "line": 32
              }
            }
          },
          {
            "kind": 8,
            "name": "confirmed",
            "range": {
              "end": {
                "character": 23,
                "line": 33
              },
              "start": {
                "character": 8,
                "line": 33
              }
            },
            "selectionRange": {
              "end": {
                "character": 22,
                "line": 33
              },
              "start": {
                "character": 13,
                "line": 33
              }
            }
          }
        ],
        "kind": 23,
        "name": "Order",
        "range": {
          "end": {
            "character": 5,
            "line": 34
          },
          "start": {
            "character": 4,
            "line": 28
          }
        },
        "selectionRange": {
          "end": {
            "character": 16,
            "line": 28
          },
          "start": {
            "character": 11,
            "line": 28
          }
        }
      },
      {
        "detail": "(uint256 amount, uint16 tax, uint16 base) returns (uint256)",
        "kind": 12,
        "name": "addTax",
        "range": {
          "end": {
            "character": 5,
            "line": 43
          },
          "start": {
            "character": 4,
            "line": 41
          }
        },
        "selectionRange": {
          "end": {
            "character": 19,
            "line": 41
          },
          "start": {
            "character": 13,
            "line": 41
          }
        }
      },
      {
        "detail": "(uint256 amount, uint16 rate, uint16 base) returns (uint256)",
        "kind": 12,
        "name": "getRefund",
        "range": {
          "end": {
            "character": 5,
            "line": 52
          },
          "start": {
            "character": 4,
            "line": 50
          }
        },
        "selectionRange": {
          "end": {
            "character": 22,
            "line": 50
          },
          "start": {
            "character": 13,
            "line": 50
          }
        }
      }
    ],
    "kind": 3,
    "name": "Transaction",
    "range": {
      "end": {
        "character": 1,
        "line": 53
      },
      "start": {
        "character": 0,
        "line": 21
      }
    },
    "selectionRange": {
      "end": {
        "character": 19,
        "line": 21
      },
      "start": {
        "character": 8,
        "line": 21
      }
    }
  },
  {
    "children": [
      {
        "kind": 7,
        "name": "using Transaction for uint256",
        "range": {
          "end": {
            "character": 34,
            "line": 61
          },
          "start": {
            "character": 4,
            "line": 61
          }
        },
        "selectionRange": {
          "end": {
            "character": 34,
            "line": 61
          },
          "start": {
            "character": 4,
            "line": 61
          }
        }
      },
      {
        "kind": 8,
        "name": "TAX",
        "range": {
          "end": {
            "character": 25,
            "line": 63
          },
          "start": {
            "character": 4,
            "line": 63
          }
        },
        "selectionRange": {
          "end": {
            "character": 24,
            "line": 63
          },
          "start": {
            "character": 21,
            "line": 63
          }
        }
      },
      {
        "kind": 8,
        "name": "TAX_BASE",
        "range": {
          "end": {
            "character": 30,
            "line": 64
          },
          "start": {
            "character": 4,
            "line": 64
          }
        },
        "selectionRange": {
          "end": {
            "character": 29,
            "line": 64
          },
          "start": {
            "character": 21,
            "line": 64
          }
        }
      },
      {
        "kind": 8,
        "name": "REFUND_RATE",
        "range": {
          "end": {
            "character": 33,
            "line": 65
          },
          "start": {
            "character": 4,
            "line": 65
          }
        },
        "selectionRange": {
          "end": {
            "character": 32,
            "line": 65
          },
          "start": {
            "character": 21,
            "line": 65
          }
        }
      },
      {
        "kind": 8,
        "name": "REFUND_BASE",
        "range": {
          "end": {
            "character": 33,
            "line": 66
          },
          "start": {
            "character": 4,
            "line": 66
          }
        },
        "selectionRange": {
          "end": {
            "character": 32,
            "line": 66
          },
          "start": {
            "character": 21,
            "line": 66
          }
        }
      },
      "... 48 more (53 total)"
    ],
    "kind": 5,
    "name": "Shop",
    "range": {
      "end": {
        "character": 1,
        "line": 263
      },
      "start": {
        "character": 0,
        "line": 60
      }
    },
    "selectionRange": {
      "end": {
        "character": 13,
        "line": 60
      },
      "start": {
        "character": 9,
        "line": 60
      }
    }
  },
  {
    "kind": 2,
    "name": "import \"./A.sol\"",
    "range": {
      "end": {
        "character": 29,
        "line": 265
      },
      "start": {
        "character": 0,
        "line": 265
      }
    },
    "selectionRange": {
      "end": {
        "character": 29,
        "line": 265
      },
      "start": {
        "character": 0,
        "line": 265
      }
    }
  }
]
```
</details>

**forge** (0.3ms, 25.2 MB) — 103 symbols

<details>
<summary>Summary: <code>Array(103) [{ kind: 2, location: { range: { end: { character: 1, line: 53 }, start: { character: 0, line: 21 } }, uri...</code></summary>

```json
[
  {
    "kind": 2,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 53
        },
        "start": {
          "character": 0,
          "line": 21
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    },
    "name": "Transaction"
  },
  {
    "containerName": "Transaction",
    "kind": 23,
    "location": {
      "range": {
        "end": {
          "character": 5,
          "line": 34
        },
        "start": {
          "character": 4,
          "line": 28
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    },
    "name": "Order"
  },
  {
    "containerName": "Order",
    "kind": 7,
    "location": {
      "range": {
        "end": {
          "character": 21,
          "line": 29
        },
        "start": {
          "character": 8,
          "line": 29
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    },
    "name": "buyer"
  },
  {
    "containerName": "Order",
    "kind": 7,
    "location": {
      "range": {
        "end": {
          "character": 21,
          "line": 30
        },
        "start": {
          "character": 8,
          "line": 30
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    },
    "name": "nonce"
  },
  {
    "containerName": "Order",
    "kind": 7,
    "location": {
      "range": {
        "end": {
          "character": 22,
          "line": 31
        },
        "start": {
          "character": 8,
          "line": 31
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    },
    "name": "amount"
  },
  "... 98 more (103 total)"
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
      "character": 40,
      "line": 136
    },
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (1.3ms, 20.4 MB) — [{"kind":3,"range":{"end":{"character":1...

<details>
<summary>Summary: <code>Array(2) [{ kind: 3, range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } } }, { kind: 2, ra...</code></summary>

```json
[
  {
    "kind": 3,
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 44,
        "line": 136
      },
      "start": {
        "character": 38,
        "line": 136
      }
    }
  }
]
```
</details>

**forge** (0.1ms, 33.3 MB) — [{"kind":3,"range":{"end":{"character":1...

<details>
<summary>Summary: <code>Array(2) [{ kind: 3, range: { end: { character: 19, line: 41 }, start: { character: 13, line: 41 } } }, { kind: 2, ra...</code></summary>

```json
[
  {
    "kind": 3,
    "range": {
      "end": {
        "character": 19,
        "line": 41
      },
      "start": {
        "character": 13,
        "line": 41
      }
    }
  },
  {
    "kind": 2,
    "range": {
      "end": {
        "character": 44,
        "line": 136
      },
      "start": {
        "character": 38,
        "line": 136
      }
    }
  }
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
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (0.2ms, 20.4 MB) — 1 links

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 27, line: 265 }, start: { character: 20, line: 265 } }, target: "file:///Users...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 27,
        "line": 265
      },
      "start": {
        "character": 20,
        "line": 265
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol",
    "tooltip": "/Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
  }
]
```
</details>

**forge** (0.1ms, 33.4 MB) — 1 links

<details>
<summary>Summary: <code>Array(1) [{ range: { end: { character: 28, line: 265 }, start: { character: 19, line: 265 } }, target: "file:///Users...</code></summary>

```json
[
  {
    "range": {
      "end": {
        "character": 28,
        "line": 265
      },
      "start": {
        "character": 19,
        "line": 265
      }
    },
    "target": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
  }
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
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (18.1ms, 10.6 MB) — 1 edits

<details>
<summary>Summary: <code>Array(1) [{ newText: "// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

//
//               ...", range: { end...</code></summary>

```json
[
  {
    "newText": "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\n//\n//                                                  █████\n//                                                 ░░███\n//   ██████  █████ █████ █████████████       █████  ░███████    ██████  ████████\n//  ███░░███░░███ ░░███ ░░███░░███░░███     ███░░   ░███░░███  ███░░███░░███░░███\n// ░███████  ░███  ░███  ░███ ░███ ░███    ░░█████  ░███ ░███ ░███ ░███ ░███ ░███\n// ░███░░░   ░░███ ███   ░███ ░███ ░███     ░░░░███ ░███ ░███ ░███ ░███ ░███ ░███\n// ░░██████   ░░█████    █████░███ █████    ██████  ████ █████░░██████  ░███████\n//  ░░░░░░     ░░░░░    ░░░░░ ░░░ ░░░░░    ░░░░░░  ░░░░ ░░░░░  ░░░░░░   ░███░░░\n//                                                                      ░███\n//                                                                      █████\n//                                                                     ░░░░░\n//\n\n/// @title Transaction Library\n/// @author mmsaki\n/// @notice Utility library for computing tax and refund amounts on orders.\n/// @custom:lsp-enable gas-estimates\nlibrary Transaction {\n    /// @notice Represents a purchase order in the shop.\n    /// @param buyer The address of the buyer who placed the order.\n    /// @param nonce The buyer's order sequence number.\n    /// @param amount The total amount paid including tax.\n    /// @param date The block timestamp when the order was placed.\n    /// @param confirmed Whether the buyer has confirmed receipt.\n    struct Order {\n        address buyer;\n        uint256 nonce;\n        uint256 amount;\n        uint256 date;\n        bool confirmed;\n    }\n\n    /// @notice Calculates the total amount with tax applied.\n    /// @param amount The base amount before tax.\n    /// @param tax The tax numerator.\n    /// @param base The tax denominator.\n    /// @return The total amount including tax.\n    function addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256) {\n        return amount + (amount * tax / base);\n    }\n\n    /// @notice Calculates the refund amount based on a refund rate.\n    /// @param amount The original order amount.\n    /// @param rate The refund rate numerator.\n    /// @param base The refund rate denominator.\n    /// @return The refund amount.\n    function getRefund(uint256 amount, uint16 rate, uint16 base) internal pure returns (uint256) {\n        return amount * rate / base;\n    }\n}\n\n/// @title Shop\n/// @author mmsaki\n/// @notice A simple e-commerce shop contract with tax, refunds, and two-step ownership transfer.\n/// @dev Uses the Transaction library for tax and refund calculations. Follows CEI pattern.\n/// @custom:lsp-enable gas-estimates\ncontract Shop {\n    using Transaction for uint256;\n\n    uint16 immutable TAX;\n    uint16 immutable TAX_BASE;\n    uint16 immutable REFUND_RATE;\n    uint16 immutable REFUND_BASE;\n    uint256 immutable REFUND_POLICY;\n    uint256 immutable PRICE;\n    address payable public owner;\n    address payable public pendingOwner;\n\n    mapping(bytes32 => Transaction.Order) public orders;\n    mapping(address => uint256) public nonces;\n    mapping(bytes32 => bool) public refunds;\n    mapping(bytes32 => bool) public paid;\n    uint256 lastBuy;\n    bool public partialWithdrawal;\n    bool public shopClosed;\n    uint256 public totalConfirmedAmount;\n\n    event BuyOrder(bytes32 orderId, uint256 amount);\n    event RefundProcessed(bytes32 orderId, uint256 amount);\n    event OrderConfirmed(bytes32 orderId);\n    event ShopOpen(uint256 timestamp);\n    event ShopClosed(uint256 timestamp);\n    event OwnershipTransferInitiated(address indexed previousOwner, address indexed newOwner);\n    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);\n\n    error ExcessAmount();\n    error InsufficientAmount();\n    error DuplicateRefundClaim();\n    error RefundPolicyExpired();\n    error InvalidRefundBenefiary();\n    error ShopIsClosed();\n    error UnauthorizedAccess();\n    error MissingTax();\n    error WaitUntilRefundPeriodPassed();\n    error InvalidConstructorParameters();\n    error InvalidPendingOwner();\n    error NoPendingOwnershipTransfer();\n    error TransferFailed();\n    error OrderAlreadyConfirmed();\n    error InvalidOrder();\n\n    constructor(uint256 price, uint16 tax, uint16 taxBase, uint16 refundRate, uint16 refundBase, uint256 refundPolicy) {\n        if (price == 0) revert InvalidConstructorParameters();\n        if (taxBase == 0) revert InvalidConstructorParameters();\n        if (tax > taxBase) revert InvalidConstructorParameters();\n        if (refundBase == 0) revert InvalidConstructorParameters();\n        if (refundRate > refundBase) revert InvalidConstructorParameters();\n        if (refundPolicy == 0) revert InvalidConstructorParameters();\n\n        if (msg.sender == address(0)) revert InvalidConstructorParameters();\n\n        PRICE = price;\n        TAX = tax;\n        TAX_BASE = taxBase;\n        REFUND_RATE = refundRate;\n        REFUND_BASE = refundBase;\n        REFUND_POLICY = refundPolicy;\n        owner = payable(msg.sender);\n    }\n\n    modifier onlyOwner() {\n        checkOwner();\n        _;\n    }\n\n    function checkOwner() internal view {\n        if (msg.sender != owner) revert UnauthorizedAccess();\n    }\n\n    function buy() public payable {\n        if (shopClosed) revert ShopIsClosed();\n        if (msg.value == PRICE) revert MissingTax();\n        uint256 expectedTotal = PRICE.addTax(TAX, TAX_BASE);\n        if (msg.value < expectedTotal) revert InsufficientAmount();\n        if (msg.value > expectedTotal) revert ExcessAmount();\n        uint256 nonce = nonces[msg.sender];\n        bytes32 orderId = keccak256(abi.encode(msg.sender, nonce));\n        nonces[msg.sender]++;\n        orders[orderId] = Transaction.Order(msg.sender, nonce, expectedTotal, block.timestamp, false);\n        lastBuy = block.timestamp;\n        emit BuyOrder(orderId, msg.value);\n    }\n\n    /// @param orderId the id of the order\n    function refund(bytes32 orderId) external {\n        Transaction.Order memory order = orders[orderId];\n\n        // Checks - validate order exists and caller is authorized\n        if (order.buyer == address(0)) revert InvalidRefundBenefiary();\n        if (order.buyer != msg.sender) revert InvalidRefundBenefiary();\n        if (block.timestamp > order.date + REFUND_POLICY) revert RefundPolicyExpired();\n        if (refunds[orderId]) revert DuplicateRefundClaim();\n\n        // Effects - update state before external calls\n        refunds[orderId] = true;\n        if (order.confirmed) {\n            totalConfirmedAmount -= order.amount;\n        }\n        uint256 refundAmount = order.amount.getRefund(REFUND_RATE, REFUND_BASE);\n\n        // Interactions - external call last\n        (bool success,) = payable(msg.sender).call{value: refundAmount}(\"\");\n        if (!success) revert TransferFailed();\n        emit RefundProcessed(orderId, refundAmount);\n    }\n\n    function getOrder(bytes32 orderId) external view returns (Transaction.Order memory) {\n        return orders[orderId];\n    }\n\n    function confirmReceived(bytes32 orderId) external {\n        Transaction.Order storage order = orders[orderId];\n\n        // Checks\n        if (order.buyer == address(0)) revert InvalidOrder();\n        if (order.buyer != msg.sender) revert InvalidRefundBenefiary();\n        if (order.confirmed) revert OrderAlreadyConfirmed();\n\n        // Effects\n        order.confirmed = true;\n        totalConfirmedAmount += order.amount;\n\n        emit OrderConfirmed(orderId);\n    }\n\n    function withdraw() public onlyOwner {\n        uint256 balance = address(this).balance;\n        uint256 confirmedAmount = totalConfirmedAmount;\n        uint256 unconfirmedAmount = balance - confirmedAmount;\n        uint256 withdrawable = 0;\n\n        // Check if refund period has passed\n        if (lastBuy + REFUND_POLICY < block.timestamp) {\n            // Full withdrawal allowed - refund period has passed for all orders\n            withdrawable = balance;\n            partialWithdrawal = false;\n\n            if (withdrawable > 0) {\n                totalConfirmedAmount = 0; // Reset since everything is withdrawn\n                (bool success,) = owner.call{value: withdrawable}(\"\");\n                if (!success) revert TransferFailed();\n            }\n        } else {\n            // Refund period still active - only allow partial withdrawal of unconfirmed amounts\n            // Confirmed amounts are locked until refund period passes\n            if (partialWithdrawal) {\n                revert WaitUntilRefundPeriodPassed();\n            }\n\n            withdrawable = unconfirmedAmount * REFUND_RATE / REFUND_BASE;\n            partialWithdrawal = true;\n\n            if (withdrawable > 0) {\n                // Don't touch totalConfirmedAmount - confirmed funds stay locked\n                (bool success,) = owner.call{value: withdrawable}(\"\");\n                if (!success) revert TransferFailed();\n            }\n        }\n    }\n\n    function openShop() public onlyOwner {\n        if (shopClosed) {\n            shopClosed = false;\n            emit ShopOpen(block.timestamp);\n        }\n    }\n\n    function closeShop() public onlyOwner {\n        shopClosed = true;\n        emit ShopClosed(block.timestamp);\n    }\n\n    function transferOwnership(address payable newOwner) public onlyOwner {\n        if (newOwner == address(0)) revert InvalidPendingOwner();\n        if (newOwner == owner) revert InvalidPendingOwner();\n        pendingOwner = newOwner;\n        emit OwnershipTransferInitiated(owner, newOwner);\n    }\n\n    function acceptOwnership() public {\n        if (msg.sender != pendingOwner) revert UnauthorizedAccess();\n        if (pendingOwner == address(0)) revert NoPendingOwnershipTransfer();\n\n        address payable previousOwner = owner;\n        owner = pendingOwner;\n        pendingOwner = payable(address(0));\n\n        emit OwnershipTransferred(previousOwner, owner);\n    }\n\n    function cancelOwnershipTransfer() public onlyOwner {\n        if (pendingOwner == address(0)) revert NoPendingOwnershipTransfer();\n        pendingOwner = payable(address(0));\n        emit OwnershipTransferInitiated(owner, address(0));\n    }\n\n    receive() external payable {\n        revert(\"Direct transfers not allowed\");\n    }\n}\n\nimport {Test} from \"./A.sol\";\n",
    "range": {
      "end": {
        "character": 0,
        "line": 282
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

**forge** (27.3ms, 25.2 MB) — 1 edits

<details>
<summary>Summary: <code>Array(1) [{ newText: "// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

//
//               ...", range: { end...</code></summary>

```json
[
  {
    "newText": "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\n//\n//                                                  █████\n//                                                 ░░███\n//   ██████  █████ █████ █████████████       █████  ░███████    ██████  ████████\n//  ███░░███░░███ ░░███ ░░███░░███░░███     ███░░   ░███░░███  ███░░███░░███░░███\n// ░███████  ░███  ░███  ░███ ░███ ░███    ░░█████  ░███ ░███ ░███ ░███ ░███ ░███\n// ░███░░░   ░░███ ███   ░███ ░███ ░███     ░░░░███ ░███ ░███ ░███ ░███ ░███ ░███\n// ░░██████   ░░█████    █████░███ █████    ██████  ████ █████░░██████  ░███████\n//  ░░░░░░     ░░░░░    ░░░░░ ░░░ ░░░░░    ░░░░░░  ░░░░ ░░░░░  ░░░░░░   ░███░░░\n//                                                                      ░███\n//                                                                      █████\n//                                                                     ░░░░░\n//\n\n/// @title Transaction Library\n/// @author mmsaki\n/// @notice Utility library for computing tax and refund amounts on orders.\n/// @custom:lsp-enable gas-estimates\nlibrary Transaction {\n    /// @notice Represents a purchase order in the shop.\n    /// @param buyer The address of the buyer who placed the order.\n    /// @param nonce The buyer's order sequence number.\n    /// @param amount The total amount paid including tax.\n    /// @param date The block timestamp when the order was placed.\n    /// @param confirmed Whether the buyer has confirmed receipt.\n    struct Order {\n        address buyer;\n        uint256 nonce;\n        uint256 amount;\n        uint256 date;\n        bool confirmed;\n    }\n\n    /// @notice Calculates the total amount with tax applied.\n    /// @param amount The base amount before tax.\n    /// @param tax The tax numerator.\n    /// @param base The tax denominator.\n    /// @return The total amount including tax.\n    function addTax(uint256 amount, uint16 tax, uint16 base) internal pure returns (uint256) {\n        return amount + (amount * tax / base);\n    }\n\n    /// @notice Calculates the refund amount based on a refund rate.\n    /// @param amount The original order amount.\n    /// @param rate The refund rate numerator.\n    /// @param base The refund rate denominator.\n    /// @return The refund amount.\n    function getRefund(uint256 amount, uint16 rate, uint16 base) internal pure returns (uint256) {\n        return amount * rate / base;\n    }\n}\n\n/// @title Shop\n/// @author mmsaki\n/// @notice A simple e-commerce shop contract with tax, refunds, and two-step ownership transfer.\n/// @dev Uses the Transaction library for tax and refund calculations. Follows CEI pattern.\n/// @custom:lsp-enable gas-estimates\ncontract Shop {\n    using Transaction for uint256;\n\n    uint16 immutable TAX;\n    uint16 immutable TAX_BASE;\n    uint16 immutable REFUND_RATE;\n    uint16 immutable REFUND_BASE;\n    uint256 immutable REFUND_POLICY;\n    uint256 immutable PRICE;\n    address payable public owner;\n    address payable public pendingOwner;\n\n    mapping(bytes32 => Transaction.Order) public orders;\n    mapping(address => uint256) public nonces;\n    mapping(bytes32 => bool) public refunds;\n    mapping(bytes32 => bool) public paid;\n    uint256 lastBuy;\n    bool public partialWithdrawal;\n    bool public shopClosed;\n    uint256 public totalConfirmedAmount;\n\n    event BuyOrder(bytes32 orderId, uint256 amount);\n    event RefundProcessed(bytes32 orderId, uint256 amount);\n    event OrderConfirmed(bytes32 orderId);\n    event ShopOpen(uint256 timestamp);\n    event ShopClosed(uint256 timestamp);\n    event OwnershipTransferInitiated(address indexed previousOwner, address indexed newOwner);\n    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);\n\n    error ExcessAmount();\n    error InsufficientAmount();\n    error DuplicateRefundClaim();\n    error RefundPolicyExpired();\n    error InvalidRefundBenefiary();\n    error ShopIsClosed();\n    error UnauthorizedAccess();\n    error MissingTax();\n    error WaitUntilRefundPeriodPassed();\n    error InvalidConstructorParameters();\n    error InvalidPendingOwner();\n    error NoPendingOwnershipTransfer();\n    error TransferFailed();\n    error OrderAlreadyConfirmed();\n    error InvalidOrder();\n\n    constructor(uint256 price, uint16 tax, uint16 taxBase, uint16 refundRate, uint16 refundBase, uint256 refundPolicy) {\n        if (price == 0) revert InvalidConstructorParameters();\n        if (taxBase == 0) revert InvalidConstructorParameters();\n        if (tax > taxBase) revert InvalidConstructorParameters();\n        if (refundBase == 0) revert InvalidConstructorParameters();\n        if (refundRate > refundBase) revert InvalidConstructorParameters();\n        if (refundPolicy == 0) revert InvalidConstructorParameters();\n\n        if (msg.sender == address(0)) revert InvalidConstructorParameters();\n\n        PRICE = price;\n        TAX = tax;\n        TAX_BASE = taxBase;\n        REFUND_RATE = refundRate;\n        REFUND_BASE = refundBase;\n        REFUND_POLICY = refundPolicy;\n        owner = payable(msg.sender);\n    }\n\n    modifier onlyOwner() {\n        checkOwner();\n        _;\n    }\n\n    function checkOwner() internal view {\n        if (msg.sender != owner) revert UnauthorizedAccess();\n    }\n\n    function buy() public payable {\n        if (shopClosed) revert ShopIsClosed();\n        if (msg.value == PRICE) revert MissingTax();\n        uint256 expectedTotal = PRICE.addTax(TAX, TAX_BASE);\n        if (msg.value < expectedTotal) revert InsufficientAmount();\n        if (msg.value > expectedTotal) revert ExcessAmount();\n        uint256 nonce = nonces[msg.sender];\n        bytes32 orderId = keccak256(abi.encode(msg.sender, nonce));\n        nonces[msg.sender]++;\n        orders[orderId] = Transaction.Order(msg.sender, nonce, expectedTotal, block.timestamp, false);\n        lastBuy = block.timestamp;\n        emit BuyOrder(orderId, msg.value);\n    }\n\n    /// @param orderId the id of the order\n    function refund(bytes32 orderId) external {\n        Transaction.Order memory order = orders[orderId];\n\n        // Checks - validate order exists and caller is authorized\n        if (order.buyer == address(0)) revert InvalidRefundBenefiary();\n        if (order.buyer != msg.sender) revert InvalidRefundBenefiary();\n        if (block.timestamp > order.date + REFUND_POLICY) revert RefundPolicyExpired();\n        if (refunds[orderId]) revert DuplicateRefundClaim();\n\n        // Effects - update state before external calls\n        refunds[orderId] = true;\n        if (order.confirmed) {\n            totalConfirmedAmount -= order.amount;\n        }\n        uint256 refundAmount = order.amount.getRefund(REFUND_RATE, REFUND_BASE);\n\n        // Interactions - external call last\n        (bool success,) = payable(msg.sender).call{value: refundAmount}(\"\");\n        if (!success) revert TransferFailed();\n        emit RefundProcessed(orderId, refundAmount);\n    }\n\n    function getOrder(bytes32 orderId) external view returns (Transaction.Order memory) {\n        return orders[orderId];\n    }\n\n    function confirmReceived(bytes32 orderId) external {\n        Transaction.Order storage order = orders[orderId];\n\n        // Checks\n        if (order.buyer == address(0)) revert InvalidOrder();\n        if (order.buyer != msg.sender) revert InvalidRefundBenefiary();\n        if (order.confirmed) revert OrderAlreadyConfirmed();\n\n        // Effects\n        order.confirmed = true;\n        totalConfirmedAmount += order.amount;\n\n        emit OrderConfirmed(orderId);\n    }\n\n    function withdraw() public onlyOwner {\n        uint256 balance = address(this).balance;\n        uint256 confirmedAmount = totalConfirmedAmount;\n        uint256 unconfirmedAmount = balance - confirmedAmount;\n        uint256 withdrawable = 0;\n\n        // Check if refund period has passed\n        if (lastBuy + REFUND_POLICY < block.timestamp) {\n            // Full withdrawal allowed - refund period has passed for all orders\n            withdrawable = balance;\n            partialWithdrawal = false;\n\n            if (withdrawable > 0) {\n                totalConfirmedAmount = 0; // Reset since everything is withdrawn\n                (bool success,) = owner.call{value: withdrawable}(\"\");\n                if (!success) revert TransferFailed();\n            }\n        } else {\n            // Refund period still active - only allow partial withdrawal of unconfirmed amounts\n            // Confirmed amounts are locked until refund period passes\n            if (partialWithdrawal) {\n                revert WaitUntilRefundPeriodPassed();\n            }\n\n            withdrawable = unconfirmedAmount * REFUND_RATE / REFUND_BASE;\n            partialWithdrawal = true;\n\n            if (withdrawable > 0) {\n                // Don't touch totalConfirmedAmount - confirmed funds stay locked\n                (bool success,) = owner.call{value: withdrawable}(\"\");\n                if (!success) revert TransferFailed();\n            }\n        }\n    }\n\n    function openShop() public onlyOwner {\n        if (shopClosed) {\n            shopClosed = false;\n            emit ShopOpen(block.timestamp);\n        }\n    }\n\n    function closeShop() public onlyOwner {\n        shopClosed = true;\n        emit ShopClosed(block.timestamp);\n    }\n\n    function transferOwnership(address payable newOwner) public onlyOwner {\n        if (newOwner == address(0)) revert InvalidPendingOwner();\n        if (newOwner == owner) revert InvalidPendingOwner();\n        pendingOwner = newOwner;\n        emit OwnershipTransferInitiated(owner, newOwner);\n    }\n\n    function acceptOwnership() public {\n        if (msg.sender != pendingOwner) revert UnauthorizedAccess();\n        if (pendingOwner == address(0)) revert NoPendingOwnershipTransfer();\n\n        address payable previousOwner = owner;\n        owner = pendingOwner;\n        pendingOwner = payable(address(0));\n\n        emit OwnershipTransferred(previousOwner, owner);\n    }\n\n    function cancelOwnershipTransfer() public onlyOwner {\n        if (pendingOwner == address(0)) revert NoPendingOwnershipTransfer();\n        pendingOwner = payable(address(0));\n        emit OwnershipTransferInitiated(owner, address(0));\n    }\n\n    receive() external payable {\n        revert(\"Direct transfers not allowed\");\n    }\n}\n\n",
    "range": {
      "end": {
        "character": 0,
        "line": 282
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

## textDocument/semanticTokens/full

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "textDocument/semanticTokens/full",
  "params": {
    "textDocument": {
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Shop.sol"
    }
  }
}
```

**Responses:**

**mmsaki** (2.1ms, 11.8 MB) — 455 tokens

<details>
<summary>Summary: <code>{ data: Array(2275) [0, 0, 31, ... 2272 more], resultId: "2" }</code></summary>

```json
{
  "data": [
    0,
    0,
    31,
    14,
    0,
    "... 2270 more (2275 total)"
  ],
  "resultId": "2"
}
```
</details>

**forge** (25.2 MB) — No such method textDocument/semanticToke...

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

**mmsaki** (4.3ms, 20.5 MB) — 182 symbols

<details>
<summary>Summary: <code>Array(182) [{ kind: 23, location: { range: { end: { character: 1, line: 5 }, start: { character: 0, line: 3 } }, uri:...</code></summary>

```json
[
  {
    "kind": 23,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 5
        },
        "start": {
          "character": 0,
          "line": 3
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
    },
    "name": "Test"
  },
  {
    "containerName": "Test",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 16,
          "line": 4
        },
        "start": {
          "character": 4,
          "line": 4
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
    },
    "name": "foo"
  },
  {
    "kind": 5,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 10
        },
        "start": {
          "character": 0,
          "line": 7
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "AliasUser"
  },
  {
    "containerName": "AliasUser",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 25,
          "line": 8
        },
        "start": {
          "character": 4,
          "line": 8
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "myTest"
  },
  {
    "containerName": "AliasUser",
    "kind": 8,
    "location": {
      "range": {
        "end": {
          "character": 32,
          "line": 9
        },
        "start": {
          "character": 4,
          "line": 9
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "afileTest"
  },
  "... 177 more (182 total)"
]
```
</details>

**forge** (0.9ms, 33.4 MB) — 253 symbols

<details>
<summary>Summary: <code>Array(253) [{ kind: 23, location: { range: { end: { character: 1, line: 5 }, start: { character: 0, line: 3 } }, uri:...</code></summary>

```json
[
  {
    "kind": 23,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 5
        },
        "start": {
          "character": 0,
          "line": 3
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
    },
    "name": "Test"
  },
  {
    "containerName": "Test",
    "kind": 7,
    "location": {
      "range": {
        "end": {
          "character": 15,
          "line": 4
        },
        "start": {
          "character": 4,
          "line": 4
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/A.sol"
    },
    "name": "foo"
  },
  {
    "kind": 3,
    "location": {
      "range": {
        "end": {
          "character": 26,
          "line": 4
        },
        "start": {
          "character": 0,
          "line": 4
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "AFile"
  },
  {
    "kind": 5,
    "location": {
      "range": {
        "end": {
          "character": 1,
          "line": 10
        },
        "start": {
          "character": 0,
          "line": 7
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "AliasUser"
  },
  {
    "containerName": "AliasUser",
    "kind": 7,
    "location": {
      "range": {
        "end": {
          "character": 25,
          "line": 8
        },
        "start": {
          "character": 4,
          "line": 8
        }
      },
      "uri": "file:///Users/meek/developer/asyncswap/solidity-language-server/example/Alias.sol"
    },
    "name": "myTest"
  },
  "... 248 more (253 total)"
]
```
</details>

---
