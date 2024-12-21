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
    <MarkoutSelect 
      selectedMarkout={selectedMarkout} 
      onChange={setSelectedMarkout}
    />
  );

  return (
    <PageLayout title="Aggregate Analysis" controls={controls}>
        <p className="text-gray-300 text-lg mb-8 max-w-4xl mx-auto text-center">
          View data aggregated across pools. The first two plots are aggregated across markout times. 
          The last three plots are specific to the selected markout time.
        </p>
        <div className="mt-12 text-center">
    <p className="text-sm text-gray-400">
      *We exclude days (i.e, 7200-block-long chunks starting from the Merge block)
      that had zero simulated LVR activity. Additionally, we excluded showing maximum daily total LVR for the sake of 
      keeping the y-axis scale reasonable
    </p>
  </div>
      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <RunningTotalChart />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <EfficiencyRatioChart />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <QuartilePlot selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <MaxLVRChart selectedMarkout={selectedMarkout} />
      </div>
    </PageLayout>
  );
};


export default Aggregate;