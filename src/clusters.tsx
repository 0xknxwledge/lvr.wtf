export const STABLE_POOLS = {
    "0x3416cf6c708da44db2624d63ea0aaef7113527c6": "USDC-USDT-100",
    "0x5777d92f208679db4b9778590fa3cab3ac9e2168": "DAI-USDC-100",
    "0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf": "USDC-USDT-500",
    "0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406": "DAI-USDT-100",
    "0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47": "USDe-USDT-100"
} as const;

export const WBTC_WETH_POOLS = {
    "0x4585fe77225b41b697c938b018e2ac67ac5a20c0": "WBTC-WETH-500",
    "0xcbcdf9626bc03e24f779434178a73a0b4bad62ed": "WBTC-WETH-3000"
} as const;

export const USDC_WETH_POOLS = {
    "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640": "USDC-WETH-500",
    "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8": "USDC-WETH-3000",
    "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc": "WETH-USDC-v2"
} as const;

export const USDT_WETH_POOLS = {
    "0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b": "WETH-USDT-100",
    "0x4e68ccd3e89f51c3074ca5072bbac773960dfa36": "WETH-USDT-3000",
    "0x11b815efb8f581194ae79006d24e0d814b7697f6": "WETH-USDT-500",
    "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852": "USDT-WETH-v2"
} as const;

export const DAI_WETH_POOLS = {
    "0x60594a405d53811d3bc4766596efd80fd545a270": "DAI-WETH-500",
    "0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8": "DAI-WETH-3000"
} as const;

export const USDC_WBTC_POOLS = {
    "0x9a772018fbd77fcd2d25657e5c547baff3fd7d16": "WBTC-USDC-500",
    "0x99ac8ca7087fa4a2a1fb6357269965a2014abc35": "WBTC-USDC-3000"
} as const;

export const ALTCOIN_WETH_POOLS = {
    "0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801": "UNI-WETH-3000",
    "0x11950d141ecb863f01007add7d1a342041227b58": "PEPE-WETH-3000",
    "0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8": "LINK-WETH-3000",
    "0xa43fe16908251ee70ef74718545e4fe6c5ccec9f": "PEPE-WETH-V2"
} as const;


export const ALL_CLUSTERS = {
    ...STABLE_POOLS,
    ...WBTC_WETH_POOLS,
    ...USDC_WETH_POOLS,
    ...USDT_WETH_POOLS,
    ...DAI_WETH_POOLS,
    ...USDC_WBTC_POOLS,
    ...ALTCOIN_WETH_POOLS
} as const;

export type PoolAddress = keyof typeof ALL_CLUSTERS;

export type PoolName = typeof ALL_CLUSTERS[PoolAddress];