import React from 'react';
import { Link } from 'react-router-dom';

function Overview() {
  return (
    <div className="py-12 px-6 md:px-12 lg:px-24 font-['Menlo']">
      <h1 className="text-6xl font-bold text-center mb-12">Overview</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-[1600px] mx-auto">
        {/* First card - subtle gradient from dark to sage */}
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#B2AC88]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#b4d838] text-center">What is LVR.wtf?</h2>
            <hr className="border-[#B2AC88]/20 mb-8" />
            <p className="text-white/90 text-base">
              Loss-Versus-Rebalancing (LVR), as defined by Millonis et al., measures the cost on-chain liquidity providers (LPs) face from trading at outdated prices compared to centralized exchanges (CEXs). While AMMs update prices every 12 seconds, CEXs operate in real-time, enabling arbitrageurs to profit from price gaps.
               <br/> <br/>To quantify this, Brontes flagged potential CEX-DEX arbitrage trades on Ethereum and estimated profits using T+X markouts against Binance mid-prices. However, its accuracy depends on correctly identifying arbitrage trades. 
               <br/><br/>LVR.wtf builds on this by comparing Brontes with alternative methods, 
               offering a new perspective on LVR through empirical simulation that attempts to capture the observable maximum LVR. This reveals the true scale of value leakage from LPs and highlights the opportunities to address it.
            </p>
          </div>
        </div>

        {/* Second card - different sage gradient variation */}
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#B2AC88]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#b4d838] text-center">Methodology</h2>
            <hr className="border-[#B2AC88]/20 mb-8" />
            <p className="text-white/90 text-base">
            LVR.wtf empirically simulates how much CEX-DEX arbitrage can be extracted across the top 22 token pairs by comparing pool prices on V2/V3 pools to historical orderbook data from Binance, OKX, Bybit, and Coinbase. 
            For each block, the simulation checks the pool price against the best bid or ask in the CEX orderbooks and calculates how much volume needs to trade on both venues to eliminate any price gap exceeding the pool's fee range. 
            The approach accounts for slippage and a taker fee of 0.01725% to reflect realistic conditions for professional arbitrageurs. 
            <br/><br/>This is repeated for nine different orderbook snapshots at and around the block time (T ± 0.5, 1, 1.5, 2 seconds), then aggregated across every block up to block 20,000,000. 
            The resultant data quantifies the maximum profit potential for CEX-DEX arbitrage across all combinations of pool and markout time. Future updates will expand the historical coverage on a rolling basis.

            <br/><br/>The <a href="https://lvr-wtf.gitbook.io/lvr.wtf-doc" target="_blank" rel="noopener noreferrer" className="text-[#b4d838] hover:text-[#8B9556] transition-colors duration-200">gitbook</a> provides comprehensive details about our simulator's methodology.
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
          <span className="mr-2">⟳</span> Access Dashboard
        </Link>
      </div>
    </div>
  );
}

export default Overview;