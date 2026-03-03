// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

// Error 8050: "suicide" has been deprecated in favour of "selfdestruct".
// Fix: replace `suicide` with `selfdestruct`.

contract DeprecatedSuicide {
    function destroy() public {
        suicide(payable(msg.sender));
    }
}
