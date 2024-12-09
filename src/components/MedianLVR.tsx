import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

interface PoolMedianLVR {
  pool_name: string;
  pool_address: string;
  median_lvr_cents: number;
}

interface MedianLVRResponse {
  medians: PoolMedianLVR[];
}

interface MedianLVRProps {
  selectedMarkout: string;
}

const MedianLVR: React.FC<MedianLVRProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<PoolMedianLVR[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams();
        if (selectedMarkout !== 'brontes') {
          params.append('markout_time', selectedMarkout);
        }
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/pool_medians?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const jsonData: MedianLVRResponse = await response.json();
        setData(jsonData.medians);
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

  if (error) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <p className="text-red-500">{error}</p>
      </div>
    );
  }

  // Sort data by median LVR in descending order
  const sortedData = [...data].sort((a, b) => b.median_lvr_cents - a.median_lvr_cents);

  const plotData = [{
    x: sortedData.map(d => names[d.pool_address] || `${d.pool_address.slice(0, 6)}...${d.pool_address.slice(-4)}`),
    y: sortedData.map(d => d.median_lvr_cents / 100), // Convert cents to dollars
    type: 'bar' as const,
    marker: {
      color: '#b4d838',
      opacity: 0.8,
    },
    text: sortedData.map(d => `$${(d.median_lvr_cents / 100).toFixed(2)}`),
    textposition: 'outside' as const,
    textfont: {
      size: 12,
      color: 'white',
    },
    hovertemplate: '%{x}<br>$%{y:.2f}<extra></extra>',
  }];

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <Plot
      data={plotData}
      layout={{
        title: {
          text: `Median LVR by Pool ${titleSuffix}`,
          font: { color: '#b4d838', size: 16 },
        },
        xaxis: {
          title: {
            text: 'Token Pair (Fee Tier)',
            font: { color: '#b4d838', size: 14 },
            standoff: 25,
          },
          tickangle: 45,
          tickfont: {
            size: 10,
            color: '#ffffff',
          },
          automargin: true,
          fixedrange: true,
        },
        yaxis: {
          title: {
            text: 'Median LVR',
            font: { color: '#b4d838', size: 14 },
            standoff: 30,
          },
          tickformat: '$,.2f',
          tickfont: {
            size: 12,
            color: '#ffffff',
          },
          automargin: true,
          fixedrange: true,
        },
        autosize: true,
        height: 600,
        margin: { l: 100, r: 50, b: 150, t: 80, pad: 4 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        font: { color: '#ffffff' },
        bargap: 0.05,
        bargroupgap: 0,
        showlegend: false,
        hovermode: 'closest',
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default MedianLVR;