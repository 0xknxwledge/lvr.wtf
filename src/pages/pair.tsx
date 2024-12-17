import React, { useState } from 'react';
import MarkoutSelect from '../components/select/MarkoutSelect';
import HistogramChart from '../components/plots/Histogram';
import SoleRunningTotal from '../components/plots/SoleRunningTotal';
import NonZeroProportion from '../components/plots/NonZeroProp';
import PercentileBandChart from '../components/plots/BandPlot';
import names from '../names';

function Pair() {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');
  const [selectedPool, setSelectedPool] = useState('0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640'); // USDC/WETH(5bp)

  const poolOptions = Object.entries(names).map(([address, name]) => ({
    value: address,
    label: name
  }));

  return (
    <div className="p-8 bg-[#030304]">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-bold">Pair Analysis</h1>
        <div className="flex gap-4 items-center">
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

      <div className="space-y-12">
        <div className="bg-black rounded-2xl p-6">
          <h3 className="text-xl font-semibold mb-6">Running Total LVR</h3>
          <SoleRunningTotal 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>
        
        <div className="bg-black rounded-2xl p-6">
          <h3 className="text-xl font-semibold mb-6">LVR Distribution</h3>
          <HistogramChart 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>

        <div className="bg-black rounded-2xl p-6">
          <h3 className="text-xl font-semibold mb-6">Month-to-Month Daily Percentiles</h3>
          <PercentileBandChart 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>

        <div className="bg-black rounded-2xl p-6">
          <NonZeroProportion 
            poolAddress={selectedPool}
            selectedMarkout={selectedMarkout}
          />
        </div>
      </div>
    </div>
  );
}

export default Pair;