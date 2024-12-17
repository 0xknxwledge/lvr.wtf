import React, { useState } from 'react';
import RunningTotalChart from '../components/plots/RunningTotalChart';
import EfficiencyRatioChart from '../components/plots/RealizedRatioChart';
import PoolTotalsPieChart from '../components/plots/PieChart';
import MaxLVRChart from '../components/plots/MaxLVRChart';
import QuartilePlot from '../components/plots/QuartilePlot';
import MarkoutSelect from '../components/select/MarkoutSelect';

const All: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  return (
    <div className="min-h-full bg-[#030304]">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <h1 className="text-4xl font-bold mb-8 text-white">All LVR</h1>
        
        {/* Standard chart container class with consistent spacing */}
        <div className="space-y-12"> {/* Increased from space-y-8 to space-y-12 for more breathing room */}
          {/* Running Totals Section */}
          <div className="chart-container bg-[#000000] rounded-2xl border border-[#212121] p-8">
            <RunningTotalChart />
          </div>

          {/* Realized Ratios Section */}
          <div className="chart-container bg-[#000000] rounded-2xl border border-[#212121] p-8">
            <EfficiencyRatioChart />
          </div>

          {/* Markout-dependent charts section */}
          <div className="space-y-12"> {/* Consistent spacing for this section too */}
            <div className="flex justify-end mb-6">
              <MarkoutSelect 
                selectedMarkout={selectedMarkout} 
                onChange={setSelectedMarkout}
              />
            </div>
            
            <div className="chart-container bg-[#000000] rounded-2xl border border-[#212121] p-8">
              <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
            </div>
            
            <div className="chart-container bg-[#000000] rounded-2xl border border-[#212121] p-8">
              <MaxLVRChart selectedMarkout={selectedMarkout} />
            </div>
            
            <div className="chart-container bg-[#000000] rounded-2xl border border-[#212121] p-8">
              <QuartilePlot selectedMarkout={selectedMarkout} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default All;