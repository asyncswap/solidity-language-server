// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 5424: functions without implementation must be marked virtual.
// Fix: insert `virtual` into the modifier list of `foo` and `bar`.

abstract contract MissingVirtual {
    // no body and no `virtual` — triggers 5424
    function foo() external returns (uint256);

    // with a return type — `virtual` must go before `returns`
    function bar(uint256 x) external pure returns (uint256);
}
