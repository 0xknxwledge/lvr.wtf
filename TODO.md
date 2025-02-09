- Add testing for tdigest.rs and stats.rs
- Integrate tdigest quantiles into IQR plot
- Create a descriptive statistics card
- Add asterik to realized ratio plot (used beta regression)

- FLAG THE BIG JUMPS IN RUNNING TOTALS
    1. Terra/Luna & 3AC 
    2. FTX
    3. USDC depeg, Circle
    4. Ronin Bridge hack
    5. Nomad bridge hack
    6. BlockFi 
    7. Voyager 
    8. Celcius
    9. Wormhole
    10. Multichain


POOLS WHERE TOTAL OBSERVED LVR > TOTAL SIMULATED LVR
----------------------------------------------------------------------------------------------------
Pool Name            | Pool Address                               | Observed Total - Markout 0s Total
----------------------------------------------------------------------------------------------------
USDC-WETH-500        | 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 |   $66,462,518.54
WBTC-WETH-500        | 0x4585fe77225b41b697c938b018e2ac67ac5a20c0 |   $15,645,832.43
USDC-WETH-3000       | 0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8 |   $10,730,242.49
WBTC-WETH-3000       | 0xcbcdf9626bc03e24f779434178a73a0b4bad62ed |    $8,291,126.62
DAI-WETH-500         | 0x60594a405d53811d3bc4766596efd80fd545a270 |    $5,259,682.96
DAI-WETH-3000        | 0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8 |    $3,124,656.48
WETH-USDT-500        | 0x11b815efb8f581194ae79006d24e0d814b7697f6 |    $2,250,486.24


- For pools with < 3000 non-zero blocks their TDigest throws zero for all percentiles
- ^ For observed, affected pools are 
1. DAI-USDT-100