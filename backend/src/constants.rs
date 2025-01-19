use std::collections::HashMap;
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use crate::MarkoutTime;

lazy_static! {
    pub static ref POOL_ADDRESSES: Vec<&'static str> = vec![
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
        "0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801",
        "0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8",
        "0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406",
        "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852", 
        "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc",
        "0xa43fe16908251ee70ef74718545e4fe6c5ccec9f"
    ];
    
    pub static ref POOL_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640","USDC-WETH-500");
        m.insert("0x3416cf6c708da44db2624d63ea0aaef7113527c6","USDC-USDT-100");
        m.insert("0x11b815efb8f581194ae79006d24e0d814b7697f6","WETH-USDT-500");
        m.insert("0x4585fe77225b41b697c938b018e2ac67ac5a20c0","WBTC-WETH-500");
        m.insert("0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8","USDC-WETH-3000");
        m.insert("0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b","WETH-USDT-100");
        m.insert("0xcbcdf9626bc03e24f779434178a73a0b4bad62ed","WBTC-WETH-3000");
        m.insert("0x5777d92f208679db4b9778590fa3cab3ac9e2168","DAI-USDC-100");
        m.insert("0x4e68ccd3e89f51c3074ca5072bbac773960dfa36","WETH-USDT-3000");
        m.insert("0x60594a405d53811d3bc4766596efd80fd545a270","DAI-WETH-500");
        m.insert("0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf","USDC-USDT-500");
        m.insert("0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47","USDe-USDT-100");
        m.insert("0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8","LINK-WETH-3000");
        m.insert("0x11950d141ecb863f01007add7d1a342041227b58","PEPE-WETH-3000");
        m.insert("0x9a772018fbd77fcd2d25657e5c547baff3fd7d16","WBTC-USDC-500");
        m.insert("0x99ac8ca7087fa4a2a1fb6357269965a2014abc35","WBTC-USDC-3000");
        m.insert("0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801","UNI-WETH-3000");
        m.insert("0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8","DAI-WETH-3000");
        m.insert("0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406","DAI-USDT-100");
        m.insert("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852","USDT-WETH-v2");
        m.insert("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc","WETH-USDC-v2");
        m.insert("0xa43fe16908251ee70ef74718545e4fe6c5ccec9f", "PEPE-WETH-v2");
        m
    };
    pub static ref STABLE_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x3416cf6c708da44db2624d63ea0aaef7113527c6", "USDC-USDT-100");
        m.insert("0x5777d92f208679db4b9778590fa3cab3ac9e2168", "DAI-USDC-100");
        m.insert("0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf", "USDC-USDT-500");
        m.insert("0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406", "DAI-USDT-100");
        m.insert("0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47","USDe-USDT-100");
        m
    };

    pub static ref WBTC_WETH_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x4585fe77225b41b697c938b018e2ac67ac5a20c0", "WBTC-WETH-500");
        m.insert("0xcbcdf9626bc03e24f779434178a73a0b4bad62ed", "WBTC-WETH-3000");
        m
    };

    pub static ref USDC_WETH_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640", "USDC-WETH-500");
        m.insert("0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8", "USDC-WETH-3000");
        m.insert("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc","WETH-USDC-v2");
        m
    };

    pub static ref USDT_WETH_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b", "WETH-USDT-100");
        m.insert("0x4e68ccd3e89f51c3074ca5072bbac773960dfa36", "WETH-USDT-3000");
        m.insert("0x11b815efb8f581194ae79006d24e0d814b7697f6", "WETH-USDT-500");
        m.insert("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852","USDT-WETH-v2");
        m
    };

    pub static ref DAI_WETH_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x60594a405d53811d3bc4766596efd80fd545a270", "DAI-WETH-500");
        m.insert("0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8", "DAI-WETH-3000");
        m
    };

    pub static ref USDC_WBTC_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x9a772018fbd77fcd2d25657e5c547baff3fd7d16", "WBTC-USDC-500");
        m.insert("0x99ac8ca7087fa4a2a1fb6357269965a2014abc35", "WBTC-USDC-3000");
        m
    };

    pub static ref ALTCOIN_WETH_POOLS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801", "UNI-WETH-3000");
        m.insert("0x11950d141ecb863f01007add7d1a342041227b58", "PEPE-WETH-3000");
        m.insert("0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8", "LINK-WETH-3000");
        m.insert("0xa43fe16908251ee70ef74718545e4fe6c5ccec9f", "PEPE-WETH-v2");
        m
    };

    pub static ref ALL_CLUSTERS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.extend(STABLE_POOLS.iter());
        m.extend(WBTC_WETH_POOLS.iter());
        m.extend(USDC_WETH_POOLS.iter());
        m.extend(USDT_WETH_POOLS.iter());
        m.extend(DAI_WETH_POOLS.iter());
        m.extend(USDC_WBTC_POOLS.iter());
        m.extend(ALTCOIN_WETH_POOLS.iter());
        m
    };

    pub static ref INTERVAL_RANGES: HashMap<u64, &'static str> = {
        let mut m = HashMap::new();
        m.insert(15537392, "Sep 15 - Oct 15, 2022");
        m.insert(15753392, "Oct 15 - Nov 14, 2022");
        m.insert(15969392, "Nov 14 - Dec 14, 2022");
        m.insert(16185392, "Dec 14 - Jan 14, 2023");
        m.insert(16401392, "Jan 14 - Feb 13, 2023");
        m.insert(16617392, "Feb 13 - Mar 15, 2023");
        m.insert(16833392, "Mar 15 - Apr 15, 2023");
        m.insert(17049392, "Apr 15 - May 15, 2023");
        m.insert(17265392, "May 15 - Jun 14, 2023");
        m.insert(17481392, "Jun 14 - Jul 15, 2023");
        m.insert(17697392, "Jul 15 - Aug 14, 2023");
        m.insert(17913392, "Aug 14 - Sep 13, 2023");
        m.insert(18129392, "Sep 13 - Oct 14, 2023");
        m.insert(18345392, "Oct 14 - Nov 13, 2023");
        m.insert(18561392, "Nov 13 - Dec 13, 2023");
        m.insert(18777392, "Dec 13 - Jan 13, 2024");
        m.insert(18993392, "Jan 13 - Feb 13, 2024");
        m.insert(19209392, "Feb 13 - Mar 13, 2024");
        m.insert(19425392, "Mar 13 - Apr 13, 2024");
        m.insert(19641392, "Apr 13 - May 14, 2024");
        m.insert(19857392, "May 14 - Jun 1, 2024");
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

    pub static ref PEPE_DEPLOYMENT_V3: u64 = 17083569;
    pub static ref PEPE_DEPLOYMENT_V2: u64 = 17046833;
    pub static ref USDeUSDT_DEPLOYMENT: u64 = 18634804;
    pub static ref WETH_USDT_100_DEPLOYMENT: u64 = 16266586;

    pub static ref MERGE_BLOCK: u64 = 15537393;
    
}