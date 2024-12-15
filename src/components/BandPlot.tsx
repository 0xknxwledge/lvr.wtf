import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';
import dates from '../dates';

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
  const medianValues = data_points.map(d => d.median_cents / 100);
  const percentile25Values = data_points.map(d => d.percentile_25_cents / 100);
  const percentile75Values = data_points.map(d => d.percentile_75_cents / 100);

  const titleSuffix = markoutTime === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${markoutTime}s)`;

  const plotData = [
    // Median line with enhanced hover
    {
      x: dates,
      y: medianValues,
      type: 'scatter' as const,
      mode: 'lines' as const,
      name: 'Median',
      line: {
        color: '#b4d838',
        width: 2
      },
      customdata: data_points.map((d) => [
        d.percentile_25_cents / 100,
        d.median_cents / 100,
        d.percentile_75_cents / 100,
        d.block_number + 216000,
        d.block_number
      ]),
      hovertemplate: 
        '<b>Blocks %{customdata[4]} - %{customdata[3]}</b><br>' +
        '75th Percentile: %{customdata[2]:$,.2f}<br>' +
        'Median: %{customdata[1]:$,.2f}<br>' +
        '25th Percentile: %{customdata[0]:$,.2f}' +
        '<extra></extra>'
    },
    // Fill between percentiles
    {
      x: [...dates, ...dates.slice().reverse()],
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
              text: 'Date Range (UTC)',
              font: { color: '#b4d838', size: 14 },
              standoff: 30
            },
            tickfont: { color: '#ffffff', size: 10 },
            tickangle: 45,
            fixedrange: true,
            showgrid: false,
            automargin: true // Prevent x-axis label clipping
          },
          yaxis: {
            title: {
              text: 'Daily Total LVR',
              font: { color: '#b4d838', size: 14 },
              standoff: 30
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
          margin: { l: 100, r: 50, b: 140, t: 80 }, // Increased bottom margin
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          hovermode: 'x unified',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff', size: 12 }
          },
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