// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9680: state mutability already specified as "pure".
// Fix: remove the second `pure`.

contract DuplicateStateMutability {
    function compute(uint256 x) public pure pure returns (uint256) {
        return x * 2;
    }
}
