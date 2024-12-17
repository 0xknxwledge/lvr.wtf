import React from 'react';
import { Link } from 'react-router-dom';

function Overview() {
  return (
    <div className="py-12">
      <h1 className="text-6xl font-bold mb-12">Overview</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="bg-gradient-to-br from-[#0b0b0e] to-[#4b5c10] rounded-3xl border border-[#b4d838] p-10">
          <h2 className="text-4xl font-semibold mb-8">What is LVR?</h2>
          <hr className="border-[#3a3a3a] mb-8" />
          <p className="text-base">
            Loss-versus-revbalancing (LVR) measures the opportunity cost that on-chain liquidity providers face due to CEX-DEX (Centralized Exchange-Decentralized Exchange) arbitrage. **Cite millonis paper***
            So, what the fuck does that mean? 
            Essentially, liquidity providers on the blockchain are trading at an "uninformed" price at the start of every block. This happens because blockchains update their state every block, while centralized exchanges update their state practically instantly.
            Thankfully, arbitrageurs (i.e, the people who buy low on the DEX (CEX) and sell high on the CEX (DEX)) make sure that on-chain prices are in sync with off-chain basically every block.
            Unfortuneately, this is an existential threat to DeFi, because why the fuck would anyone want to liquidity provide on-chain if they're consistently and inevitably going to be trading at shitty prices?
            Thankfully, lots of smart people are working on solving this (shoutout Sorella Labs, maybe tuff competitors)

            **Add another section explaining markouts in intuitive manner, leads into purpose of lvr.wtf**
          </p>
        </div>
        <div className="bg-gradient-to-br from-[#0b0b0e] to-[#70881d] rounded-3xl border border-[#b4d838] p-10">
          <h2 className="text-4xl font-semibold mb-8">Methodology</h2>
          <hr className="border-[#3a3a3a] mb-8" />
          <p className="text-base">
            As of December ***, we have simulated from the Merge up until block 20,000,000 (June 1st, 2024). We calculate a theoretical maximum LVR per pool, per markout, per block. 
            We do so through snapshotting orderbook data so that every block has a corresponding orderbook state to use for the off-chain price. We use XYZ CEXs
            Then, we basically buy/sell on the orderbook snapshot and sell/buy (using Uniswap Quoter library) on the AMM until the prices converge.
            ***Include graph that Ryan made here***
            The orderbook snapshots are fresh when simulating each pool's respective LVR. We ensure that simulated arbs only occur when the price discrepancy is larger than the fee of the AMM

            **Section addressing future improvements e.g, pool-priority for orderbook state, capital efficiency thresholds, etc.**
         </p>
        </div>
      </div>
      <div className="mt-12">
        <Link to="/aggregate" className="inline-flex items-center px-6 py-4 rounded-[9.75rem] border border-[#b4d838] text-[#b4d838] text-lg font-medium">
          <span className="mr-2">⟳</span> Access Data Dashboard
        </Link>
      </div>
    </div>
  );
}

export default Overview;