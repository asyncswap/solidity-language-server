// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Warning 2072: unused local variable.
// Fix: delete the entire `uint256 fee = 100;` statement.

contract UnusedVar {
    function deposit(uint256 amount) public pure returns (uint256) {
        uint256 fee = 100;
        return amount;
    }
}
