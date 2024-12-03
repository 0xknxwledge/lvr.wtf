import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import type { Dash } from 'plotly.js';

interface RunningTotal {
  block_number: number;
  markout: string;
  running_total_cents: number;
}

const RunningTotalChart: React.FC = () => {
  const [data, setData] = useState<RunningTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch('http://127.0.0.1:3000/running_total?start_block=15537392&end_block=20000000');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const rawData: RunningTotal[] = await response.json();
        setData(rawData);
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

  // Group data by markout time
  const groupedData = data.reduce((acc, item) => {
    if (!acc[item.markout]) {
      acc[item.markout] = {
        x: [],
        y: [],
        markout: item.markout
      };
    }
    acc[item.markout].x.push(item.block_number);
    acc[item.markout].y.push(item.running_total_cents / 100); // Convert cents to dollars
    return acc;
  }, {} as Record<string, { x: number[]; y: number[]; markout: string }>);

  const plotData = Object.values(groupedData).map(series => {
    const isBrontes = series.markout === 'brontes';
    
    return {
      x: series.x,
      y: series.y,
      type: 'scatter' as const,
      mode: 'lines' as const,
      name: isBrontes ? 'Observed LVR (Brontes)' : `Theoretical (${series.markout}s)`,
      line: {
        color: isBrontes ? '#b4d838' : '#4682B4',
        width: isBrontes ? 3 : 1,
        dash: isBrontes ? undefined : ('dot' as Dash)
      },
      opacity: isBrontes ? 1 : 0.3,
      hoverinfo: 'x+y' as const,
      hoverlabel: {
        bgcolor: '#424242',
        bordercolor: isBrontes ? '#b4d838' : '#4682B4',
        font: { color: '#ffffff' }
      }
    };
  });

  return (
    <div className="w-full">
      <Plot
        data={plotData}
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
          showlegend: true,
          legend: {
            x: 0,
            y: 1,
            bgcolor: '#000000',
            bordercolor: '#212121',
            font: { color: '#ffffff' }
          },
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
    </div>
  );
};

export default RunningTotalChart;