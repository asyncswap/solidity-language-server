// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9439: visibility already specified (on a function).
// Fix: remove the second `public`.

contract DuplicateVisibility {
    function foo() public public returns (uint256) {
        return 1;
    }
}
