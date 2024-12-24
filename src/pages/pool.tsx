import React, { useState } from 'react';
import { MarkoutSelect, PoolSelect }from '../components/LabeledSelect';
import HistogramChart from '../components/plots/Histogram';
import SoleRunningTotal from '../components/plots/SoleRunningTotal';
import NonZeroProportion from '../components/plots/NonZeroProp';
import PercentileBandChart from '../components/plots/BandPlot';
import names from '../names';
import PageLayout from '../components/pagelayout';

const Pool: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');
  const [selectedPool, setSelectedPool] = useState('0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640');

  const controls = (
    <div className="flex gap-6 bg-gradient-to-r from-[#0b0b0e] via-[#B2AC88]/5 to-[#0b0b0e] p-6 rounded-lg">
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
      <p className="text-[#B2AC88] text-lg mb-8 max-w-4xl mx-auto text-center">
        View data for individual pool and markout time combinations
      </p>
      
      <div className="mt-12 text-center">
        <p className="text-sm text-[#B2AC88]/80">
          *We exclude days (i.e, 7200-block-long chunks starting from the Merge block)
          that had zero simulated LVR activity
        </p>
      </div>

      <div className="space-y-8">
        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <SoleRunningTotal 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <HistogramChart 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <PercentileBandChart 
            poolAddress={selectedPool}
            markoutTime={selectedMarkout}
          />
        </div>

        <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
          <NonZeroProportion 
            poolAddress={selectedPool}
            selectedMarkout={selectedMarkout}
          />
        </div>
      </div>
    </PageLayout>
  );
};

export default Pool;