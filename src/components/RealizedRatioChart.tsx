import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';

interface MarkoutRatio {
  markout_time: string;
  ratio: number;
  realized_lvr_cents: number;
  theoretical_lvr_cents: number;
}

interface LVRRatioResponse {
  ratios: MarkoutRatio[];
}

const RealizedRatioChart: React.FC = () => {
  const [data, setData] = useState<MarkoutRatio[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch('http://127.0.0.1:3000/ratios?start_block=15537392&end_block=20000000');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const rawData: LVRRatioResponse = await response.json();
        setData(rawData.ratios);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-[#000000] rounded-lg border border-[#212121]">
        <div className="text-white text-lg">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-[#000000] rounded-lg border border-[#212121]">
        <div className="text-white bg-red-600 p-4 rounded">{error}</div>
      </div>
    );
  }

  // Sort data by markout time for consistent display
  const sortedData = [...data].sort((a, b) => {
    const aNum = parseFloat(a.markout_time);
    const bNum = parseFloat(b.markout_time);
    return aNum - bNum;
  });

  const mainTrace: Partial<Data> = {
    x: sortedData.map(d => parseFloat(d.markout_time)),
    y: sortedData.map(d => d.ratio),
    type: 'scatter',
    mode: 'lines+markers',
    line: {
      color: '#b4d838',
      width: 3,
    },
    marker: {
      color: '#b4d838',
      size: 8,
    },
    hovertemplate: 
      'Markout: %{x}s<br>' +
      'Realized: %{y:.1f}%<br>' +
      'Realized: $%{customdata[0]:,.2f}<br>' +
      'Theoretical: $%{customdata[1]:,.2f}<extra></extra>',
    customdata: sortedData.map(d => [
      d.realized_lvr_cents / 100,
      d.theoretical_lvr_cents / 100
    ]),
  };

  // Calculate linear regression
  const xValues = sortedData.map(d => parseFloat(d.markout_time));
  const yValues = sortedData.map(d => d.ratio);
  
  const n = xValues.length;
  const sumX = xValues.reduce((a, b) => a + b, 0);
  const sumY = yValues.reduce((a, b) => a + b, 0);
  const sumXY = xValues.reduce((acc, x, i) => acc + x * yValues[i], 0);
  const sumXX = xValues.reduce((acc, x) => acc + x * x, 0);
  
  const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
  const intercept = (sumY - slope * sumX) / n;

  const trendLine: Partial<Data> = {
    x: [Math.min(...xValues), Math.max(...xValues)],
    y: [
      slope * Math.min(...xValues) + intercept,
      slope * Math.max(...xValues) + intercept
    ],
    type: 'scatter',
    mode: 'lines',
    name: 'Trend',
    line: {
      color: 'rgba(180, 216, 56, 0.3)',
      width: 2,
      dash: 'dash',
    },
    hoverinfo: 'skip',
  };

  const plotData: Partial<Data>[] = [mainTrace, trendLine];

  return (
    <div className="w-full">
      <Plot
        data={plotData}
        layout={{
          title: {
            text: 'LVR Realized Ratio by Markout Time',
            font: { color: '#b4d838', size: 16 },
            y: 0.95
          },
          xaxis: {
            title: {
              text: 'Markout Time (seconds)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            zeroline: true,
            zerolinecolor: '#404040',
            gridcolor: '#212121',
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Realized Ratio (%)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: '.1f',
            ticksuffix: '%',
            tickfont: { color: '#ffffff' },
            range: [0, Math.max(...yValues) * 1.1],
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
          },
          autosize: true,
          height: 600,
          margin: { l: 80, r: 50, b: 80, t: 100, pad: 4 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          font: { color: '#ffffff' },
          hovermode: 'closest',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff' }
          },
          showlegend: false,
          annotations: [{
            x: 0,
            y: intercept,
            xref: 'x',
            yref: 'y',
            text: `Slope: ${slope.toFixed(2)}%/s`,
            showarrow: false,
            font: { color: '#b4d838' },
            bgcolor: 'rgba(0,0,0,0.7)',
            borderpad: 4,
          }]
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

export default RealizedRatioChart;