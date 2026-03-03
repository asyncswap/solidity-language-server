// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 4126: free functions cannot have visibility.
// Fix: remove the `internal` visibility specifier.

function double(uint256 x) internal pure returns (uint256) {
    return x * 2;
}

contract FreeFunction {
    function compute(uint256 x) public pure returns (uint256) {
        return double(x);
    }
}
