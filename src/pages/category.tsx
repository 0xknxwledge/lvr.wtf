import React, { useState } from 'react';
import { MarkoutSelect } from '../components/LabeledSelect';
import CategoryHistogram from '../components/plots/CategoryHistogram';
import CategoryStackedBar from '../components/plots/CategoryStackedBar';
import CategoryNonZero from '../components/plots/CategoryNonZero';
import CategoryPieChart from '../components/plots/CategoryPie';
import PageLayout from '../components/pagelayout';

const Category: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <div className="bg-gradient-to-r from-[#0b0b0e] via-[#B2AC88]/5 to-[#0b0b0e] p-6 rounded-lg">
      <MarkoutSelect
        selectedMarkout={selectedMarkout}
        onChange={setSelectedMarkout}
      />
    </div>
  );

  return (
    <PageLayout title="Category Analysis" controls={controls}>
      <div className="text-[#B2AC88] text-lg mb-8 max-w-4xl mx-auto">
        <p className="text-center mb-4">
          View data grouped across similar pools. The categories are composed as follows:
        </p>
        
        {/* Category grid with sage green accents */}
        <div className="grid grid-cols-3 gap-6 mb-6">
          <CategorySection 
            title="Stablecoin Pairs" 
            items={[
              'USDC/USDT (0.01% fee)',
              'USDC/USDT (0.05% fee)',
              'DAI/USDC (0.01% fee)',
              'DAI/USDT (0.01% fee)',
              'USDe/USDT (0.01% fee)'
            ]}
          />
          <CategorySection 
            title="WBTC-WETH Pairs" 
            items={[
              'WBTC/WETH (0.05% fee)',
              'WBTC/WETH (0.30% fee)'
            ]}
          />
          <CategorySection 
            title="USDC-WETH Pairs" 
            items={[
              'USDC/WETH (0.05% fee)',
              'USDC/WETH (0.30% fee)',
              'WETH/USDC (Uniswap v2)'
            ]}
          />
        </div>

        <div className="grid grid-cols-3 gap-6 mb-6">
          <CategorySection 
            title="USDT-WETH Pairs" 
            items={[
              'WETH/USDT (0.01% fee)',
              'WETH/USDT (0.05% fee)',
              'WETH/USDT (0.30% fee)',
              'USDT/WETH (Uniswap v2)'
            ]}
          />
          <CategorySection 
            title="DAI-WETH Pairs" 
            items={[
              'DAI/WETH (0.05% fee)',
              'DAI/WETH (0.30% fee)'
            ]}
          />
          <CategorySection 
            title="USDC-WBTC Pairs" 
            items={[
              'WBTC/USDC (0.05% fee)',
              'WBTC/USDC (0.30% fee)'
            ]}
          />
        </div>

        <div className="flex justify-center">
          <CategorySection 
            title="Altcoin-WETH Pairs" 
            items={[
              'UNI/WETH (0.30% fee)',
              'PEPE/WETH (0.30% fee)',
              'PEPE/WETH (Uniswap v2)',
              'LINK/WETH (0.30% fee)'
            ]}
          />
        </div>
      </div>

      <div className="space-y-8">
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <CategoryPieChart selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <CategoryStackedBar selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <CategoryHistogram selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <CategoryNonZero selectedMarkout={selectedMarkout} />
        </div>
      </div>
    </PageLayout>
  );
};

// Helper component for category sections
const CategorySection: React.FC<{ title: string; items: string[] }> = ({ title, items }) => (
  <div className="bg-[#0b0b0e]/50 p-4 rounded-lg border border-[#B2AC88]/20">
    <h3 className="text-[#b4d838] font-medium mb-2 text-center">{title}</h3>
    <ul className="list-disc pl-6 space-y-1">
      {items.map((item, index) => (
        <li key={index} className="text-[#B2AC88]/90">{item}</li>
      ))}
    </ul>
  </div>
);

export default Category;