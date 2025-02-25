import React from 'react';
import { Link } from 'react-router-dom';

function Overview() {
  return (
    <div className="py-12 px-6 md:px-12 lg:px-24 font-['Geist']">
      <h1 className="text-6xl font-bold text-center mb-12">Overview</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-[1600px] mx-auto">
        {/* First card - gradient from dark purple to pink */}
        <div className="bg-gradient-to-br from-[#30283A] via-[#8247E5]/20 to-[#F651AE]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#F651AE]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#F651AE] text-center">What is LVR.wtf?</h2>
            <hr className="border-[#8247E5]/20 mb-8" />
            <p className="text-white/90 text-base">
              Loss-Versus-Rebalancing (LVR), as defined by Millonis et al., measures the cost on-chain liquidity providers (LPs) face from trading at outdated prices compared to centralized exchanges (CEXs). While Automated Market Makers (AMMs) update prices every 12 seconds, CEXs operate in real-time, enabling arbitrageurs to profit from price gaps.
               <br/> <br/>To quantify this, Brontes flagged potential CEX-DEX arbitrage trades on Ethereum and estimated profits using T+0 markouts against Binance mid-prices. However, its accuracy depends on correctly identifying arbitrage trades. 
               <br/><br/>LVR.wtf builds on this by comparing Brontes with alternative methods, 
               offering a new perspective on LVR through empirical simulation that attempts to capture the observable LVR. This reveals the true scale of value leakage from LPs and highlights the opportunities to address it.
            </p>
          </div>
        </div>

        {/* Second card - different purple/pink gradient variation */}
        <div className="bg-gradient-to-br from-[#30283A] via-[#8247E5]/20 to-[#F651AE]/20 rounded-3xl p-10 relative overflow-hidden group">
          {/* Adding an overlay that becomes more visible on hover */}
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-[#F651AE]/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          
          <div className="relative z-10"> {/* Content stays above the overlay */}
            <h2 className="text-4xl font-semibold mb-8 text-[#F651AE] text-center">Methodology</h2>
            <hr className="border-[#8247E5]/20 mb-8" />
            <p className="text-white/90 text-base">
            LVR.wtf measures potential CEX–DEX arbitrage by empirically simulating trades across the top 22 token pairs. For each block, we compare the V2/V3 pool price to the best bid or ask from Binance, OKX, Bybit, and Coinbase. If a price gap exceeds the pool’s fee range, the simulation calculates how much volume must trade on both the CEX and the DEX to eliminate it. This process accounts for slippage and a 0.01725% taker fee, emulating realistic market liquidity.

            <br/><br/>We repeat this process for nine different orderbook snapshots around the block time (T + 0, ± 0.5s, ±1s, ±1.5s, ±2s), then aggregate results for every block up to block 20,000,000. The outcome reveals the observable profit potential for CEX–DEX arbitrage across each pool at its corresponding markout. 

            <br/><br/> Brontes data is also available in the dashboard for comparison. We plan to extend historical coverage on a rolling basis. For more in-depth details on our simulation methodology and architecture, refer to our <a href="https://lvr-wtf.gitbook.io/lvr.wtf-doc" target="_blank" rel="noopener noreferrer" className="text-[#F651AE] hover:text-[#FF7BC5] transition-colors duration-200">GitBook</a>
            </p>
          </div>
        </div>
      </div>

      {/* Dashboard access button */}
      <div className="mt-12 flex justify-center">
        <Link 
          to="/aggregate" 
          className="inline-flex items-center px-6 py-4 rounded-[9.75rem] bg-[#030304] border border-[#F651AE]/20 
          text-[#F651AE] text-lg font-medium hover:border-[#F651AE] hover:bg-[#F651AE]/10 
          transition-all duration-300"
        >
          <span className="mr-2">⟳</span> Access Dashboard
        </Link>
      </div>
    </div>
  );
}

export default Overview;