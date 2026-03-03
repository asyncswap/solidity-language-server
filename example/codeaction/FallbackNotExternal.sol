// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 1159: Fallback function must be defined as "external".
// Fix: replace `internal` visibility with `external`.

contract FallbackNotExternal {
    fallback() internal {}
}
