// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 9348: interfaces do not need the `abstract` keyword.
// Fix: remove `abstract` from the interface declaration.

abstract interface AbstractInterface {
    function foo() external returns (uint256);
}
