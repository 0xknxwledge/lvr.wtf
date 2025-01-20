import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import type { Data } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface QuartilePlotResponse {
  markout_time: string;
  pool_name: string;
  pool_address: string;
  percentile_25_cents: number;
  median_cents: number;
  percentile_75_cents: number;
}

interface QuartilePlotProps {
  poolAddress: string;
  markoutTime: string;
}

const QuartilePlot: React.FC<QuartilePlotProps> = ({ poolAddress, markoutTime }) => {
  const [data, setData] = useState<QuartilePlotResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime
        });

        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/quartile_plot?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonData: QuartilePlotResponse = await response.json();
        console.log(jsonData)
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [poolAddress, markoutTime]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-white">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500">{error || 'No data available'}</p>
      </div>
    );
  }

  // Calculate y-axis range and tick spacing
  const maxY = data.percentile_75_cents / 100;
  const minY = 0;
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);

  // Create the plot traces
  const plotData: Data[] = [
    // Vertical line (Q1 to Q3)
    {
      type: 'scatter',
      x: [0, 0],
      y: [data.percentile_25_cents / 100, data.percentile_75_cents / 100],
      mode: 'lines',
      line: { color: plotColors.accent, width: 1 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Box
    {
      type: 'scatter',
      x: [
        -0.25,
        0.25,
        0.25,
        -0.25,
        -0.25
      ],
      y: [
        data.percentile_25_cents / 100,
        data.percentile_25_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_25_cents / 100
      ],
      fill: 'toself',
      fillcolor: `${plotColors.accent}33`,
      line: { color: plotColors.accent, width: 1 },
      mode: 'lines',
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Median line
    {
      type: 'scatter',
      x: [-0.25, 0.25],
      y: [data.median_cents / 100, data.median_cents / 100],
      mode: 'lines',
      line: { color: plotColors.accent, width: 2 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Invisible hover area
    {
      type: 'scatter',
      x: [0],
      y: [data.median_cents / 100],
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
      text: [names[data.pool_address] || data.pool_name],
      customdata: [[
        data.percentile_25_cents / 100,
        data.median_cents / 100,
        data.percentile_75_cents / 100
      ]]
    }
  ];

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix = markoutTime === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${markoutTime}s)`;

  const title = `Daily LVR Quartile Plot for ${poolName} ${titleSuffix}*`;
  const baseLayout = createBaseLayout(title);

  return (
    <Plot
      data={plotData}
      layout={{
        ...baseLayout,
        showlegend: false,
        xaxis: {
          ...baseLayout.xaxis,
          showticklabels: false,
          zeroline: false,
          fixedrange: true,
          range: [-1, 1]
        },
        yaxis: {
          ...baseLayout.yaxis,
          title: {
            text: 'Daily Total LVR',
            font: { 
              color: plotColors.accent, 
              size: fontConfig.sizes.axisTitle,
              family: fontConfig.family 
            },
            standoff: 30
          },
          tickformat: '$,.2f',
          zeroline: false,
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
          nticks: numTicks,
          range: [0, maxY * 1.1],
          automargin: true
        },
        height: 500,
        margin: { l: 120, r: 50, b: 80, t: 100 },
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: plotColors.accent,
          font: { 
            color: '#ffffff',
            size: fontConfig.sizes.hover,
            family: fontConfig.family 
          }
        },
        hovermode: 'closest'
      }}
      config={commonConfig}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default QuartilePlot;