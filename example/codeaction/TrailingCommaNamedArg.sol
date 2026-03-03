// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 2074: unexpected trailing comma in named argument block.
// Fix: remove the trailing comma after `b: 2`.

contract TrailingCommaNamedArg {
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }

    function call() public pure returns (uint256) {
        return add({a: 1, b: 2,});
    }
}
