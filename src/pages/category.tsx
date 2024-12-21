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
    <MarkoutSelect
      selectedMarkout={selectedMarkout}
      onChange={setSelectedMarkout}
    />
  );

  return (
    <PageLayout title="Category Analysis" controls={controls}>
<div className="text-gray-300 text-lg mb-8 max-w-4xl mx-auto">
  <p className="text-center mb-4">
    View data grouped across similar pools. The categories are composed as follows:
  </p>
  
  {/* First row of three */}
  <div className="grid grid-cols-3 gap-6 mb-6">
    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">Stablecoin Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>USDC/USDT (0.01% fee)</li>
        <li>USDC/USDT (0.05% fee)</li>
        <li>DAI/USDC (0.01% fee)</li>
        <li>DAI/USDT (0.01% fee)</li>
      </ul>
    </div>

    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">WBTC-WETH Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>WBTC/WETH (0.05% fee)</li>
        <li>WBTC/WETH (0.30% fee)</li>
      </ul>
    </div>

    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">USDC-WETH Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>USDC/WETH (0.05% fee)</li>
        <li>USDC/WETH (0.30% fee)</li>
      </ul>
    </div>
  </div>

  {/* Second row of three */}
  <div className="grid grid-cols-3 gap-6 mb-6">
    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">USDT-WETH Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>WETH/USDT (0.01% fee)</li>
        <li>WETH/USDT (0.05% fee)</li>
        <li>WETH/USDT (0.30% fee)</li>
      </ul>
    </div>

    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">DAI-WETH Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>DAI/WETH (0.05% fee)</li>
        <li>DAI/WETH (0.30% fee)</li>
      </ul>
    </div>

    <div>
      <h3 className="text-[#b4d838] font-medium mb-2">USDC-WBTC Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>WBTC/USDC (0.05% fee)</li>
        <li>WBTC/USDC (0.30% fee)</li>
      </ul>
    </div>
  </div>

  {/* Last row with single item, centered */}
  <div className="flex justify-center">
    <div className="w-1/3">
      <h3 className="text-[#b4d838] font-medium mb-2">Altcoin-WETH Pairs</h3>
      <ul className="list-disc pl-6 space-y-1">
        <li>UNI/WETH (0.30% fee)</li>
        <li>PEPE/WETH (0.30% fee)</li>
        <li>LINK/WETH (0.30% fee)</li>
      </ul>
    </div>
  </div>
</div>
      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryPieChart selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryStackedBar selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryHistogram selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryNonZero selectedMarkout={selectedMarkout} />
      </div>
    </PageLayout>
  );
};

export default Category;