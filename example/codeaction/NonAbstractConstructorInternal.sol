// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 1845: Non-abstract contracts cannot have internal constructors.
// Fix: remove the `internal` keyword from the constructor.

contract NonAbstractConstructorInternal {
    uint256 public value;

    constructor() internal {
        value = 1;
    }
}
