// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 6879: virtual already specified on a function.
// Fix: remove the second `virtual`.

contract DuplicateVirtual {
    function foo() external virtual virtual returns (uint256) {
        return 1;
    }
}
