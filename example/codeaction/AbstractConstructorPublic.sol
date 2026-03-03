// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 8295: Abstract contracts cannot have public constructors.
// Fix: remove the `public` keyword from the constructor.

abstract contract AbstractConstructorPublic {
    uint256 public value;

    constructor() public {
        value = 1;
    }
}
