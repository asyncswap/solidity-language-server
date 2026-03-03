// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 8063: Modifiers without implementation must be marked virtual.
// Fix: add `virtual` to the modifier declaration.

abstract contract MissingVirtualModifier {
    modifier onlyOwner();
}
