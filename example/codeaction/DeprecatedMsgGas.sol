// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 1400: "msg.gas" has been deprecated in favor of "gasleft()".
// Fix: replace `msg.gas` with `gasleft()`.

contract DeprecatedMsgGas {
    function remainingGas() public view returns (uint256) {
        return msg.gas;
    }
}
