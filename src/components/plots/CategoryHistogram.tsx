import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';

interface HistogramBucket {
  range_start: number;
  range_end: number | null;
  count: number;
  label: string;
}

interface BucketWithPercentage extends HistogramBucket {
  percentage: number;
}

interface CategoryData {
  name: string;
  buckets: BucketWithPercentage[];
  total_observations: number;
}

interface CategoryHistogramProps {
  selectedMarkout: string;
}

// Updated color scheme from CategoryNonZero
export const CATEGORY_CONFIG = [
  { name: 'Stable Pairs',  label: 'Stable Pairs',  color: '#F651AE' }, // Pink
  { name: 'WBTC-WETH',     label: 'WBTC-WETH',     color: '#8247E5' }, // Purple
  { name: 'USDC-WETH',     label: 'USDC-WETH',     color: '#BA8EF7' }, // Light Purple
  { name: 'USDT-WETH',     label: 'USDT-WETH',     color: '#30283A' }, // Dark Purple
  { name: 'DAI-WETH',      label: 'DAI-WETH',      color: '#FF84C9' }, // Light Pink
  { name: 'USDC-WBTC',     label: 'USDC-WBTC',     color: '#644AA0' }, // Medium Purple
  { name: 'Altcoin-WETH',  label: 'Altcoin-WETH',  color: '#9B6FE8' }, // Lavender
] as const;

const bucketOrder = [
  '$0.01-$10',
  '$10-$100',
  '$100-$500',
  '>$500',
];

// Helper function to compute exact percentages that sum to 100%
function computeExactPercentages(
  buckets: HistogramBucket[],
  total: number
): BucketWithPercentage[] {
  if (total === 0) {
    return buckets.map(bucket => ({ ...bucket, percentage: 0 }));
  }
  const rawPercentages = buckets.map(bucket => (bucket.count / total) * 100);
  const rounded = rawPercentages.map(p => Math.round(p * 100) / 100);
  const sumRounded = rounded.reduce((acc, p) => acc + p, 0);
  const delta = Math.round((100 - sumRounded) * 100) / 100;
  const maxIndex = buckets.reduce(
    (maxIdx, bucket, idx) =>
      bucket.count > buckets[maxIdx].count ? idx : maxIdx,
    0
  );
  rounded[maxIndex] += delta;

  return buckets.map((bucket, idx) => ({
    ...bucket,
    percentage: rounded[idx],
  }));
}

