import React from 'react';
import RunningTotalChart from '../components/RunningTotalChart';
import EfficiencyRatioChart from '../components/EfficiencyRatioChart';

const All: React.FC = () => {
  return (
    <div className="min-h-full bg-[#030304]">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <h1 className="text-4xl font-bold mb-8 text-white">All LVR</h1>
        <div className="space-y-8">
          {/* Theoretical/Realized across all pools, for each markout time */}
          <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-6 text-white">Efficiency Ratios</h2>
            <EfficiencyRatioChart />
          </div>

          {/* Total LVR (across 22 pairs) */}
          <div className="bg-[#000000] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-6 text-white">Running Totals</h2>
            <RunningTotalChart />
          </div>
        </div>
      </div>
    </div>
  );
};

export default All;