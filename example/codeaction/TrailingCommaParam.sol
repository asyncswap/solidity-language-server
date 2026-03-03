// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 7591: unexpected trailing comma in parameter list.
// Fix: remove the trailing comma after `b`.

contract TrailingCommaParam {
    function add(uint256 a, uint256 b,) public pure returns (uint256) {
        return a + b;
    }
}
