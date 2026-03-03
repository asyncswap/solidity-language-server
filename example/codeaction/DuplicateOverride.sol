// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 1827: override already specified on a function.
// Fix: remove the second `override`.

abstract contract Base {
    function foo() external virtual returns (uint256);
}

contract DuplicateOverride is Base {
    function foo() external override override returns (uint256) {
        return 1;
    }
}
