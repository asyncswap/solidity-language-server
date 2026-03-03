// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 4095: Receive ether function must be defined as "external".
// Fix: replace `internal` visibility with `external`.

contract ReceiveNotExternal {
    receive() internal payable {}
}
