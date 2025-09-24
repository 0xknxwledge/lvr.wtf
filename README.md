Loss-Versus-Rebalancing (LVR), as defined by Millonis et al., measures the cost on-chain liquidity providers (LPs) face from trading at outdated prices compared to centralized exchanges (CEXs). While Automated Market Makers (AMMs) update prices every 12 seconds, CEXs operate in real-time, enabling arbitrageurs to profit from price gaps.

To quantify this, Brontes flagged potential CEX-DEX arbitrage trades on Ethereum and estimated profits using T+0 markouts against Binance mid-prices. However, its accuracy depends on correctly identifying arbitrage trades.

LVR.wtf builds on this by comparing Brontes with alternative methods, offering a new perspective on LVR through empirical simulation that attempts to capture the observable LVR. This reveals the true scale of value leakage from LPs and highlights the opportunities to address it.

LVR.wtf measures potential CEX–DEX arbitrage by empirically simulating trades across the top 22 token pairs. For each block, we compare the V2/V3 pool price to the best bid or ask from Binance, OKX, Bybit, and Coinbase. If a price gap exceeds the pool’s fee range, the simulation calculates how much volume must trade on both the CEX and the DEX to eliminate it. This process accounts for slippage and a 0.01725% taker fee, emulating realistic market liquidity.

We repeat this process for nine different orderbook snapshots around the block time (T + 0, ± 0.5s, ±1s, ±1.5s, ±2s), then aggregate results for every block up to block 20,000,000. The outcome reveals the observable profit potential for CEX–DEX arbitrage across each pool at its corresponding markout.

Brontes data is also available in the dashboard for comparison. We plan to extend historical coverage on a rolling basis. For more in-depth details on our simulation methodology and architecture, refer to our GitBook (https://lvr-wtf.gitbook.io/lvr.wtf-doc)