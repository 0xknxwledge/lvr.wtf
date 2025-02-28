TODO:
- Fuzz test DistributionMetrics and quantile estimates for lognormal, pareto, bimodal, etc. distributions; whats the expected relative error? How consistent is it?
- Use SmartCore K-means for category analysis
- How can we normalize volume across categories?
- Time series analysis for testing the effects of markout time
    - Quantile regression w/ markout as ordinal variable (coded as ortho polynomials)?
    - Treat markout time as continuous or ordinal?
    - EDA on daily total LVR time series shows stationarity and no significant autocorrelation for all markout times
    - ^how does fat tails affect time series assumptions?
    - Stochastic volatility modeling (often used for returns, which are multiplicative tho)?

- How will daily updates work? Talk to Joe about getting CI for the process and precompute scripts

- WRITE README; encourage pull requests!!
- ask yuki about the state of cex-dex searching/searching in general; how institutionalized is it atp?

_ SIMULATION FIXES:
    - Safe assumption to treat USDC, USDT, and DAI as the same asset on the CEX side?
        - Would then mark against most liquid orderbook @ that snapshot
        - What is methodology of the latest run; Same as gitbook except USDC == USDT on CEX side if pool is WETH/USDC or WETH/USDT?