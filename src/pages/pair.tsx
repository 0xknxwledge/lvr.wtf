import React, { useState } from 'react';
import StackedAreaChart from '../components/StackedAreaChart';
import PoolTotalsPieChart from '../components/PieChart';
import MarkoutSelect from '../components/MarkoutSelect';
import HistogramChart from '../components/Histogram';
import MaxLVRDisplay from '../components/MaxLVR';
import MedianLVR from '../components/MedianLVR';
import names from '../names';

function Pair() {
  const [selectedMarkout, setSelectedMarkout] = useState('brontes');
  const [selectedPool, setSelectedPool] = useState('0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640'); // USDC/WETH(5bp)

  const poolOptions = Object.entries(names).map(([address, name]) => ({
    value: address,
    label: name
  }));

  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-4xl font-bold">Pair Analysis</h1>
        <MarkoutSelect 
          selectedMarkout={selectedMarkout} 
          onChange={setSelectedMarkout} 
        />
      </div>

      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6 mb-8">
        <h3 className="text-xl font-semibold mb-4">Per-Block Median</h3>
        <MedianLVR 
        selectedMarkout={selectedMarkout}/>
      </div>

      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6 mb-8">
        <h3 className="text-xl font-semibold mb-4">Total LVR (across 22 pairs)</h3>
        <StackedAreaChart selectedMarkout={selectedMarkout} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">Proportion of total LVR (each pair)</h2>
          <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
        </div>
      </div>

      <div className="mb-8">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-4xl font-semibold">Individual Pairs</h2>
          <div className="space-x-4">
            <select
              value={selectedPool}
              onChange={(e) => setSelectedPool(e.target.value)}
              className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded cursor-pointer"
            >
              {poolOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <MarkoutSelect 
              selectedMarkout={selectedMarkout} 
              onChange={setSelectedMarkout}
            />
          </div>
        </div>
        
        <div className="space-y-8">
          <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h3 className="text-xl font-semibold mb-4">LVR Distribution</h3>
            <HistogramChart 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h3 className="text-xl font-semibold mb-4">Max LVR</h3>
          <MaxLVRDisplay 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>
      </div>
    </div>
  );
}

export default Pair;