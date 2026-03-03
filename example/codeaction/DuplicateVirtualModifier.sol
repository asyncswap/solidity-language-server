// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 2662: virtual already specified on a modifier.
// Fix: remove the second `virtual`.

contract DuplicateVirtualModifier {
    modifier onlyOwner() virtual virtual {
        _;
    }
}
