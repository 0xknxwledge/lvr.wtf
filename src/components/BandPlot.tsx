import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

interface PercentileDataPoint {
  block_number: number;
  percentile_25_cents: number;
  median_cents: number;
  percentile_75_cents: number;
}

interface PercentileBandResponse {
  pool_name: string;
  pool_address: string;
  markout_time: string;
  data_points: PercentileDataPoint[];
}

interface PercentileBandChartProps {
  poolAddress: string;
  markoutTime: string;
}

const PercentileBandChart: React.FC<PercentileBandChartProps> = ({ poolAddress, markoutTime }) => {
  const [data, setData] = useState<PercentileBandResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime
        });

        const response = await fetch(`http://127.0.0.1:3000/percentile_band?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: PercentileBandResponse = await response.json();
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

  const { data_points } = data;
  const blockNumbers = data_points.map(d => d.block_number);
  const medianValues = data_points.map(d => d.median_cents / 100); // Convert to dollars
  const percentile25Values = data_points.map(d => d.percentile_25_cents / 100);
  const percentile75Values = data_points.map(d => d.percentile_75_cents / 100);

  const titleSuffix = markoutTime === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${markoutTime}s)`;

  const plotData = [
    // Median line
    {
      x: blockNumbers,
      y: medianValues,
      type: 'scatter' as const,
      mode: 'lines' as const,
      name: 'Median',
      line: {
        color: '#b4d838',
        width: 2
      },
      hovertemplate: 'Block: %{x}<br>Median: $%{y:.2f}<extra></extra>'
    },
    // Fill between 25th and 75th percentiles
    {
      x: [...blockNumbers, ...blockNumbers.slice().reverse()],
      y: [...percentile75Values, ...percentile25Values.slice().reverse()],
      fill: 'toself' as const,
      fillcolor: 'rgba(180, 216, 56, 0.2)',
      line: { color: 'rgba(180, 216, 56, 0.5)' },
      name: '25th-75th Percentile',
      showlegend: true,
      type: 'scatter' as const,
      mode: 'none' as const,
      hoverinfo: 'skip' as const
    }
  ];

  return (
    <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
      <Plot
        data={plotData}
        layout={{
          title: {
            text: `LVR Distribution Over Time ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 }
          },
          xaxis: {
            title: {
              text: 'Block Number',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: ',d',
            tickfont: { color: '#ffffff' },
            fixedrange: true,
            showgrid: false
          },
          yaxis: {
            title: {
              text: 'LVR ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: '$,.2f',
            tickfont: { color: '#ffffff' },
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121'
          },
          showlegend: true,
          legend: {
            x: 0,
            y: 1,
            bgcolor: '#000000',
            font: { color: '#ffffff' }
          },
          autosize: true,
          height: 400,
          margin: { l: 80, r: 50, b: 60, t: 80 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          hovermode: 'closest'
        }}
        config={{
          responsive: true,
          displayModeBar: false
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default PercentileBandChart;