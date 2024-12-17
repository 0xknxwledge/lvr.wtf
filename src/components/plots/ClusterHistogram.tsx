import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';

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
        // Process the data to consolidate all buckets above $500 into a single $500+ bucket for each cluster
        const processedClusters = jsonData.clusters.map((cluster: ClusterData) => {
          const consolidatedBuckets = cluster.buckets.reduce((acc: HistogramBucket[], bucket: HistogramBucket) => {
            if (bucket.range_start < 500) {
              acc.push(bucket);
            } else {
              // Find existing $500+ bucket or create it
              let consolidatedBucket = acc.find(b => b.label === '$500+');
              if (!consolidatedBucket) {
                consolidatedBucket = {
                  range_start: 500,
                  range_end: null,
                  count: 0,
                  label: '$500+'
                };
                acc.push(consolidatedBucket);
              }
              consolidatedBucket.count += bucket.count;
            }
            return acc;
          }, []);
          
          return {
            ...cluster,
            buckets: consolidatedBuckets
          };
        });
        setData(processedClusters);
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

  const bucketOrder = [
    '$0.01-$10',
    '$10-$100',
    '$100-$500',
    '$500+'
  ];

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  // Create traces for each cluster with improved hover information
  const traces: Data[] = data.map((cluster) => {
    const orderedBuckets = [...cluster.buckets].sort((a, b) => 
      bucketOrder.indexOf(a.label) - bucketOrder.indexOf(b.label)
    );

    const percentages = orderedBuckets.map(bucket => 
      (bucket.count / cluster.total_observations * 100).toFixed(2)
    );

    const smallBarThreshold = cluster.total_observations * 0.001;
    const smallBars = orderedBuckets.filter(bucket => bucket.count < smallBarThreshold);
    
    const formatRangeText = (start: number, end: number | null) => {
      if (end === null) {
        return `$${start.toLocaleString()}+`;
      }
      return `$${start.toLocaleString()} to $${end.toLocaleString()}`;
    };

    const mainTrace: Data = {
      name: cluster.name,
      x: orderedBuckets.map(bucket => bucket.label),
      y: orderedBuckets.map(bucket => bucket.count),
      type: 'bar',
      customdata: orderedBuckets.map((bucket, idx) => [
        percentages[idx],
        bucket.range_start,
        bucket.range_end,
        cluster.total_observations,
        formatRangeText(bucket.range_start, bucket.range_end)
      ]),
      hovertemplate: 
        '<b>%{fullData.name}</b><br><br>' +
        'Count: %{y:,}<br>' +
        'Percentage: %{customdata[0]}%<br>' +
        'Range: %{customdata[4]}' +
        '<extra></extra>',
      hoverlabel: {
        align: 'left' as const
      },
      hoverinfo: 'skip' as const,
      hoveron: 'points' as const
    };

    const smallBarTrace: Data | null = smallBars.length > 0 ? {
      name: cluster.name,
      x: smallBars.map(bucket => bucket.label),
      y: smallBars.map(bucket => bucket.count),
      type: 'scatter',
      mode: 'markers',
      marker: {
        size: 10,
        opacity: 0
      },
      customdata: smallBars.map((bucket, idx) => [
        (bucket.count / cluster.total_observations * 100).toFixed(2),
        bucket.range_start,
        bucket.range_end,
        cluster.total_observations,
        formatRangeText(bucket.range_start, bucket.range_end)
      ]),
      hovertemplate: 
        '<b>%{fullData.name}</b><br><br>' +
        'Count: %{y:,}<br>' +
        'Percentage: %{customdata[0]}%<br>' +
        'Range: %{customdata[4]}' +
        '<extra></extra>',
      hoverlabel: {
        align: 'left' as const
      },
      showlegend: false,
      hoverinfo: 'skip' as const,
      hoveron: 'points' as const
    } : null;

    return smallBarTrace ? [mainTrace, smallBarTrace] : [mainTrace];
  }).flat();

  return (
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
          automargin: true,
          showgrid: false,
          categoryorder: 'array' as const,
          categoryarray: bucketOrder
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
          zeroline: true,
          zerolinecolor: '#404040'
        },
        showlegend: true,
        legend: {
          font: { color: '#ffffff' },
          bgcolor: '#000000',
          bordercolor: '#212121',
          x: 0.95,
          y: 0.95,
          xanchor: 'right',
          yanchor: 'top'
        },
        autosize: true,
        height: 500,
        margin: { 
          l: 80, 
          r: 50, 
          b: 160, 
          t: 80,
          pad: 10 
        },
        paper_bgcolor: '#000000',
        plot_bgcolor: '#000000',
        hovermode: 'closest',
        hoverdistance: 100,
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: '#b4d838',
          font: { color: '#ffffff', size: 12 },
          namelength: -1
        },
        bargap: 0.15,
        bargroupgap: 0.1
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default ClusterHistogram;