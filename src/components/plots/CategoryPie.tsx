import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { STABLE_POOLS, WBTC_WETH_POOLS, USDC_WETH_POOLS, USDT_WETH_POOLS, DAI_WETH_POOLS, USDC_WBTC_POOLS, ALTCOIN_WETH_POOLS } from '../../clusters';

interface CategoryData {
  name: string;
  total_lvr_cents: number;
}

interface CategoryPieResponse {
  clusters: CategoryData[];
  total_lvr_cents: number;
}

interface CategoryPieChartProps {
  selectedMarkout: string;
}

const CategoryPieChart: React.FC<CategoryPieChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryPieResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams();
        params.append('markout_time', selectedMarkout);
        
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/pie?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  if (isLoading) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-96">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-96">
          <p className="text-red-500">{error || 'No data available'}</p>
        </div>
      </div>
    );
  }

  // Calculate percentages manually
  const values = data.clusters.map(cluster => cluster.total_lvr_cents / 100); // Convert to dollars
  const percentages = data.clusters.map(cluster => 
    (cluster.total_lvr_cents / data.total_lvr_cents) * 100
  );
  
  const labels = data.clusters.map((cluster, index) => 
    `${cluster.name} (${percentages[index].toFixed(1)}%)`
  );

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
      <Plot
        data={[
          {
            values,
            labels,
            type: 'pie',
            textinfo: 'label',
            textposition: 'outside',
            automargin: true,
            marker: {
              colors: [
                '#b4d838', // Primary brand color
                '#9fc732',
                '#8ab62c',
                '#75a526',
                '#609420',
                '#4b831a',
                '#367214'
              ],
              line: {
                color: '#000000',
                width: 2
              }
            },
            hoverlabel: {
              bgcolor: '#424242',
              font: { color: '#ffffff' }
            },
            hovertemplate: '%{label}<br>$%{value:,.2f}<extra></extra>'
          }
        ]}
        layout={{
          title: {
            text: `Total LVR by Category ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 }
          },
          showlegend: false,
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          margin: { t: 50, b: 50, l: 50, r: 50 },
          height: 500,
          font: { color: '#ffffff' }
        }}
        config={{
          responsive: true,
          displayModeBar: false
        }}
        style={{ width: '100%', height: '100%' }}
      />
  );
};

export default CategoryPieChart;