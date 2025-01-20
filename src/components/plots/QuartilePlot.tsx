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
          markout_time: markoutTime,
        });

        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/quartile_plot?${params.toString()}`
        );
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const jsonData: QuartilePlotResponse = await response.json();
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

  // Calculate x-axis range
  const maxX = data.percentile_75_cents / 100;

  // Create the plot traces
  const plotData: Data[] = [
    // IQR
    {
      type: 'scatter',
      x: [data.percentile_25_cents / 100, data.percentile_75_cents / 100],
      y: [0, 0],
      mode: 'lines',
      line: { color: plotColors.accent, width: 1 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    {
      type: 'scatter',
      x: [
        data.percentile_25_cents / 100,
        data.percentile_25_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_25_cents / 100,
      ],
      y: [-0.25, 0.25, 0.25, -0.25, -0.25],
      fill: 'toself',
      fillcolor: `${plotColors.accent}33`,
      line: { color: plotColors.accent, width: 1 },
      mode: 'lines',
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Median
    {
      type: 'scatter',
      x: [data.median_cents / 100, data.median_cents / 100],
      y: [-0.25, 0.25],
      mode: 'lines',
      line: { color: plotColors.accent, width: 2 },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Hover area
    {
      type: 'scatter',
      x: [data.median_cents / 100],
      y: [0],
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
      customdata: [
        [
          data.percentile_25_cents / 100,
          data.median_cents / 100,
          data.percentile_75_cents / 100,
        ],
      ],
    },
  ];

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix =
    markoutTime === 'brontes' ? '(Observed)' : `(Markout ${markoutTime}s)`;

  const title = `Single-Block LVR Interquartile Plot for ${poolName} ${titleSuffix}*`;
  const baseLayout = createBaseLayout(title);

  return (
    <Plot
      data={plotData}
      layout={{
        ...baseLayout,
        showlegend: false,
        xaxis: {
          ...baseLayout.xaxis,
          title: {
            font: {
              color: plotColors.accent,
              size: fontConfig.sizes.axisTitle,
              family: fontConfig.family,
            },
          },
          tickformat: '$,.2f',
          zeroline: false,
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
          range: [0, maxX * 1.1],
          automargin: true,
        },
        yaxis: {
          ...baseLayout.yaxis,
          showticklabels: false,
          zeroline: false,
          fixedrange: true,
          range: [-1, 1],
        },
        height: 300,
        margin: { l: 50, r: 50, b: 50, t: 100 },
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: plotColors.accent,
          font: {
            color: '#ffffff',
            size: fontConfig.sizes.hover,
            family: fontConfig.family,
          },
        },
        hovermode: 'closest',
      }}
      config={commonConfig}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default QuartilePlot;
