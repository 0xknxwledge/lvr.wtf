import React from 'react';
import { Link } from 'react-router-dom';

function Overview() {
  return (
    <div className="py-12">
      <h1 className="text-6xl font-bold text-center mb-12">Overview</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {/* First card - subtle gradient from dark to sage */}
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#B2AC88]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#b4d838]">What is lvr.wtf?</h2>
            <hr className="border-[#B2AC88]/20 mb-8" />
            <p className="text-white/90 text-base">
              Loss-Versus-Rebalancing (LVR), as defined by Millonis et al., measures the cost on-chain liquidity providers (LPs) face from trading at outdated prices compared to centralized exchanges (CEXs). While AMMs update prices every 12 seconds, CEXs operate in real-time, enabling arbitrageurs to profit from price gaps. To quantify this, Brontes flagged potential CEX-DEX arbitrage trades on Ethereum and estimated profits using T+X markouts against Binance mid-prices. However, its accuracy depends on correctly identifying arbitrage trades. LVR.wtf builds on this by comparing Brontes with alternative methods, offering a new perspective on LVR through empirical simulation that attempts to capture the observable maximum LVR. This reveals the true scale of value leakage from LPs and highlights the opportunities to address it.
            </p>
          </div>
        </div>

        {/* Second card - different sage gradient variation */}
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#B2AC88]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#b4d838]">Methodology</h2>
            <hr className="border-[#B2AC88]/20 mb-8" />
            <p className="text-white/90 text-base">
              As of December ***, we have simulated from the Merge up until block 20,000,000 (June 1st, 2024). We calculate a theoretical maximum LVR per pool, per markout, per block. We do so through snapshotting orderbook data so that every block has a corresponding orderbook state to use for the off-chain price. We use XYZ CEXs Then, we basically buy/sell on the orderbook snapshot and sell/buy (using Uniswap Quoter library) on the AMM until the prices converge. ***Include graph that Ryan made here*** The orderbook snapshots are fresh when simulating each pool's respective LVR. We ensure that simulated arbs only occur when the price discrepancy is larger than the fee of the AMM

              **Section addressing future improvements e.g, pool-priority for orderbook state, capital efficiency thresholds, etc.**
            </p>
          </div>
        </div>
      </div>

      {/* Dashboard access button */}
      <div className="mt-12 flex justify-center">
        <Link 
          to="/aggregate" 
          className="inline-flex items-center px-6 py-4 rounded-[9.75rem] bg-[#0b0b0e] border border-[#B2AC88]/50 
                     text-[#b4d838] text-lg font-medium hover:border-[#b4d838] hover:bg-[#B2AC88]/10 
                     transition-all duration-300"
        >
          <span className="mr-2">⟳</span> Access Data Dashboard
        </Link>
      </div>
    </div>
  );
}

export default Overview;