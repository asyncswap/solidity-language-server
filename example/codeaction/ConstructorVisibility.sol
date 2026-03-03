// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Warning 2462: visibility for constructor is ignored.
// Fix: remove the `public` visibility specifier from the constructor.

contract ConstructorVisibility {
    uint256 public value;

    constructor() public {
        value = 1;
    }
}
