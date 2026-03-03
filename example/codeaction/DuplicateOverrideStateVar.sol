// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9125: override already specified on a state variable.
// Fix: remove the second `override`.

abstract contract Base {
    function value() external virtual returns (uint256);
}

contract DuplicateOverrideStateVar is Base {
    uint256 public override override value = 1;
}
