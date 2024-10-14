import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

type Names = {
  [key: string]: string;
};

const typedNames = names as Names;

interface MedianLVRData {
  pool_address: string;
  median_lvr: number;
}

const MedianLVRChart: React.FC = () => {
  const [medianLVRData, setMedianLVRData] = useState<MedianLVRData[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMedianLVR = async () => {
      try {
        console.log('Attempting to fetch data...');
        const response = await fetch('https://lvr-wtf-568975696472.us-central1.run.app/median_lvr');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: MedianLVRData[] = await response.json();
        console.log('Data fetched successfully:', data);
        setMedianLVRData(data);
        setIsLoading(false);
      } catch (err) {
        console.error('Error details:', err);
        setError('Failed to fetch median LVR data. Please try again later.');
        setIsLoading(false);
      }
    };

    fetchMedianLVR();
  }, []);

  if (isLoading) return <p>Loading...</p>;
  if (error) return <p>{error}</p>;

  // Sort the data by median LVR in descending order
  const sortedData = [...medianLVRData].sort((a, b) => b.median_lvr - a.median_lvr);

  const poolAddresses = sortedData.map(item => item.pool_address);
  const medianLVRs = sortedData.map(item => item.median_lvr);
  const poolNames = poolAddresses.map(address =>
    typedNames[address] || `Unknown (${address.slice(0, 6)}...${address.slice(-4)})`
  );

  const maxLVR = Math.max(...medianLVRs);
  const yAxisMax = Math.ceil(maxLVR * 1.2); // 20% headroom above the highest bar

  return (
    <Plot
      data={[
        {
          x: poolNames,
          y: medianLVRs,
          type: 'bar',
          marker: {
            color: '#b4d838',
          },
          text: medianLVRs.map(value => `$${value.toFixed(2)}`),
          textposition: 'outside',
          textfont: {
            size: 12,
            color: 'white',
          },
          hoverinfo: 'x+y',
          hovertemplate: '%{x}<br>$%{y:.2f}<extra></extra>',
          width: 0.8,
        },
      ]}
      layout={{
        xaxis: {
          title: {
            text: 'Token Pair (Fee Tier)',
            font: {
              size: 14,
              color: '#b4d838',
            },
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
            text: 'Per-Block Median',
            font: {
              size: 14,
              color: '#b4d838',
            },
            standoff: 30,
          },
          tickformat: '$,.2f',
          tickfont: {
            size: 12,
            color: '#ffffff',
          },
          automargin: true,
          range: [0, yAxisMax],
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
        hovermode: 'closest',
      }}
      config={{
        responsive: true,
        scrollZoom: false,
        displayModeBar: false,
        staticPlot: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default MedianLVRChart;