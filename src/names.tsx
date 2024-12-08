import React from "react"

let names: PoolNames = {
    "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640": "USDC/WETH(5bp)",
    "0x3416cf6c708da44db2624d63ea0aaef7113527c6": "USDC/USDT(1bp)",
    "0x11b815efb8f581194ae79006d24e0d814b7697f6": "WETH/USDT(5bp)",
    "0x4585fe77225b41b697c938b018e2ac67ac5a20c0": "WBTC/WETH(5bp)",
    "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8": "USDC/WETH(30bp)",
    "0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b": "WETH/USDT(1bp)",
    "0xcbcdf9626bc03e24f779434178a73a0b4bad62ed": "WBTC/WETH(30bp)",
    "0x5777d92f208679db4b9778590fa3cab3ac9e2168": "DAI/USDC(1bp)",
    "0x4e68ccd3e89f51c3074ca5072bbac773960dfa36": "WETH/USDT(30bp)",
    "0x60594a405d53811d3bc4766596efd80fd545a270": "DAI/WETH(5bp)",
    "0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf": "USDC/USDT(5bp)",
    "0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47": "USDe/USDT(1bp)",
    "0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8": "LINK/WETH(30bp)",
    "0x11950d141ecb863f01007add7d1a342041227b58": "PEPE/WETH(30bp)",
    "0x9a772018fbd77fcd2d25657e5c547baff3fd7d16": "WBTC/USDC(5bp)",
    "0x99ac8ca7087fa4a2a1fb6357269965a2014abc35": "WBTC/USDC(30bp)",
    "0xa3f558aebaecaf0e11ca4b2199cc5ed341edfd74": "LDO/WETH(30bp)",
    "0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801": "UNI/WETH(30bp)",
    "0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8": "DAI/WETH(30bp)",
    "0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406": "DAI/USDT(1bp)",
    "0x840deeef2f115cf50da625f7368c24af6fe74410": "cbETH/WETH(5bp)"
}

export type PoolNames = {
    [key: string]: string;
};
  

export default names;