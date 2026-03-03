// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 1560: Functions in interfaces must be declared external.
// Fix: replace `public` visibility with `external`.

interface IFoo {
    function foo() public returns (uint256);
}
