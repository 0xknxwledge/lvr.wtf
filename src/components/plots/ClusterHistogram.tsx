import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';

interface HistogramBucket {
  range_start: number;
  range_end: number | null;
  count: number;
  label: string;
}

interface ClusterData {
  name: string;
  buckets: HistogramBucket[];
  total_observations: number;
}

interface ClusterHistogramProps {
  selectedMarkout: string;
}

const ClusterHistogram: React.FC<ClusterHistogramProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<ClusterData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`http://127.0.0.1:3000/clusters/histogram?${params.toString()}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData.clusters);
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
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-96">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-96">
          <p className="text-red-500">{error || 'No data available'}</p>
        </div>
      </div>
    );
  }

  // Create traces for each cluster
  const traces = data.map((cluster) => {
    const percentages = cluster.buckets.map(bucket => 
      (bucket.count / cluster.total_observations * 100).toFixed(2)
    );

    return {
      name: cluster.name,
      x: cluster.buckets.map(bucket => bucket.label),
      y: cluster.buckets.map(bucket => bucket.count),
      type: 'bar' as const,
      customdata: percentages,
      hovertemplate: 
        '<b>%{x}</b><br>' +
        'Count: %{y}<br>' +
        'Percentage: %{customdata}%' +
        '<extra></extra>',
    };
  });

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
      <Plot
        data={traces}
        layout={{
          title: {
            text: `Non-Zero Single-Block LVR Distribution by Cluster ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 }
          },
          barmode: 'group',
          xaxis: {
            title: {
              text: 'LVR Range ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff', size: 10 },
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
          showlegend: true,
          legend: {
            font: { color: '#ffffff' },
            bgcolor: '#000000',
            bordercolor: '#212121',
          },
          autosize: true,
          height: 500,
          margin: { l: 80, r: 50, b: 160, t: 80 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          hovermode: 'closest',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff' }
          },
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

export default ClusterHistogram;