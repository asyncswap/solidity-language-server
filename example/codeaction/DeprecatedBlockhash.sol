// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 8113: "block.blockhash()" has been deprecated in favor of "blockhash()".
// Fix: replace `block.blockhash` with `blockhash`.

contract DeprecatedBlockhash {
    function getHash(uint256 blockNumber) public view returns (bytes32) {
        return block.blockhash(blockNumber);
    }
}
