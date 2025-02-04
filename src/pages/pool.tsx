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

const Pool: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');
  const [selectedPool, setSelectedPool] = useState('0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640');

  const controls = (
    <div className="font-['Menlo'] w-full flex flex-col md:flex-row gap-4 justify-center items-center bg-gradient-to-r from-[#0b0b0e] via-[#B2AC88]/5 to-[#0b0b0e] p-6 rounded-lg">
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
    <PageLayout title="Pool Analysis" controls={controls}>
      <div className="max-w-7xl mx-auto">
        <p className="font-['Menlo'] text-[#B2AC88] text-lg mb-8 text-center">
          View data for individual pool and markout time combinations
        </p>
        
        <div className="font-['Menlo'] mt-4 mb-12 text-center">
          <p className="text-sm text-[#B2AC88]/80">
            *For the interquartile plot, we estimate percentile values through the t-digest algorithm and excluded blocks with zero simulated/observed LVR from the estimated distrbution. 
            For the bandplot, we excluded days (i.e, 7200-block-long chunks starting from the Merge block) that had zero simulated/observed LVR.
          </p>
        </div>

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

          <PlotContainer>
            <NonZeroProportion 
              poolAddress={selectedPool}
              selectedMarkout={selectedMarkout}
            />
          </PlotContainer>
        </div>
      </div>
    </PageLayout>
  );
};

export default Pool;