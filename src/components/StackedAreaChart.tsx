import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';
import names from '../names';

interface DataPoint {
  block_number: number;
  markout: string;
  pool_name: string;
  running_total_cents: number;
}

interface StackedAreaChartProps {
  selectedMarkout: string;
}

const StackedAreaChart: React.FC<StackedAreaChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<DataPoint[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        const params = new URLSearchParams({
          aggregate: 'false'
        });

        if (selectedMarkout !== 'brontes') {
          params.append('markout_time', selectedMarkout);
        }

        const endpoint = `http://127.0.0.1:3000/running_total?${params.toString()}`;
        
        const response = await fetch(endpoint);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const jsonData: DataPoint[] = await response.json();
        setData(jsonData || []); // Ensure we always set an array, even if empty
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
      <div className="w-full bg-black rounded-lg border border-[#212121]">
        <div className="flex items-center justify-center h-[600px]">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full bg-black rounded-lg border border-[#212121]">
        <div className="flex items-center justify-center h-[600px]">
          <p className="text-red-500">{error}</p>
        </div>
      </div>
    );
  }

  if (!data || data.length === 0) {
    return (
      <div className="w-full bg-black rounded-lg border border-[#212121]">
        <div className="flex items-center justify-center h-[600px]">
          <p className="text-white">No data available</p>
        </div>
      </div>
    );
  }

  // Get unique block numbers and pool addresses
  const blockNumbers = Array.from(new Set(data.map(d => d.block_number))).sort((a, b) => a - b);
  const poolAddresses = Array.from(new Set(data.map(d => d.pool_name)));

  // Create traces for each pool
  const traces: Data[] = poolAddresses.map((poolAddress) => {
    // Get data points for this pool and sort by block number
    const poolData = data
      .filter(d => d.pool_name === poolAddress)
      .sort((a, b) => a.block_number - b.block_number);

    return {
      x: poolData.map(d => d.block_number),
      y: poolData.map(d => d.running_total_cents / 100), // Convert cents to dollars
      name: names[poolAddress] || `${poolAddress?.slice(0, 6)}...${poolAddress?.slice(-4)}`,
      type: 'scatter',
      mode: 'lines',
      stackgroup: 'one',
      hoverinfo: 'x+y+name' as const,
      hoverlabel: {
        bgcolor: '#424242',
        font: { color: '#ffffff' }
      },
    };
  });

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-semibold text-white">Running Total LVR by Pool</h2>
      </div>
      <Plot
        data={traces}
        layout={{
          title: {
            text: titleSuffix,
            font: { color: '#b4d838', size: 16 },
            x: 0.5,
            y: 0.95,
          },
          xaxis: {
            title: {
              text: 'Block Number',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: ',d',
            tickfont: { color: '#ffffff' },
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Running Total LVR ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 30
            },
            tickformat: '$,.0f',
            tickfont: { color: '#ffffff' },
            fixedrange: true,
          },
          showlegend: true,
          legend: {
            bgcolor: '#000000',
            font: { color: '#ffffff' },
            yanchor: 'top',
            y: -0.2,
            xanchor: 'left',
            x: 0,
            orientation: 'h'
          },
          autosize: true,
          height: 600,
          margin: { l: 80, r: 50, t: 50, b: 150 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          hovermode: 'closest',
        }}
        config={{
          responsive: true,
          displayModeBar: false,
          scrollZoom: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default StackedAreaChart;