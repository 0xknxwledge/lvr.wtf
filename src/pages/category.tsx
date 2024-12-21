import React, { useState } from 'react';
import { MarkoutSelect } from '../components/LabeledSelect';
import CategoryHistogram from '../components/plots/CategoryHistogram';
import CategoryStackedBar from '../components/plots/CategoryStackedBar';
import CategoryNonZero from '../components/plots/CategoryNonZero';
import CategoryPieChart from '../components/plots/CategoryPie';
import PageLayout from '../components/pagelayout';

const Category: React.FC = () => {
  const [selectedMarkout, setSelectedMarkout] = useState('0.0');

  const controls = (
    <MarkoutSelect
      selectedMarkout={selectedMarkout}
      onChange={setSelectedMarkout}
    />
  );

  return (
    <PageLayout title="Category Analysis" controls={controls}>
      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryPieChart selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryStackedBar selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryHistogram selectedMarkout={selectedMarkout} />
      </div>

      <div className="bg-black rounded-2xl border border-[#212121] p-8">
        <CategoryNonZero selectedMarkout={selectedMarkout} />
      </div>
    </PageLayout>
  );
};

export default Category;