const CategoryHistogram: React.FC<CategoryHistogramProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Which bin is selected?
  const [selectedLabel, setSelectedLabel] = useState<string | null>(null);

  // Which categories should be visible?
  const [visibleTraces, setVisibleTraces] = useState<Set<string>>(
    new Set(CATEGORY_CONFIG.map(c => c.label))
  );

  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const getResponsiveLayout = useCallback(() => {
    const isMobile = windowWidth < 768;
    const isTablet = windowWidth >= 768 && windowWidth <= 1024;

    return {
      height: isMobile ? 500 : isTablet ? 550 : 600,
      margin: {
        l: isMobile ? 100 : isTablet ? 130 : 150,
        r: isMobile ? 20 : isTablet ? 30 : 40,
        b: isMobile ? 160 : isTablet ? 180 : 200,
        t: isMobile ? 80 : isTablet ? 90 : 100,
      },
      fontSize: {
        title: isMobile ? 14 : isTablet ? 16 : 18,
        axis: isMobile ? 12 : isTablet ? 14 : 16,
        tick: isMobile ? 10 : isTablet ? 12 : 14,
        annotation: isMobile ? 12 : isTablet ? 14 : 16,
        legend: isMobile ? 12 : isTablet ? 14 : 16,
      },
      legendPosition: {
        x: isMobile ? 0.5 : 1,
        y: isMobile ? -0.6 : 1.1,
        xanchor: isMobile ? 'center' : 'right',
        yanchor: 'top',
        orientation: isMobile ? 'h' : 'v',
      } as const,
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/clusters/histogram?${params.toString()}`
        );

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonData = await response.json();
        const processedCategories = jsonData.clusters.map((cluster: CategoryData) => {
          // Consolidate any buckets with range_start >= 500 into a single bucket labeled '>$500'
          const consolidatedBuckets = cluster.buckets.reduce(
            (acc: HistogramBucket[], bucket: HistogramBucket) => {
              if (bucket.range_start < 500) {
                acc.push(bucket);
              } else {
                let consolidatedBucket = acc.find(b => b.label === '>$500');
                if (!consolidatedBucket) {
                  consolidatedBucket = {
                    range_start: 500,
                    range_end: null,
                    count: 0,
                    label: '>$500',
                  };
                  acc.push(consolidatedBucket);
                }
                consolidatedBucket.count += bucket.count;
              }
              return acc;
            },
            []
          );

          // Compute exact percentages for this category's buckets
          const bucketsWithPercentages = computeExactPercentages(
            consolidatedBuckets,
            cluster.total_observations
          );
          return {
            ...cluster,
            buckets: bucketsWithPercentages,
          };
        });

        // Sort to match CATEGORY_CONFIG order
        const sortedCategories = CATEGORY_CONFIG.map(config =>
          processedCategories.find((cat: CategoryData) => cat.name === config.name)
        ).filter((cat): cat is CategoryData => !!cat);

        setData(sortedCategories);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  const layoutOverrides = {
    legend: {
      itemclick: false as const,
      itemdoubleclick: false as const,
    },
  };

  const toggleCategory = (label: string) => {
    setVisibleTraces(prev => {
      const newSet = new Set(prev);
      if (newSet.has(label)) {
        newSet.delete(label);
        if (selectedLabel) {
          const stillVisibleData = data.some(cluster => {
            const c = CATEGORY_CONFIG.find(cfg => cfg.name === cluster.name);
            if (!c || !newSet.has(c.label)) return false;
            return cluster.buckets.some(b => b.label === selectedLabel && b.count > 0);
          });
          if (!stillVisibleData) {
            setSelectedLabel(null);
          }
        }
      } else {
        newSet.add(label);
      }
      return newSet;
    });
  };

  const handleBinClick = useCallback(
    (label: string) => {
      setSelectedLabel(prev => {
        if (prev === label) {
          return null;
        }
        const hasVisibleData = data.some(cluster => {
          const catConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
          if (!catConfig || !visibleTraces.has(catConfig.label)) return false;
          return cluster.buckets.some(b => b.label === label && b.count > 0);
        });
        return hasVisibleData ? label : null;
      });
    },
    [data, visibleTraces]
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[500px]">
        <p className="text-white text-base md:text-lg font-['Geist']">Loading (may take up to 30 seconds)...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[500px]">
        <p className="text-red-500 text-sm md:text-base font-['Geist']">
          {error || 'No data available'}
        </p>
      </div>
    );
  }

  const isMobile = windowWidth < 768;
  const titleSuffix =
    selectedMarkout === 'brontes'
      ? '(Brontes)'
      : `(Markout ${selectedMarkout}s)`;

  const title = isMobile
    ? `Single Block LVR Histogram\nGrouped by Category\n${titleSuffix}`
    : `Single Block LVR Histogram Grouped by Category ${titleSuffix}`;

  // Build the final Plotly traces
  const traces = data.reduce((acc, cluster) => {
    const categoryConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
    if (!categoryConfig) return acc;
    if (!visibleTraces.has(categoryConfig.label)) {
      return acc;
    }
    const orderedBuckets = [...cluster.buckets].sort(
      (a, b) => bucketOrder.indexOf(a.label) - bucketOrder.indexOf(b.label)
    );
    acc.push({
      name: categoryConfig.label,
      x: orderedBuckets.map(b => b.label),
      y: orderedBuckets.map(b => b.count),
      type: 'bar',
      marker: { color: categoryConfig.color },
      hoverinfo: 'none',
    });
    return acc;
  }, [] as Partial<Plotly.PlotData>[]);

  const responsiveLayout = getResponsiveLayout();
  const annotations = selectedLabel
    ? [
        {
          x: selectedLabel,
          y: Math.max(
            ...traces.map(trace => {
              const bucketIndex = bucketOrder.indexOf(selectedLabel);
              return (trace.y?.[bucketIndex] as number) || 0;
            })
          ),
          text: data
            .map(cluster => {
              const catConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
              if (!catConfig || !visibleTraces.has(catConfig.label)) return null;
              const bucketData = cluster.buckets.find(b => b.label === selectedLabel);
              if (!bucketData || bucketData.count === 0) return null;
              return {
                color: catConfig.color,
                name: catConfig.label,
                count: bucketData.count,
                pct: bucketData.percentage,
              };
            })
            .filter(Boolean)
            .map(
              item =>
                `<span style="color:${item!.color}">■</span> <b>${item!.name}</b>: ` +
                `${item!.count.toLocaleString()} (${item!.pct.toFixed(2)}%)`
            )
            .join('<br>'),
          showarrow: true,
          arrowhead: 2,
          arrowsize: 1,
          arrowwidth: 2,
          arrowcolor: '#FFFFFF',
          bgcolor: '#30283A',
          bordercolor: '#F651AE',
          font: {
            color: '#ffffff',
            size: responsiveLayout.fontSize.annotation,
            family: 'Geist',
          },
          borderwidth: 2,
          borderpad: 4,
          ay: -40,
          ax: 0,
          align: 'left' as const,
        },
      ]
    : [];

  const layout: Partial<Layout> = {
    paper_bgcolor: '#030304',
    plot_bgcolor: '#030304',
    barmode: 'group',
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    xaxis: {
      title: {
        text: 'LVR Range ($)',
        font: {
          color: '#F651AE',
          size: responsiveLayout.fontSize.axis,
          family: 'Geist',
        },
        standoff: isMobile ? 25 : 30,
      },
      tickfont: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.tick,
        family: 'Geist',
      },
      tickangle: isMobile ? -90 : -45,
      categoryorder: 'array' as const,
      categoryarray: bucketOrder,
      fixedrange: true as const,
      showgrid: true as const,
      gridcolor: '#30283A',
    },
    yaxis: {
      title: {
        text: 'Number of Blocks',
        font: {
          color: '#F651AE',
          size: responsiveLayout.fontSize.axis,
          family: 'Geist',
        },
        standoff: isMobile ? 40 : 50,
      },
      tickfont: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.tick,
        family: 'Geist',
      },
      fixedrange: true as const,
      showgrid: true as const,
      gridcolor: '#30283A',
    },
    annotations,
    title: {
      text: `<b>${title}</b>`,
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: 'Geist',
      },
    },
    ...layoutOverrides,
  };

  return (
    <div className="w-full">
      <div className="mb-6 text-center">
        <p className="text-white/80 text-sm md:text-base font-['Geist'] bg-[#30283A]/50 inline-block px-4 py-2 rounded-lg">
          Click on the bin buttons to view exact counts and percentages. Toggle
          the category buttons to add/remove categories from view
        </p>
      </div>

      <Plot
        data={traces}
        layout={layout}
        config={{
          responsive: true,
          scrollZoom: false,
          displayModeBar: false,
          toImageButtonOptions: {
            format: 'png',
            filename: 'category_histogram',
            height: responsiveLayout.height,
            width: windowWidth,
            scale: 2,
          },
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
      />

      {/* Bin Selection Buttons */}
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {bucketOrder.map(label => {
          const isSelected = selectedLabel === label;
          return (
            <button
              key={label}
              onClick={() => handleBinClick(label)}
              className={`
                px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base hover:scale-105 cursor-pointer
                ${
                  isSelected
                    ? 'bg-[#00FFC8] text-black font-medium' // New highlight color
                    : 'bg-[#30283A] text-white hover:bg-[#8247E5]/20'
                }
              `}
            >
              {label}
            </button>
          );
        })}
      </div>

      {/* External category toggles */}
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {CATEGORY_CONFIG.map(cfg => {
          const isVisible = visibleTraces.has(cfg.label);
          return (
            <button
              key={cfg.label}
              onClick={() => toggleCategory(cfg.label)}
              style={{
                backgroundColor: isVisible ? cfg.color : '#30283A',
                color: isVisible ? '#FFFFFF' : '#000000',
              }}
              className={`
                px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base hover:scale-105 cursor-pointer
                ${isVisible ? 'font-medium' : 'hover:bg-[#8247E5]/20'}
              `}
            >
              {cfg.label}
            </button>
          );
        })}
      </div>
    </div>
  );
};

export default CategoryHistogram;
