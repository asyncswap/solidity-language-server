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

// Built-in members (#242): arrays, bytes, value types, selectors, and private
// members of base contracts.

type Wad is uint256;

contract HiddenBase {
    // Private: invisible to `Derived`, like forge-std's `VmSafe private vm`.
    Outer private hidden;
}

contract VisibleBase {
    Inner internal hidden;
}

contract Derived is VisibleBase, HiddenBase {
    error Bad();

    uint256[] stored;

    function members(Inner[] memory list, bytes memory raw, bytes32 word) external returns (uint256) {
        stored.push(list.length);
        bytes4 sel = ISource.fetch.selector;
        bytes4 err = Derived.Bad.selector;
        return Wad.unwrap(Wad.wrap(raw.length + word.length + list[0].innerField + hidden.innerField))
            + uint32(sel) + uint32(err);
    }
}
