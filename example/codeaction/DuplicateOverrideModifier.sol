// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9102: override already specified on a modifier.
// Fix: remove the second `override`.

abstract contract Base {
    modifier onlyOwner() virtual;
}

contract DuplicateOverrideModifier is Base {
    modifier onlyOwner() override override {
        _;
    }
}
