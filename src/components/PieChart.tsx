import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

interface PoolTotal {
  pool_name: string;
  pool_address: string;
  total_lvr_cents: number;
}

interface PoolTotalsPieChartProps {
  selectedMarkout: string;
}

const PoolTotalsPieChart: React.FC<PoolTotalsPieChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<PoolTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams();
        params.append('markout_time', selectedMarkout);
        const response = await fetch(`http://127.0.0.1:3000/pool_totals?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const jsonData = await response.json();
        if (Array.isArray(jsonData.totals)) {
          setData(jsonData.totals);
        } else {
          throw new Error('Received invalid data format');
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
        setData([]); // Reset data to empty array on error
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-white">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-red-500">{error}</p>
      </div>
    );
  }

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-white">No data available</p>
      </div>
    );
  }

  // Sort data by total_lvr_cents in descending order
  const sortedData = [...data].sort((a, b) => b.total_lvr_cents - a.total_lvr_cents);

  // Calculate percentages and format labels
  const total = sortedData.reduce((sum, item) => sum + item.total_lvr_cents, 0);
  const values = sortedData.map(item => item.total_lvr_cents / 100); // Convert cents to dollars
  const labels = sortedData.map(item => {
    const poolName = names[item.pool_address] || `${item.pool_address.slice(0, 6)}...${item.pool_address.slice(-4)}`;
    const percentage = ((item.total_lvr_cents / total) * 100).toFixed(1);
    return `${poolName} (${percentage}%)`;
  });

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
              '#b4d838',
              '#9fc732',
              '#8ab62c',
              '#75a526',
              '#609420',
              '#4b831a',
              '#367214',
              '#21610e',
              '#0c5008',
              '#003f02'
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
          text: `Total LVR Distribution ${titleSuffix}`,
          font: { color: '#b4d838', size: 16 },
        },
        showlegend: false,
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        margin: { t: 50, b: 50, l: 50, r: 50 },
        height: 600,
        font: { color: '#ffffff' }
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default PoolTotalsPieChart;