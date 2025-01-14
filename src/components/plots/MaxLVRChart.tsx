import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

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
        
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/max_lvr?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
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
      <div className="flex items-center justify-center h-96">
        <p className="text-white font-['Menlo']">Loading...</p>
      </div>
    );
  }

  if (error || !data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500 font-['Menlo']">{error || 'No data available'}</p>
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
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  const title = `Maximum Single-Block LVR by Pool ${titleSuffix}`;
  const baseLayout = createBaseLayout(title);

  return (
    <Plot
      data={[
        {
          x: sortedData.map(d => names[d.pool_address] || d.pool_name),
          y: sortedData.map(d => d.lvr_cents / 100),
          type: 'bar',
          marker: {
            color: plotColors.accent,
            opacity: 0.8,
          },
          hovertemplate:
            '<b>%{x}</b><br>' +
            'Maximum LVR: $%{y:,.2f}<br>' +
            'Block: %{customdata:,d}' +
            '<extra></extra>',
          customdata: sortedData.map(d => d.block_number),
          width: 0.8,
          showlegend: false,
        }
      ]}
      layout={{
        ...baseLayout,
        xaxis: {
          ...baseLayout.xaxis,
          tickfont: { 
            color: '#ffffff', 
            size: fontConfig.sizes.axisLabel,
            family: fontConfig.family 
          },
          tickangle: 45,
          fixedrange: true,
          automargin: true,
        },
        yaxis: {
          ...baseLayout.yaxis,
          tickformat: '$,.2f',
          tickfont: { 
            color: '#ffffff',
            family: fontConfig.family 
          },
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
          zeroline: false,
          nticks: numTicks,
          range: [0, maxY * 1.1],
          automargin: true,
        },
        showlegend: false,
        autosize: true,
        height: 500,
        margin: { 
          l: 100,
          r: 50,
          b: 160,
          t: 80,
          pad: 10
        },
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: plotColors.accent,
          font: { 
            color: '#ffffff',
            size: fontConfig.sizes.hover,
            family: fontConfig.family 
          },
          namelength: 0
        },
        hovermode: 'x unified',
        hoverdistance: 50,
        bargap: 0.2,
      }}
      config={commonConfig}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default MaxLVRChart;