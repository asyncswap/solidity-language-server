// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 4110: visibility already specified (on a state variable).
// Fix: remove the second `public`.

contract DuplicateVisibilityStateVar {
    uint256 public public value = 1;
}
