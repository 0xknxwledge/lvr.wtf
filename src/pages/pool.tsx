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
    <>
      <PoolSelect
        selectedPool={selectedPool}
        onChange={setSelectedPool}
        names={names}
      />
      <MarkoutSelect 
        selectedMarkout={selectedMarkout} 
        onChange={setSelectedMarkout}
      />
    </>
  );

  return (
    <PageLayout title="Pool Analysis" controls={controls}>
        <p className="text-gray-300 text-lg mb-8 max-w-4xl mx-auto text-center">
        View data for individual pool and markout time combinations
        </p>
        <div className="mt-12 text-center">
    <p className="text-sm text-gray-400">
      *We exclude days (i.e, 7200-block-long chunks starting from the Merge block)
      that had zero simulated LVR activity
    </p>
  </div>
      <SoleRunningTotal 
        poolAddress={selectedPool}
        markoutTime={selectedMarkout}
      />

      <HistogramChart 
        poolAddress={selectedPool}
        markoutTime={selectedMarkout}
      />

      <PercentileBandChart 
        poolAddress={selectedPool}
        markoutTime={selectedMarkout}
      />

      <NonZeroProportion 
        poolAddress={selectedPool}
        selectedMarkout={selectedMarkout}
      />
    </PageLayout>
  );
};

export default Pool;