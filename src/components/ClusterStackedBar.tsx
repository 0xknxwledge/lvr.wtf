import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';

interface MonthlyData {
  time_range: string;
  cluster_totals: { [key: string]: number };
  total_lvr_cents: number;
}

interface ClusterStackedBarResponse {
  monthly_data: MonthlyData[];
  clusters: string[];
}

interface ClusterStackedBarProps {
  selectedMarkout: string;
}

const ClusterStackedBar: React.FC<ClusterStackedBarProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<ClusterStackedBarResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`http://127.0.0.1:3000/clusters/monthly?${params.toString()}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData);
        console.log(jsonData);
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

  // Create traces for each cluster
  const traces = data.clusters.map((cluster) => ({
    name: cluster,
    x: data.monthly_data.map(d => d.time_range),
    y: data.monthly_data.map(d => d.cluster_totals[cluster] / 100), // Convert cents to dollars
    type: 'bar' as const,
    hovertemplate: 
      '<b>%{x}</b><br>' +
      '%{fullData.name}: $%{y:,.2f}<br>' +
      '<extra></extra>',
  }));

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <Plot
      data={traces}
      layout={{
        title: {
          text: `Monthly LVR by Cluster ${titleSuffix}`,
          font: { color: '#b4d838', size: 16 }
        },
        barmode: 'stack',
        xaxis: {
          title: {
            text: 'Time Range',
            font: { color: '#b4d838', size: 14 },
            standoff: 20
          },
          tickfont: { color: '#ffffff', size: 10 },
          tickangle: 45,
          fixedrange: true,
        },
        yaxis: {
          title: {
            text: 'Total LVR ($)',
            font: { color: '#b4d838', size: 14 },
            standoff: 20
          },
          tickfont: { color: '#ffffff' },
          tickformat: '$,.0f',
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
        },
        showlegend: true,
        legend: {
          font: { color: '#ffffff' },
          bgcolor: '#000000',
          bordercolor: '#212121',
          orientation: 'h',
          y: -0.2,
        },
        autosize: true,
        height: 500,
        margin: { l: 80, r: 50, b: 120, t: 80 },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        hovermode: 'x unified',
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
  );
};

export default ClusterStackedBar;