// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct Inner {
    uint256 innerField;
}

struct Outer {
    uint256 outerField;
}

interface ISource {
    function fetch() external returns (Inner memory);
}

contract ScopedLocals {
    // Same names as the locals below, different type. A lookup that misses
    // the local's scope falls through to these and offers `outerField`.
    Outer item;
    Outer got;

    function loop(Inner[] memory items) external pure returns (uint256 sum) {
        for (Inner memory item = items[0]; sum < 3; sum++) {
            sum += item.innerField;
        }
    }

    function attempt(ISource src) external returns (uint256 out) {
        try src.fetch() returns (Inner memory got) {
            out = got.innerField;
        } catch {}
    }
}
