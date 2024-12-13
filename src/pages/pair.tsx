import React, { useState } from 'react';
import PoolTotalsPieChart from '../components/PieChart';
import MarkoutSelect from '../components/MarkoutSelect';
import HistogramChart from '../components/Histogram';
import SoleRunningTotal from '../components/SoleRunningTotal';
import NonZeroProportion from '../components/NonZeroProp';
import PercentileBandChart from '../components/BandPlot';
import names from '../names';

function Pair() {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');
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
          </div>
        </div>

        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6 mb-8">
          <h3 className="text-xl font-semibold mb-4">Running Total LVR</h3>
            <SoleRunningTotal 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
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

      <div className="space-y-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h3 className="text-xl font-semibold mb-4">Month-to-Month Daily Percentiles</h3>
            <PercentileBandChart 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
        </div>
    </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
        </div>
        <NonZeroProportion 
          poolAddress={selectedPool}
          selectedMarkout={selectedMarkout}
        />
      </div>
    </div>
  );
}

export default Pair;