import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface PoolQuartileData {
  pool_name: string;
  pool_address: string;
  percentile_25_cents: number;
  median_cents: number;
  percentile_75_cents: number;
}

interface QuartilePlotResponse {
  markout_time: string;
  pool_data: PoolQuartileData[];
}

interface QuartilePlotProps {
  selectedMarkout: string;
}

const QuartilePlot: React.FC<QuartilePlotProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<QuartilePlotResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });

        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/quartile_plot?${params.toString()}`);
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

  // Sort pools by median LVR for better visualization
  const sortedData = [...data.pool_data].sort((a, b) => b.median_cents - a.median_cents);
  const xPositions = Array.from({ length: sortedData.length }, (_, i) => i);

  // Calculate y-axis range and tick spacing
  const maxY = Math.max(...sortedData.map(d => d.percentile_75_cents / 100));
  const minY = 0; // Since we're dealing with LVR values
  const yRange = maxY - minY;
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);

  // Create separate traces for each pool
  const plotData: Data[] = sortedData.flatMap((pool, index) => [
    // Vertical line (Q1 to Q3)
    {
      type: 'scatter',
      x: [index, index],
      y: [pool.percentile_25_cents / 100, pool.percentile_75_cents / 100],
      mode: 'lines',
      line: { color: '#b4d838', width: 1 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Box
    {
      type: 'scatter',
      x: [
        index - 0.25,
        index + 0.25,
        index + 0.25,
        index - 0.25,
        index - 0.25
      ],
      y: [
        pool.percentile_25_cents / 100,
        pool.percentile_25_cents / 100,
        pool.percentile_75_cents / 100,
        pool.percentile_75_cents / 100,
        pool.percentile_25_cents / 100
      ],
      fill: 'toself',
      fillcolor: 'rgba(180, 216, 56, 0.2)',
      line: { color: '#b4d838', width: 1 },
      mode: 'lines',
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Median line
    {
      type: 'scatter',
      x: [index - 0.25, index + 0.25],
      y: [pool.median_cents / 100, pool.median_cents / 100],
      mode: 'lines',
      line: { color: '#b4d838', width: 2 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Invisible hover area
    {
      type: 'scatter',
      x: [index],
      y: [pool.median_cents / 100],
      mode: 'markers',
      marker: { 
        color: 'rgba(0,0,0,0)',
        size: 20,
      },
      showlegend: false,
      hovertemplate: 
        '<b>%{text}</b><br>' +
        '75th Percentile: $%{customdata[2]:,.2f}<br>' +
        'Median: $%{customdata[1]:,.2f}<br>' +
        '25th Percentile: $%{customdata[0]:,.2f}' +
        '<extra></extra>',
      text: [names[pool.pool_address] || pool.pool_name],
      customdata: [
        pool.percentile_25_cents / 100,
        pool.median_cents / 100,
        pool.percentile_75_cents / 100
      ]
    }
  ]);

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <Plot
      data={plotData}
      layout={{
        title: {
          text: `Daily LVR Box Plots by Pool ${titleSuffix}*`,
          font: { color: '#b4d838', size: 16, family: fontConfig.family}
        },
        xaxis: {
          ticktext: sortedData.map(d => names[d.pool_address] || d.pool_name),
          tickvals: xPositions,
          tickfont: { color: '#ffffff', size: 10 },
          tickangle: 45,
          fixedrange: true,
        },
        yaxis: {
          title: {
            text: 'Daily Total LVR',
            font: { color: '#b4d838', size: 14, family: fontConfig.family},
            standoff: 30
          },
          tickformat: '$,.2f',
          tickfont: { color: '#ffffff' },
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
        height: 600,
        margin: { l: 120, r: 50, b: 160, t: 80 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: '#b4d838',
          font: { color: '#ffffff', family: fontConfig.family }
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

export default QuartilePlot;
