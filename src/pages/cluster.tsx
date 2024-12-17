import React, { useState } from 'react';
import { MarkoutSelect } from '../components/LabeledSelect';
import ClusterHistogram from '../components/plots/ClusterHistogram';
import ClusterStackedBar from '../components/plots/ClusterStackedBar';
import ClusterNonZero from '../components/plots/ClusterNonZero';
import ClusterPieChart from '../components/plots/ClusterPie';
import PageLayout from '../components/pagelayout';

const Cluster: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <MarkoutSelect
      selectedMarkout={selectedMarkout}
      onChange={setSelectedMarkout}
    />
  );

  return (
    <PageLayout title="Cluster Analysis" controls={controls}>
      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <ClusterPieChart selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <ClusterStackedBar selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <ClusterHistogram selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <ClusterNonZero selectedMarkout={selectedMarkout} />
      </div>
    </PageLayout>
  );
};

export default Cluster;