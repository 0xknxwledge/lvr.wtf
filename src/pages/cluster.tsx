import React, { useState } from 'react';
import ClusterPieChart from '../components/plots/ClusterPie';
import MarkoutSelect from '../components/select/MarkoutSelect';
import ClusterHistogram from '../components/plots/ClusterHistogram';
import ClusterStackedBar from '../components/plots/ClusterStackedBar';
import ClusterNonZero from '../components/plots/ClusterNonZero';

function Cluster() {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  return (
    <div className="p-8 bg-[#030304]">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-bold">Cluster Data</h1>
        <MarkoutSelect 
          selectedMarkout={selectedMarkout}
          onChange={setSelectedMarkout}
        />
      </div>

      <div className="space-y-12">
        <div className="bg-black rounded-2xl p-6">
          <h2 className="text-xl font-semibold mb-6">Proportion of total LVR (each cluster)</h2>
          <ClusterPieChart selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-black rounded-2xl p-6">
          <h2 className="text-xl font-semibold mb-6">Monthly LVR by Cluster</h2>
          <ClusterStackedBar selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-black rounded-2xl p-6">
          <h2 className="text-xl font-semibold mb-6">LVR Distribution by Cluster</h2>
          <ClusterHistogram selectedMarkout={selectedMarkout} />
        </div>

        <ClusterNonZero selectedMarkout={selectedMarkout} />
      </div>
    </div>
  );
}

export default Cluster;