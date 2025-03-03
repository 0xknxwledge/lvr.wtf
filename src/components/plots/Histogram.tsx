import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig, createAnnotationConfig } from '../plotUtils';

interface HistogramBucket {
  range_start: number;
  range_end: number | null;
  count: number;
  label: string;
}

interface BucketWithPercentage extends HistogramBucket {
  percentage: number;
}

interface HistogramResponse {
  pool_name: string;
  pool_address: string;
  buckets: BucketWithPercentage[];
  total_observations: number;
}

interface HistogramChartProps {
  poolAddress: string;
  markoutTime: string;
}

// Helper function to compute exact percentages
function computeExactPercentages(buckets: HistogramBucket[], total: number): BucketWithPercentage[] {
  if (total === 0) {
    return buckets.map(bucket => ({ ...bucket, percentage: 0 }));
  }
  const rawPercentages = buckets.map(bucket => (bucket.count / total) * 100);
  const rounded = rawPercentages.map(p => Math.round(p * 100) / 100);
  const sumRounded = rounded.reduce((acc, p) => acc + p, 0);
  const delta = Math.round((100 - sumRounded) * 100) / 100;
  const maxIndex = buckets.reduce((maxIdx, bucket, idx) => bucket.count > buckets[maxIdx].count ? idx : maxIdx, 0);
  rounded[maxIndex] += delta;
  return buckets.map((bucket, idx) => ({
    ...bucket,
    percentage: rounded[idx]
  }));
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
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const getResponsiveLayout = useCallback(() => {
    const isMobile = windowWidth < 768;
    const isTablet = windowWidth >= 768 && windowWidth < 1024;

    return {
      height: isMobile ? 400 : 600,
      margin: {
        l: isMobile ? 80 : (isTablet ? 100 : 120),
        r: isMobile ? 30 : (isTablet ? 40 : 50),
        b: isMobile ? 100 : (isTablet ? 110 : 120),
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        pad: 4
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        annotation: isMobile ? 10 : (isTablet ? 11 : 12)
      },
      barWidth: isMobile ? 0.6 : 0.8
    };
  }, [windowWidth]);

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
        
        // Consolidate buckets (all buckets with range_start >= 500 are merged into one labeled ">$500")
        const consolidatedBuckets = jsonData.buckets.reduce((acc: HistogramBucket[], bucket: HistogramBucket) => {
          if (bucket.range_start < 500) {
            acc.push(bucket);
          } else {
            let consolidatedBucket = acc.find(b => b.label === '>$500');
            if (!consolidatedBucket) {
              consolidatedBucket = {
                range_start: 500,
                range_end: null,
                count: 0,
                label: '>$500'
              };
              acc.push(consolidatedBucket);
            }
            consolidatedBucket.count += bucket.count;
          }
          return acc;
        }, []);
        
        // Compute exact percentages that sum exactly to 100%
        const bucketsWithPercentages = computeExactPercentages(consolidatedBuckets, jsonData.total_observations);

        setData({
          ...jsonData,
          buckets: bucketsWithPercentages
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
      setSelectedBucket(null);
    } else {
      const bucket = data.buckets.find(b => b.label === label);
      if (bucket) {
        setSelectedBucket({
          label: bucket.label,
          count: bucket.count,
          percentage: bucket.percentage
        });
      }
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-white text-base md:text-lg font-['Geist']">Loading (may take up to 30 seconds)...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-red-500 text-sm md:text-base font-['Geist']">{error || 'No data available'}</p>
      </div>
    );
  }

  const bucketOrder = [
    '$0.01-$10',
    '$10-$100',
    '$100-$500',
    '>$500'
  ];

  const sortedBuckets = [...data.buckets].sort((a, b) => 
    bucketOrder.indexOf(a.label) - bucketOrder.indexOf(b.label)
  );

  // Ensure unique buckets in sorted data
  const uniqueBuckets = Array.from(new Set(sortedBuckets.map(b => b.label)))
    .map(label => sortedBuckets.find(b => b.label === label))
    .filter((b): b is HistogramBucket & { percentage: number } => b !== undefined);

  const xValues = uniqueBuckets.map(bucket => bucket.label);
  const yValues = uniqueBuckets.map(bucket => bucket.count);

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix = markoutTime === 'brontes' ? 
    '(Brontes)' : 
    `(Markout ${markoutTime}s)`;

  const isMobile = windowWidth <= 768;
  const title = isMobile ?
    `Single-Block LVR Histogram for<br>${poolName}<br>${titleSuffix}` :
    `Single-Block LVR Histogram for ${poolName} ${titleSuffix}`;

  const baseLayout = createBaseLayout(title);
  const responsiveLayout = getResponsiveLayout();

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
      arrowcolor: '#FFFFFF',
      ay: -40,
      ax: 0,
      font: {
        size: responsiveLayout.fontSize.annotation,
        family: fontConfig.family,
        color: '#ffffff'
      }
    })
  }] : [];

  const layout: Partial<Layout> = {
    ...baseLayout,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    xaxis: {
      ...baseLayout.xaxis,
      title: {
        text: 'LVR Range ($)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 15 : 20
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      tickangle: 45,
      fixedrange: true,
      categoryorder: 'array' as const,
      categoryarray: bucketOrder,
      showline: false
    },
    yaxis: {
      ...baseLayout.yaxis,
      title: {
        text: 'Number of Blocks',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 30 : 40
      },
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      fixedrange: true,
      showgrid: true,
      gridcolor: '#212121',
      showline: false
    },
    bargap: 0.1,
    title: {
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      },
      text:`<b>${title}</b>`,
    },
    annotations: annotations,
    hovermode: false
  };

  return (
    <div className="w-full">
      {/* User instruction text */}
      <div className="mb-6 text-center">
        <p className="text-white/80 text-sm md:text-base font-['Geist'] bg-[#30283A]/50 inline-block px-4 py-2 rounded-lg">
          Click on the bin buttons below the chart to view exact counts and percentages for a given LVR range
        </p>
      </div>

      {/* Main histogram plot */}
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
            width: responsiveLayout.barWidth
          }
        ]}
        layout={layout}
        config={{
          ...commonConfig,
          responsive: true,
          displayModeBar: false,
          toImageButtonOptions: {
            format: 'png',
            filename: `histogram_${poolAddress}`,
            height: responsiveLayout.height,
            width: windowWidth,
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
      />
      
      {/* Interactive bucket selection buttons */}
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {xValues.map((label) => (
          <button
            key={label}
            onClick={() => handleLabelClick(label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base
              ${selectedBucket?.label === label
                ? 'bg-[#F651AE] text-black font-medium'
                : 'bg-[#30283A] text-white hover:bg-[#8247E5]/20'
              } hover:scale-105 cursor-pointer`}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
};

export default HistogramChart;
