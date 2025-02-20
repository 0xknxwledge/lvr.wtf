import React, { useState } from 'react';
import RunningTotalChart from '../components/plots/RunningTotalChart';
import PoolTotalsPieChart from '../components/plots/PieChart';
import MaxLVRChart from '../components/plots/MaxLVRChart';
import { MarkoutSelect } from '../components/LabeledSelect';
import PlotContainer from '../components/PlotContainer';

const Aggregate: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <div className="w-full flex flex-col sm:flex-row gap-4 justify-center items-center bg-[#030304] p-6 rounded-lg">
      <MarkoutSelect 
        selectedMarkout={selectedMarkout} 
        onChange={setSelectedMarkout}
      />
    </div>
  );

  return (
    <div className="font-['Geist'] px-4 sm:px-6 md:px-8 py-4 sm:py-6 md:py-8 bg-[#030304] min-h-screen">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold text-[#F651AE] mb-4 text-center">
          Aggregate Analysis
        </h1>

        {controls}

        <p className="font-['Geist'] text-white text-lg my-8 text-center">
          View data aggregated across pools. 
          The last two plots are specific to the selected markout time.
        </p>

        <div className="flex flex-col">
          <PlotContainer>
            <RunningTotalChart />
          </PlotContainer>

          <PlotContainer>
            <PoolTotalsPieChart selectedMarkout={selectedMarkout} />
          </PlotContainer>

          <PlotContainer>
            <MaxLVRChart selectedMarkout={selectedMarkout} />
          </PlotContainer>
        </div>
      </div>
    </div>
  );
};

export default Aggregate;