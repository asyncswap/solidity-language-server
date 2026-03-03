// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9559: Free functions cannot be payable.
// Fix: remove the `payable` modifier.

function freePayable(uint256 x) payable returns (uint256) {
    return x;
}
