// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 5587: "internal" and "private" functions cannot be payable.
// Fix: remove the `payable` modifier.

contract InternalPayable {
    function compute(uint256 x) internal payable returns (uint256) {
        return x * 2;
    }
}
