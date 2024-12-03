use std::collections::HashMap;
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use crate::MarkoutTime;

lazy_static! {
    pub static ref POOL_ADDRESSES: Vec<&'static str> = vec![
        "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640",
        "0x3416cF6C708Da44DB2624D63ea0AAef7113527C6",
        "0x11b815efB8f581194ae79006d24E0d814B7697F6",
        "0x4585FE77225b41b697C938B018E2Ac67Ac5a20c0",
        "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
        "0xc7bBeC68d12a0d1830360F8Ec58fA599bA1b0e9b",
        "0xCBCdF9626bC03E24f779434178A73a0B4bad62eD",
        "0x5777d92f208679DB4b9778590Fa3CAB3aC9e2168",
        "0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36",
        "0x60594a405d53811d3BC4766596EFD80fd545A270",
        "0x7858E59e0C01EA06Df3aF3D20aC7B0003275D4Bf",
        "0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47",
        "0xa6Cc3C2531FdaA6Ae1A3CA84c2855806728693e8",
        "0x11950d141EcB863F01007AdD7D1A342041227b58",
        "0x9a772018FbD77fcD2d25657e5C547BAfF3Fd7D16",
        "0x99ac8cA7087fA4A2A1FB6357269965A2014ABc35",
        "0xa3f558aebAecAf0e11cA4b2199cC5Ed341edfd74",
        "0x1d42064Fc4Beb5F8aAF85F4617AE8b3b5B8Bd801",
        "0xC2e9F25Be6257c210d7Adf0D4Cd6E3E881ba25f8",
        "0x48DA0965ab2d2cbf1C17C09cFB5Cbe67Ad5B1406",
        "0x840DEEef2f115Cf50DA625F7368C24af6fE74410",
        "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852",
        "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
    ];

    pub static ref BRONTES_ADDIES: Vec<&'static str> = vec![
        "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
        "0x3416cf6c708da44db2624d63ea0aaef7113527c6",
        "0x11b815efb8f581194ae79006d24e0d814b7697f6",
        "0x4585fe77225b41b697c938b018e2ac67ac5a20c0",
        "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8",
        "0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b",
        "0xcbcdf9626bc03e24f779434178a73a0b4bad62ed",
        "0x5777d92f208679db4b9778590fa3cab3ac9e2168",
        "0x4e68ccd3e89f51c3074ca5072bbac773960dfa36",
        "0x60594a405d53811d3bc4766596efd80fd545a270",
        "0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf",
        "0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47",
        "0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8",
        "0x11950d141ecb863f01007add7d1a342041227b58",
        "0x9a772018fbd77fcd2d25657e5c547baff3fd7d16",
        "0x99ac8ca7087fa4a2a1fb6357269965a2014abc35",
        "0xa3f558aebaecaf0e11ca4b2199cc5ed341edfd74",
        "0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801",
        "0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8",
        "0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406",
        "0x840deeef2f115cf50da625f7368c24af6fe74410",
        "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852",
        "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
    ];
    
    pub static ref POOL_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640","USDC-WETH-500");
        m.insert("0x3416cF6C708Da44DB2624D63ea0AAef7113527C6","USDC-USDT-100");
        m.insert("0x11b815efB8f581194ae79006d24E0d814B7697F6","WETH-USDT-500");
        m.insert("0x4585FE77225b41b697C938B018E2Ac67Ac5a20c0","WBTC-WETH-500");
        m.insert("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8","USDC-WETH-3000");
        m.insert("0xc7bBeC68d12a0d1830360F8Ec58fA599bA1b0e9b","WETH-USDT-100");
        m.insert("0xCBCdF9626bC03E24f779434178A73a0B4bad62eD","WBTC-WETH-3000");
        m.insert("0x5777d92f208679DB4b9778590Fa3CAB3aC9e2168","DAI-USDC-100");
        m.insert("0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36","WETH-USDT-3000");
        m.insert("0x60594a405d53811d3BC4766596EFD80fd545A270","DAI-WETH-500");
        m.insert("0x7858E59e0C01EA06Df3aF3D20aC7B0003275D4Bf","USDC-USDT-500");
        m.insert("0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47","USDe-USDT-100");
        m.insert("0xa6Cc3C2531FdaA6Ae1A3CA84c2855806728693e8","LINK-WETH-3000");
        m.insert("0x11950d141EcB863F01007AdD7D1A342041227b58","PEPE-WETH-3000");
        m.insert("0x9a772018FbD77fcD2d25657e5C547BAfF3Fd7D16","WBTC-USDC-500");
        m.insert("0x99ac8cA7087fA4A2A1FB6357269965A2014ABc35","WBTC-USDC-3000");
        m.insert("0xa3f558aebAecAf0e11cA4b2199cC5Ed341edfd74","LDO-WETH-3000");
        m.insert("0x1d42064Fc4Beb5F8aAF85F4617AE8b3b5B8Bd801","UNI-WETH-3000");
        m.insert("0xC2e9F25Be6257c210d7Adf0D4Cd6E3E881ba25f8","DAI-WETH-3000");
        m.insert("0x48DA0965ab2d2cbf1C17C09cFB5Cbe67Ad5B1406","DAI-USDT-100");
        m.insert("0x840DEEef2f115Cf50DA625F7368C24af6fE74410","cbETH-WETH-500");
        m.insert("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852","USDT-WETH-v2");
        m.insert("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc","WETH-USDC-v2");
        m
    };
    
    #[derive(Debug)]
    pub static ref MARKOUT_TIMES: Vec<f64> = vec![-2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0];
    pub static ref CHECKPOINT_UPDATE_INTERVAL: u64 = 1000; // Update checkpoints every 1000 blocks
    pub static ref MARKOUT_TIME_MAPPING: HashMap<OrderedFloat<f64>, u64> = {
        let mut map = HashMap::new();
        let markout_variants = [
            MarkoutTime::Negative2,
            MarkoutTime::Negative15,
            MarkoutTime::Negative1,
            MarkoutTime::Negative05,
            MarkoutTime::Zero,
            MarkoutTime::Positive05,
            MarkoutTime::Positive1,
            MarkoutTime::Positive15,
            MarkoutTime::Positive2,
        ];
        for (index, variant) in markout_variants.iter().enumerate() {
            if let Some(value) = variant.as_f64() {
                map.insert(OrderedFloat(value), index as u64);
            }
        }
        map
    };

    pub static ref PEPE_DEPLOYMENT: u64 = 17083569;
    pub static ref USDeUSDT_DEPLOYMENT: u64 = 18634804;
    pub static ref WETH_USDT_100_DEPLOYMENT: u64 = 16266586;

    pub static ref MERGE_BLOCK: u64 = 15537393;
    
}