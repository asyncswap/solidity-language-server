// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9239: Constructor cannot have visibility.
// Fix: remove the visibility specifier (private/external) from the constructor.

contract ConstructorPrivateVisibility {
    uint256 public value;

    constructor() private {
        value = 1;
    }
}
