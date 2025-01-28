import React, { useState } from 'react';
import { MarkoutSelect } from '../components/LabeledSelect';
import CategoryHistogram from '../components/plots/CategoryHistogram';
import CategoryStackedBar from '../components/plots/CategoryStackedBar';
import CategoryNonZero from '../components/plots/CategoryNonZero';
import CategoryPieChart from '../components/plots/CategoryPie';
import PlotContainer from '../components/PlotContainer';
import PageLayout from '../components/pagelayout';

const CategorySection: React.FC<{ title: string; items: string[] }> = ({ title, items }) => (
  <div className="bg-[#0b0b0e]/50 p-4 rounded-lg border border-[#B2AC88]/20">
    <h3 className="text-[#b4d838] font-medium mb-2 text-center">{title}</h3>
    <ul className="list-disc pl-4 space-y-2">
      {items.map((item, index) => (
        <li 
          key={index} 
          className="text-[#B2AC88]/90 text-sm leading-tight break-words overflow-hidden"
        >
          {item}
        </li>
      ))}
    </ul>
  </div>
);

const Category: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <div className="font-['Menlo'] w-full flex flex-col md:flex-row gap-4 justify-center items-center bg-gradient-to-r from-[#0b0b0e] via-[#B2AC88]/5 to-[#0b0b0e] p-6 rounded-lg">
      <MarkoutSelect
        selectedMarkout={selectedMarkout}
        onChange={setSelectedMarkout}
      />
    </div>
  );

  return (
    <PageLayout title="Category Analysis" controls={controls}>
      <div className="max-w-7xl mx-auto">
        <div className="text-[#B2AC88] text-lg mb-8">
          <p className="text-center mb-4">
            View data grouped across similar pools. The categories are composed as follows:
          </p>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
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

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
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

        <div className="flex flex-col">
          <PlotContainer>
            <CategoryPieChart selectedMarkout={selectedMarkout} />
          </PlotContainer>

          <PlotContainer>
            <CategoryStackedBar selectedMarkout={selectedMarkout} />
          </PlotContainer>

          <PlotContainer>
            <CategoryHistogram selectedMarkout={selectedMarkout} />
          </PlotContainer>

          <PlotContainer>
            <CategoryNonZero selectedMarkout={selectedMarkout} />
          </PlotContainer>
        </div>
      </div>
    </PageLayout>
  );
};

export default Category;