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

  // Color scheme for different markout times
  const markoutColors = {
    'brontes': '#b4d838',  // Bright lime green for observed
    '-2.0': '#FF6B6B',     // Red
    '-1.5': '#4ECDC4',     // Turquoise
    '-1.0': '#45B7D1',     // Light blue
    '-0.5': '#96CEB4',     // Sage green
    '0.0': '#FFBE0B',      // Yellow
    '0.5': '#FF006E',      // Pink
    '1.0': '#8338EC',      // Purple
    '1.5': '#3A86FF',      // Blue
    '2.0': '#FB5607'       // Orange
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch('http://127.0.0.1:3000/running_total?aggregate=true');
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
      name: isBrontes ? 'Observed (Brontes)' : `${series.markout}s`,
      line: {
        color: markoutColors[series.markout as keyof typeof markoutColors],
        width: isBrontes ? 3 : 2,
        dash: isBrontes ? undefined : ('solid' as Dash)
      },
      opacity: isBrontes ? 1 : 0.8,
      hoverinfo: 'x+y+name' as const,
      hoverlabel: {
        bgcolor: '#424242',
        bordercolor: markoutColors[series.markout as keyof typeof markoutColors],
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
          },
          yaxis: {
            tickformat: '$,.0f',
            tickfont: { color: '#ffffff' },
            side: 'right',
            showgrid: false,
            rangemode: 'tozero',
          },
          title: {
            text: 'Running Total LVR',
            font: { color: '#b4d838', size: 16 },
            y: 0.95
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
        }}
        config={{
          responsive: true,
          displayModeBar: true,
          displaylogo: false,
          modeBarButtonsToAdd: ['zoomIn2d', 'zoomOut2d', 'autoScale2d'],
          modeBarButtonsToRemove: ['lasso2d', 'select2d'],
          toImageButtonOptions: {
            format: 'png',
            filename: 'running_total_lvr',
            height: 600,
            width: 1200,
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default RunningTotalChart;