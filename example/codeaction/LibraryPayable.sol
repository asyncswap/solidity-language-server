// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 7708: Library functions cannot be payable.
// Fix: remove the `payable` modifier.

library LibPayable {
    function compute(uint256 x) external payable returns (uint256) {
        return x * 2;
    }
}
