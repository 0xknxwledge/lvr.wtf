import React, { useState } from 'react';
import RunningTotalChart from '../components/plots/RunningTotalChart';
import EfficiencyRatioChart from '../components/plots/RealizedRatioChart';
import PoolTotalsPieChart from '../components/plots/PieChart';
import MaxLVRChart from '../components/plots/MaxLVRChart';
import { MarkoutSelect } from '../components/LabeledSelect';
import PageLayout from '../components/pagelayout';
import PlotContainer from '../components/PlotContainer';

const Aggregate: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <div className="font-['Menlo'] w-full flex flex-col md:flex-row gap-4 justify-center items-center bg-gradient-to-r from-[#0b0b0e] via-[#B2AC88]/5 to-[#0b0b0e] p-6 rounded-lg">
      <MarkoutSelect 
        selectedMarkout={selectedMarkout} 
        onChange={setSelectedMarkout}
      />
    </div>
  );

  return (
    <PageLayout title="Aggregate Analysis" controls={controls}>
      <div className="max-w-7xl mx-auto">
        <p className="font-['Menlo'] text-[#B2AC88] text-lg mb-8 text-center">
          View data aggregated across pools. The first two plots are aggregated across markout times. 
          The last two plots are specific to the selected markout time.
        </p>

        <div className="font-['Menlo'] mt-4 mb-12 text-center">
          <p className="text-sm text-[#B2AC88]/80">
            *Average marginal effect is estimated through simple beta regression
          </p>
        </div>

        <div className="flex flex-col">
          <PlotContainer>
            <RunningTotalChart />
          </PlotContainer>

          <PlotContainer>
            <EfficiencyRatioChart />
          </PlotContainer>

          <PlotContainer>
            <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
          </PlotContainer>

          <PlotContainer>
            <MaxLVRChart selectedMarkout={selectedMarkout} />
          </PlotContainer>
        </div>
      </div>
    </PageLayout>
  );
};

export default Aggregate;