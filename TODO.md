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

