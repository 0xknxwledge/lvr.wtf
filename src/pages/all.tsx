import React, { useState } from 'react';
import RunningTotalChart from '../components/RunningTotalChart';
import EfficiencyRatioChart from '../components/RealizedRatioChart';
import PoolTotalsPieChart from '../components/PieChart';
import MaxLVRChart from '../components/MaxLVRChart';
import QuartilePlot from '../components/QuartilePlot';
import MarkoutSelect from '../components/MarkoutSelect';

const All: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  return (
    <div className="min-h-full bg-[#030304]">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <h1 className="text-4xl font-bold mb-8 text-white">All LVR</h1>
        <div className="space-y-8">
          <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-6 text-white">Running Totals</h2>
            <RunningTotalChart />
          </div>

          <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-6 text-white">Realized Ratios</h2>
            <EfficiencyRatioChart />
          </div>

          <div>
            <div className="flex justify-end mb-4">
              <MarkoutSelect 
                selectedMarkout={selectedMarkout} 
                onChange={setSelectedMarkout}
              />
            </div>
            <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
              <h2 className="text-xl font-semibold mb-6 text-white">Proportion of total LVR (each pair)</h2>
              <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
            </div>
            <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
              <h2 className="text-xl font-semibold mb-6 text-white">Maximum LVR by Pool</h2>
              <MaxLVRChart selectedMarkout={selectedMarkout} />
            </div>
            <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
              <h2 className="text-xl font-semibold mb-6 text-white">Daily LVR Distribution by Pool</h2>
              <QuartilePlot selectedMarkout={selectedMarkout} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default All;