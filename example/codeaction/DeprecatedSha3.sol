// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 3557: "sha3" has been deprecated in favour of "keccak256".
// Fix: replace `sha3` with `keccak256`.

contract DeprecatedSha3 {
    function hash(bytes memory data) public pure returns (bytes32) {
        return sha3(data);
    }
}
