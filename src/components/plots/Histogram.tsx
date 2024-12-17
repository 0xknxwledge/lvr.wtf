import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';

interface HistogramBucket {
  range_start: number;
  range_end: number | null;
  count: number;
  label: string;
}

interface HistogramResponse {
  pool_name: string;
  pool_address: string;
  buckets: HistogramBucket[];
  total_observations: number;
}

interface HistogramChartProps {
  poolAddress: string;
  markoutTime: string;
}

const HistogramChart: React.FC<HistogramChartProps> = ({ poolAddress, markoutTime }) => {
  const [data, setData] = useState<HistogramResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        const response = await fetch(
          `http://127.0.0.1:3000/histogram?pool_address=${poolAddress}&markout_time=${markoutTime}`
        );
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: HistogramResponse = await response.json();
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch histogram data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [poolAddress, markoutTime]);

  if (isLoading) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[400px] flex items-center justify-center">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[400px] flex items-center justify-center">
          <p className="text-red-500">{error || 'No data available'}</p>
        </div>
      </div>
    );
  }

  const xValues = data.buckets.map(bucket => bucket.label);
  const yValues = data.buckets.map(bucket => bucket.count);
  
  // Calculate percentage of total observations for hover text
  const percentages = yValues.map(count => 
    ((count / data.total_observations) * 100).toFixed(2)
  );

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix = markoutTime === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${markoutTime}s)`;

  return (
    <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
      <Plot
        data={[
          {
            type: 'bar',
            x: xValues,
            y: yValues,
            marker: {
              color: '#b4d838',
              opacity: 0.8,
            },
            hovertemplate: 
              'Range: %{x}<br>' +
              'Count: %{y}<br>' +
              'Percentage: %{customdata}%' +
              '<extra></extra>',
            customdata: percentages,
          }
        ]}
        layout={{
          title: {
            text: `Non-Zero Single-Block LVR Distribution for ${poolName} ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 }
          },
          xaxis: {
            title: {
              text: 'LVR Range ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            tickangle: 45,
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Number of Blocks',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
          },
          bargap: 0.1,
          autosize: true,
          height: 400,
          margin: { l: 80, r: 50, b: 100, t: 80, pad: 4 },
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
        }}
        config={{
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default HistogramChart;