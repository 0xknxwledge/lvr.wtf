import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig, createAnnotationConfig } from '../plotUtils';

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
  const [selectedBucket, setSelectedBucket] = useState<{
    label: string;
    count: number;
    percentage: number;
  } | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/histogram?pool_address=${poolAddress}&markout_time=${markoutTime}`
        );
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: HistogramResponse = await response.json();
        
        // Process the data to consolidate all buckets above $500
        const consolidatedBuckets = jsonData.buckets.reduce((acc: HistogramBucket[], bucket: HistogramBucket) => {
          if (bucket.range_start < 500) {
            acc.push(bucket);
          } else {
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

        setData({
          ...jsonData,
          buckets: consolidatedBuckets
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch histogram data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [poolAddress, markoutTime]);

  const handleLabelClick = (label: string) => {
    if (!data) return;

    if (selectedBucket?.label === label) {
      // If clicking the same label, clear the selection
      setSelectedBucket(null);
    } else {
      // Find the bucket data and set the selection
      const bucket = data.buckets.find(b => b.label === label);
      if (bucket) {
        const percentage = (bucket.count / data.total_observations) * 100;
        setSelectedBucket({
          label: bucket.label,
          count: bucket.count,
          percentage: percentage
        });
      }
    }
  };

  if (isLoading) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[400px] flex items-center justify-center">
          <p className="text-white font-['Menlo']">Loading...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[400px] flex items-center justify-center">
          <p className="text-red-500 font-['Menlo']">{error || 'No data available'}</p>
        </div>
      </div>
    );
  }

  const bucketOrder = [
    '$0.01-$10',
    '$10-$100',
    '$100-$500',
    '$500+'
  ];

  const sortedBuckets = [...data.buckets].sort((a, b) => 
    bucketOrder.indexOf(a.label) - bucketOrder.indexOf(b.label)
  );

  const xValues = sortedBuckets.map(bucket => bucket.label);
  const yValues = sortedBuckets.map(bucket => bucket.count);
  const percentages = yValues.map(count => 
    ((count / data.total_observations) * 100).toFixed(2)
  );

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix = markoutTime === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${markoutTime}s)`;

  const title = `Per-Block LVR Histogram for ${poolName} ${titleSuffix}`;
  const baseLayout = createBaseLayout(title);

  // Create annotation for selected bucket
  const annotations = selectedBucket ? [{
    ...createAnnotationConfig({
      x: selectedBucket.label,
      y: yValues[xValues.indexOf(selectedBucket.label)],
      text: `Count: ${selectedBucket.count.toLocaleString()}<br>` +
            `Percentage: ${selectedBucket.percentage.toFixed(2)}%`,
      showarrow: true,
      arrowhead: 2,
      arrowsize: 1,
      arrowwidth: 2,
      arrowcolor: plotColors.accent,
      ay: -40,
      ax: 0,
    })
  }] : [];

  return (
    <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
      <Plot
        data={[
          {
            type: 'bar',
            x: xValues,
            y: yValues,
            marker: {
              color: plotColors.accent,
              opacity: 0.8,
            },
            hoverinfo: 'none',
            showlegend: false,
          }
        ]}
        layout={{
          ...baseLayout,
          xaxis: {
            ...baseLayout.xaxis,
            title: {
              text: 'LVR Range ($)',
              font: { color: plotColors.accent, size: fontConfig.sizes.axisTitle, family: fontConfig.family },
              standoff: 20
            },
            tickfont: { color: '#ffffff', size: fontConfig.sizes.axisLabel, family: fontConfig.family },
            tickangle: 45,
            fixedrange: true,
            categoryorder: 'array' as const,
            categoryarray: bucketOrder
          },
          yaxis: {
            ...baseLayout.yaxis,
            title: {
              text: 'Number of Blocks',
              font: { color: plotColors.accent, size: fontConfig.sizes.axisTitle, family: fontConfig.family },
              standoff: 20
            },
            tickfont: { color: '#ffffff', family: fontConfig.family },
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
          },
          bargap: 0.1,
          autosize: true,
          height: 400,
          margin: { l: 80, r: 50, b: 100, t: 80, pad: 4 },
          annotations: annotations,
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: plotColors.accent,
            font: { color: '#ffffff', size: fontConfig.sizes.hover, family: fontConfig.family },
          },
          hovermode: 'x unified'
        }}
        config={commonConfig}
        style={{ width: '100%', height: '100%' }}
      />
      
      {/* Clickable labels below the chart */}
      <div className="flex justify-center mt-8 gap-4">
        {xValues.map((label) => (
          <button
            key={label}
            onClick={() => handleLabelClick(label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 font-['Menlo'] ${
              selectedBucket?.label === label
                ? 'bg-[#b4d838] text-black font-medium'
                : 'bg-[#212121] text-white hover:bg-[#2a2a2a]'
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
};

export default HistogramChart;