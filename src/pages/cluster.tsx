import React, { useState } from 'react';
import ClusterPieChart from '../components/ClusterPie';
import MarkoutSelect from '../components/MarkoutSelect';
import ClusterHistogram from '../components/ClusterHistogram';
import ClusterStackedBar from '../components/ClusterStackedBar';

function Cluster() {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-bold">Cluster Data</h1>
        <MarkoutSelect 
          selectedMarkout={selectedMarkout}
          onChange={setSelectedMarkout}
        />
      </div>

      <div className="space-y-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">Proportion of total LVR (each cluster)</h2>
          <ClusterPieChart selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">Monthly LVR by Cluster</h2>
          <ClusterStackedBar selectedMarkout={selectedMarkout} />
        </div>

        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">LVR Distribution by Cluster</h2>
          <ClusterHistogram selectedMarkout={selectedMarkout} />
        </div>
      </div>
    </div>
  );
}

export default Cluster;