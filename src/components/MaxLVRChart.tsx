import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

interface MaxLVRPoolData {
  pool_name: string;
  pool_address: string;
  block_number: number;
  lvr_cents: number;
}

interface MaxLVRResponse {
  pools: MaxLVRPoolData[];
}

interface MaxLVRChartProps {
  selectedMarkout: string;
}

const MaxLVRChart: React.FC<MaxLVRChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<MaxLVRResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });

        const response = await fetch(`http://127.0.0.1:3000/max_lvr?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: MaxLVRResponse = await response.json();
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
      <div className="flex items-center justify-center h-[400px]">
        <p className="text-white">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-[400px]">
        <p className="text-red-500">{error || 'No data available'}</p>
      </div>
    );
  }

  // Sort pools by LVR in descending order
  const sortedPools = [...data.pools].sort((a, b) => b.lvr_cents - a.lvr_cents);

  // Convert data for plotting
  const xValues = sortedPools.map(pool => names[pool.pool_address] || pool.pool_name);
  const yValues = sortedPools.map(pool => pool.lvr_cents / 100); // Convert cents to dollars
  const blockNumbers = sortedPools.map(pool => pool.block_number);
  
  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
      <Plot
        data={[
          {
            type: 'bar',
            x: xValues,
            y: yValues,
            marker: {
              color: '#b4d838',
              opacity: 0.8,
            },
            hovertemplate: 
              '<b>%{x}</b><br>' +
              'Max LVR: $%{y:,.2f}<br>' +
              'Block: %{customdata:,d}' +
              '<extra></extra>',
            customdata: blockNumbers,
          }
        ]}
        layout={{
          title: {
            text: `Maximum Single-Block LVR by Pool ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 }
          },
          xaxis: {
            title: {
              text: 'Pool',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            tickangle: 45,
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Maximum LVR ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: '$,.2f',
            tickfont: { color: '#ffffff' },
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
          },
          bargap: 0.1,
          autosize: true,
          height: 500,
          margin: { l: 80, r: 50, b: 120, t: 80 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          font: { color: '#ffffff' },
          hovermode: 'closest',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff' }
          },
        }}
        config={{
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default MaxLVRChart;