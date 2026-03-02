// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Test as MyTest} from "./A.sol";
import "./A.sol" as AFile;

contract AliasUser {
    MyTest public myTest;
    AFile.Test public afileTest;
}
