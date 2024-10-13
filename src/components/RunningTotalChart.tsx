import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';

interface RunningTotalDataPoint {
  block_number: number;
  running_total: number;
}

const RunningTotalChart: React.FC = () => {
  const [runningTotalData, setRunningTotalData] = useState<RunningTotalDataPoint[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchRunningTotal = async () => {
      try {
        setIsLoading(true);
        setError(null);
        console.log('Fetching running total data...');
        const response = await fetch('http://127.0.0.1:5002/lvr_running_total');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data: RunningTotalDataPoint[] = await response.json();
        console.log('Running total data fetched successfully:', data);
        setRunningTotalData(data);
        setIsLoading(false);
      } catch (err) {
        console.error('Error fetching running total data:', err);
        setError(err instanceof Error ? err.message : 'An unexpected error occurred');
        setIsLoading(false);
      }
    };

    fetchRunningTotal();
  }, []);

  if (isLoading) return <div className="text-white">Loading...</div>;
  if (error) return <div className="text-white bg-red-600 p-4 rounded">{error}</div>;

  const blockNumbers = runningTotalData.map(item => item.block_number);
  const runningTotals = runningTotalData.map(item => item.running_total);

  return (
    <Plot
      data={[
        {
          x: blockNumbers,
          y: runningTotals,
          type: 'scatter',
          mode: 'lines',
          line: { color: '#b4d838' },
          name: 'Running Total LVR',
          hoverinfo: 'x+y',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff' }
          },
        },
      ]}
      layout={{
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
          tickformat: '$,.0f',
          tickfont: { color: '#ffffff' },
          side: 'right',
          showgrid: false,
          fixedrange: true,
          rangemode: 'tozero',
        },
        autosize: true,
        height: 600,
        margin: { l: 80, r: 100, b: 100, t: 80, pad: 4 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        font: { color: '#ffffff' },
        hovermode: 'closest',
        annotations: [
          {
            text: 'Running Total',
            font: { color: '#b4d838', size: 14 },
            showarrow: false,
            xref: 'paper',
            yref: 'paper',
            x: -0.07,
            y: 0.5,
            textangle: '-90',
          },
        ],
      }}
      config={{ 
        responsive: true,
        displayModeBar: false,
        scrollZoom: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default RunningTotalChart;