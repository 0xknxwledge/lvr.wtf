import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { plotColors, createBaseLayout, commonConfig } from '../plotUtils';

interface HistogramBucket {
  range_start: number;
  range_end: number | null;
  count: number;
  label: string;
}

interface CategoryData {
  name: string;
  buckets: HistogramBucket[];
  total_observations: number;
}

interface CategoryHistogramProps {
  selectedMarkout: string;
}

// Updated color configuration
const CATEGORY_CONFIG = [
  { name: "Stable Pairs",   label: "Stable Pairs",   color: '#E2DFC9' },  // Light cream
  { name: "WBTC-WETH",      label: "WBTC-WETH",      color: '#738C3A' },  // Medium olive
  { name: "USDC-WETH",      label: "USDC-WETH",      color: '#A4C27B' },  // Sage green
  { name: "USDT-WETH",      label: "USDT-WETH",      color: '#2D3A15' },  // Dark forest
  { name: "DAI-WETH",       label: "DAI-WETH",       color: '#BAC7A7' },  // Light sage
  { name: "USDC-WBTC",      label: "USDC-WBTC",      color: '#4A5D23' },  // Deep forest
  { name: "Altcoin-WETH",   label: "Altcoin-WETH",   color: '#8B9556' }   // Muted olive
] as const;

const CategoryHistogram: React.FC<CategoryHistogramProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedLabel, setSelectedLabel] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/histogram?${params.toString()}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        const processedCategories = jsonData.clusters.map((cluster: CategoryData) => {
          const consolidatedBuckets = cluster.buckets.reduce((acc: HistogramBucket[], bucket: HistogramBucket) => {
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
          
          return {
            ...cluster,
            buckets: consolidatedBuckets
          };
        });

        const sortedCategories = CATEGORY_CONFIG
          .map(config => processedCategories.find((cat: CategoryData) => cat.name === config.name))
          .filter((cat: CategoryData | undefined): cat is CategoryData => cat !== undefined);

        setData(sortedCategories);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  const handleLabelClick = (label: string) => {
    setSelectedLabel(selectedLabel === label ? null : label);
  };

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

  const traces = data.map((cluster, index) => {
    const orderedBuckets = [...cluster.buckets].sort((a, b) => 
      bucketOrder.indexOf(a.label) - bucketOrder.indexOf(b.label)
    );

    const categoryConfig = CATEGORY_CONFIG[index];

    return {
      name: categoryConfig.label,
      x: orderedBuckets.map(bucket => bucket.label),
      y: orderedBuckets.map(bucket => bucket.count),
      type: 'bar',
      marker: { color: categoryConfig.color },
      hoverinfo: 'none'
    } as const;
  });

  const annotations = selectedLabel ? [{
    x: selectedLabel,
    y: Math.max(...traces.map(trace => {
      const bucketIndex = bucketOrder.indexOf(selectedLabel);
      return (trace.y?.[bucketIndex] as number) || 0;
    })),
    text: traces
      .map((trace, index) => {
        const bucketIndex = bucketOrder.indexOf(selectedLabel);
        const count = trace.y?.[bucketIndex] as number;
        if (!count || count === 0) return null;

        const cluster = data[index];
        if (!cluster) return null;

        const percentage = (count / cluster.total_observations) * 100;
        const categoryConfig = CATEGORY_CONFIG[index];
        
        return `<span style="color:${categoryConfig.color}">■</span> <b>${trace.name}</b>: ${count.toLocaleString()} (${percentage.toFixed(2)}%)`;
      })
      .filter(Boolean)
      .sort((a, b) => {
        const countA = parseInt(a!.split(': ')[1]);
        const countB = parseInt(b!.split(': ')[1]);
        return countB - countA;
      })
      .join('<br>'),
    showarrow: true,
    arrowhead: 2,
    arrowsize: 1,
    arrowwidth: 2,
    arrowcolor: '#b4d838',
    bgcolor: '#424242',
    bordercolor: '#b4d838',
    font: { color: '#ffffff', size: 12 },
    borderwidth: 2,
    borderpad: 4,
    ay: -40,
    ax: 0,
    align: 'left' as const
  }] : [];

  const baseLayout = createBaseLayout(`Per-Block LVR Histogram Grouped by Category ${titleSuffix}`);

  return (
    <>
      <Plot
        data={traces}
        layout={{
          ...baseLayout,
          barmode: 'group',
          xaxis: {
            ...baseLayout.xaxis,
            title: {
              text: 'LVR Range ($)',
              font: { color: '#FFFFFF', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#FFFFFF', size: 10 },
            tickangle: 45,
            categoryorder: 'array' as const,
            categoryarray: bucketOrder
          },
          yaxis: {
            ...baseLayout.yaxis,
            title: {
              text: 'Number of Blocks',
              font: { color: '#FFFFFF', size: 14 },
              standoff: 100
            },
            tickfont: { color: '#FFFFFF' },
          },
          height: 500,
          margin: { l: 150, r: 50, b: 160, t: 80 },
          annotations: annotations,
          legend: {
            font: { color: '#FFFFFF' },
            bgcolor: '#000000',
            bordercolor: '#212121',
            x: 1,
            y: 1.1,
            xanchor: 'right',
            yanchor: 'top',
          },
        }}
        config={commonConfig}
        style={{ width: '100%', height: '100%' }}
      />
      
      <div className="flex justify-center mt-8 gap-4">
        {bucketOrder.map((label) => (
          <button
            key={label}
            onClick={() => handleLabelClick(label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 ${
              selectedLabel === label
                ? 'bg-[#b4d838] text-black font-medium'
                : 'bg-[#212121] text-white hover:bg-[#2a2a2a]'
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </>
  );
};

export default CategoryHistogram;