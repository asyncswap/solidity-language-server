// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 7359: "now" has been deprecated. Use "block.timestamp" instead.
// Fix: replace `now` with `block.timestamp`.

contract DeprecatedNow {
    uint256 public createdAt;

    function stamp() public {
        createdAt = now;
    }
}
