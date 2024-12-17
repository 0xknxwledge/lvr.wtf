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