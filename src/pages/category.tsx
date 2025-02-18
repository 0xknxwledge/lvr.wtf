import React, { useState } from 'react';
import { MarkoutSelect } from '../components/LabeledSelect';
import CategoryHistogram from '../components/plots/CategoryHistogram';
import CategoryStackedBar from '../components/plots/CategoryStackedBar';
import CategoryNonZero from '../components/plots/CategoryNonZero';
import CategoryPieChart from '../components/plots/CategoryPie';
import PlotContainer from '../components/PlotContainer';

interface CategorySectionProps {
  title: string;
  items: string[];
}

const CategorySection: React.FC<CategorySectionProps> = ({ title, items }) => (
  <div className="bg-[#30283A]/50 p-4 rounded-lg border border-[#8247E5]/20">
    <h3 className="text-[#F651AE] font-medium mb-2 text-center">{title}</h3>
    <ul className="list-disc pl-4 space-y-2">
      {items.map((item, index) => (
        <li 
          key={index} 
          className="text-white/90 text-sm leading-tight break-words overflow-hidden"
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
    <div className="w-full flex flex-col sm:flex-row gap-4 justify-center items-center bg-[#030304] p-6 rounded-lg">
      <MarkoutSelect
        selectedMarkout={selectedMarkout}
        onChange={setSelectedMarkout}
      />
    </div>
  );

  return (
    <div className="font-['Geist'] px-4 sm:px-6 md:px-8 py-4 sm:py-6 md:py-8 bg-[#030304] min-h-screen">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold text-[#F651AE] mb-4 text-center">
          Category Analysis
        </h1>

        {controls}

        <div className="text-white text-lg my-8">
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
                'WETH/USDC (Uniswap V2)'
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
                'USDT/WETH (Uniswap V2)'
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

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            <div className="md:col-start-2">
              <CategorySection 
                title="Altcoin-WETH Pairs" 
                items={[
                  'UNI/WETH (0.30% fee)',
                  'PEPE/WETH (0.30% fee)',
                  'PEPE/WETH (Uniswap V2)',
                  'LINK/WETH (0.30% fee)'
                ]}
              />
            </div>
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
    </div>
  );
};

export default Category;