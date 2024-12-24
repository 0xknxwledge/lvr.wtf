import React, { useState } from 'react';
import RunningTotalChart from '../components/plots/RunningTotalChart';
import EfficiencyRatioChart from '../components/plots/RealizedRatioChart';
import PoolTotalsPieChart from '../components/plots/PieChart';
import MaxLVRChart from '../components/plots/MaxLVRChart';
import QuartilePlot from '../components/plots/QuartilePlot';
import { MarkoutSelect } from '../components/LabeledSelect';
import PageLayout from '../components/pagelayout';

const Aggregate: React.FC = () => {
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
    <PageLayout title="Aggregate Analysis" controls={controls}>
      <p className="text-[#B2AC88] text-lg mb-8 max-w-4xl mx-auto text-center">
        View data aggregated across pools. The first two plots are aggregated across markout times. 
        The last three plots are specific to the selected markout time.
      </p>
      
      <div className="mt-12 text-center">
        <p className="text-sm text-[#B2AC88]/80">
          *We exclude days (i.e, 7200-block-long chunks starting from the Merge block)
          that had zero simulated LVR activity. Additionally, we excluded showing maximum daily total LVR for the sake of 
          keeping the y-axis scale reasonable
        </p>
      </div>

      <div className="space-y-8">
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <RunningTotalChart />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <EfficiencyRatioChart />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <QuartilePlot selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <MaxLVRChart selectedMarkout={selectedMarkout} />
        </div>
      </div>
    </PageLayout>
  );
};

export default Aggregate;