import React, { useState } from 'react';
import { MarkoutSelect, PoolSelect } from '../components/LabeledSelect';
import HistogramChart from '../components/plots/Histogram';
import SoleRunningTotal from '../components/plots/SoleRunningTotal';
import NonZeroProportion from '../components/plots/NonZeroProp';
import PercentileBandChart from '../components/plots/BandPlot';
import QuartilePlot from '../components/plots/QuartilePlot';
import PlotContainer from '../components/PlotContainer';
import names from '../names';
import PageLayout from '../components/pagelayout';
import DistributionMetrics from '../components/plots/DistributionMetrics';

const Pool: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');
  const [selectedPool, setSelectedPool] = useState('0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640');

  const controls = (
    <div className="w-full flex flex-col sm:flex-row gap-4 justify-center items-center bg-[#030304] p-6 rounded-lg">
      <PoolSelect
        selectedPool={selectedPool}
        onChange={setSelectedPool}
        names={names}
      />
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
          Pool Analysis
        </h1>
        
        {controls}

        <p className="font-['Geist'] text-white text-lg my-8 text-center">
          View data for individual pool and markout time combinations
        </p>

        <div className="flex flex-col">
          <PlotContainer>
            <SoleRunningTotal 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </PlotContainer>

          <PlotContainer>
            <QuartilePlot
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </PlotContainer>

          <PlotContainer>
            <DistributionMetrics
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </PlotContainer>

          <PlotContainer>
            <HistogramChart 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </PlotContainer>

          <PlotContainer>
            <PercentileBandChart 
              poolAddress={selectedPool}
              markoutTime={selectedMarkout}
            />
          </PlotContainer>
        </div>
      </div>
    </div>
  );
};
export default Pool;