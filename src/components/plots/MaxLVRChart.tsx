import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';

interface PoolMaxLVR {
  pool_name: string;
  pool_address: string;
  block_number: number;
  lvr_cents: number;
}

interface MaxLVRChartProps {
  selectedMarkout: string;
}

const MaxLVRChart: React.FC<MaxLVRChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<PoolMaxLVR[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });
        
        const response = await fetch(`http://127.0.0.1:3000/max_lvr?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        // API returns array under 'pools' key
        setData(jsonData.pools);
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

  if (error || !data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[400px]">
        <p className="text-red-500">{error || 'No data available'}</p>
      </div>
    );
  }

  // Sort data by LVR
  const sortedData = [...data].sort((a, b) => b.lvr_cents - a.lvr_cents);

  // Calculate y-axis range and tick spacing
  const maxY = Math.max(...sortedData.map(d => d.lvr_cents / 100));
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <Plot
      data={[
        {
          x: sortedData.map(d => names[d.pool_address] || d.pool_name),
          y: sortedData.map(d => d.lvr_cents / 100),
          type: 'bar',
          marker: {
            color: '#b4d838',
            opacity: 0.8,
          },
          hovertemplate: 
            '<b>%{x}</b><br>' +
            'Maximum LVR: $%{y:,.2f}<br>' +
            'Block: %{customdata:,d}' +
            '<extra></extra>',
          customdata: sortedData.map(d => d.block_number),
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
          tickfont: { color: '#ffffff', size: 10 },
          tickangle: 45,
          fixedrange: true,
        },
        yaxis: {
          title: {
            text: 'Maximum LVR ($)',
            font: { color: '#b4d838', size: 14 },
            standoff: 30
          },
          tickformat: '$,.2f',
          tickfont: { color: '#ffffff' },
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
          zeroline: false,
          nticks: numTicks,
          range: [0, maxY * 1.1], // Add 10% padding at the top
          automargin: true,
        },
        showlegend: false,
        autosize: true,
        height: 400,
        margin: { l: 100, r: 50, b: 160, t: 80 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: '#b4d838',
          font: { color: '#ffffff' }
        },
        hovermode: 'closest'
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default MaxLVRChart;