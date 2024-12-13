import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';
import names from '../names';

interface PoolBoxplotData {
  pool_name: string;
  pool_address: string;
  percentile_25_cents: number;
  median_cents: number;
  percentile_75_cents: number;
  max_lvr_cents: number;
  max_lvr_block: number;
}

interface BoxplotLVRResponse {
  markout_time: string;
  pool_data: PoolBoxplotData[];
}

interface BoxPlotProps {
  selectedMarkout: string;
}

const BoxPlot: React.FC<BoxPlotProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<BoxplotLVRResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch(`http://127.0.0.1:3000/boxplot_lvr?markout_time=${selectedMarkout}`);
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
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-white">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-red-500">{error || 'No data available'}</p>
      </div>
    );
  }

  // Sort pools by median LVR
  const sortedPools = [...data.pool_data].sort((a, b) => b.median_cents - a.median_cents);
  const poolNames = sortedPools.map(pool => names[pool.pool_address] || pool.pool_name);

  const plotData: Data[] = [
    // 25th to 75th percentile range bars
    {
      name: 'IQR Range',
      x: poolNames,
      y: sortedPools.map(pool => pool.percentile_75_cents / 100),
      type: 'bar',
      marker: {
        color: 'rgba(180, 216, 56, 0.3)',
        line: {
          color: '#b4d838',
          width: 2
        }
      },
      width: 0.6,
      hovertemplate: '%{x}<br>75th: $%{y:.2f}<extra></extra>',
      showlegend: false,
    },
    {
      name: 'IQR Range',
      x: poolNames,
      y: sortedPools.map(pool => pool.percentile_25_cents / 100),
      type: 'bar',
      marker: {
        color: 'rgba(180, 216, 56, 0.3)',
        line: {
          color: '#b4d838',
          width: 2
        }
      },
      width: 0.6,
      hovertemplate: '%{x}<br>25th: $%{y:.2f}<extra></extra>',
      showlegend: false,
    },
    // Median lines
    {
      name: 'Median',
      x: poolNames,
      y: sortedPools.map(pool => pool.median_cents / 100),
      type: 'scatter',
      mode: 'lines',
      line: {
        color: '#b4d838',
        width: 3
      },
      hovertemplate: '%{x}<br>Median: $%{y:.2f}<extra></extra>',
    }
  ];

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  // Calculate y-axis range to show some padding above the highest 75th percentile
  const maxValue = Math.max(...sortedPools.map(p => p.percentile_75_cents / 100));
  const yAxisMax = maxValue * 1.1; // Add 10% padding

  return (
    <Plot
      data={plotData}
      layout={{
        title: {
          text: `LVR Distribution by Pool ${titleSuffix}`,
          font: { color: '#b4d838', size: 16 }
        },
        yaxis: {
          title: {
            text: 'LVR ($)',
            font: { color: '#b4d838', size: 14 },
            standoff: 20
          },
          range: [0, yAxisMax],
          tickformat: '$,.2f',
          tickfont: { color: '#ffffff' },
          showgrid: true,
          gridcolor: '#212121',
          zeroline: true,
          zerolinecolor: '#424242'
        },
        xaxis: {
          tickangle: 45,
          tickfont: { color: '#ffffff', size: 10 },
          showgrid: false,
        },
        showlegend: true,
        legend: {
          x: 0,
          y: 1,
          bgcolor: '#000000',
          bordercolor: '#424242',
          font: { color: '#ffffff' }
        },
        autosize: true,
        height: 600,
        margin: { l: 80, r: 50, b: 150, t: 80, pad: 4 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        bargap: 0.15,
        barmode: 'overlay' as const,
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default BoxPlot;