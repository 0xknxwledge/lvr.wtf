import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';

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

// Updated to match the new color scheme from CategoryNonZero
export const CATEGORY_CONFIG = [
  { name: "Stable Pairs",   label: "Stable Pairs",   color: '#F651AE' },   // Pink
  { name: "WBTC-WETH",      label: "WBTC-WETH",      color: '#8247E5' },   // Purple
  { name: "USDC-WETH",      label: "USDC-WETH",      color: '#BA8EF7' },   // Light Purple
  { name: "USDT-WETH",      label: "USDT-WETH",      color: '#30283A' },   // Dark Purple
  { name: "DAI-WETH",       label: "DAI-WETH",       color: '#FF84C9' },   // Light Pink
  { name: "USDC-WBTC",      label: "USDC-WBTC",      color: '#644AA0' },   // Medium Purple
  { name: "Altcoin-WETH",   label: "Altcoin-WETH",   color: '#9B6FE8' }    // Lavender
] as const;

const bucketOrder = [
  '$0.01-$10',
  '$10-$100',
  '$100-$500',
  '>$500'
];

const CategoryHistogram: React.FC<CategoryHistogramProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Which bin is selected?
  const [selectedLabel, setSelectedLabel] = useState<string | null>(null);

  // Which categories should be visible?
  // If you want them all visible initially, seed the set with every label.
  const [visibleTraces, setVisibleTraces] = useState<Set<string>>(
    new Set(CATEGORY_CONFIG.map(c => c.label))
  );

  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  // Responsive sizing
  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const getResponsiveLayout = useCallback(() => {
    const isMobile = windowWidth < 768;
    const isTablet = windowWidth >= 768 && windowWidth <= 1024;

    return {
      height: isMobile ? 500 : (isTablet ? 550 : 600),
      margin: {
        l: isMobile ? 100 : (isTablet ? 130 : 150),
        r: isMobile ? 20 : (isTablet ? 30 : 40),
        b: isMobile ? 160 : (isTablet ? 180 : 200),
        t: isMobile ? 80 : (isTablet ? 90 : 100)
      },
      fontSize: {
        title: isMobile ? 14 : (isTablet ? 16 : 18),
        axis: isMobile ? 12 : (isTablet ? 14 : 16),
        tick: isMobile ? 10 : (isTablet ? 12 : 14),
        annotation: isMobile ? 12 : (isTablet ? 14 : 16),
        legend: isMobile ? 12 : (isTablet ? 14 : 16)
      },
      legendPosition: {
        x: isMobile ? 0.5 : 1,
        y: isMobile ? -0.6 : 1.1,
        xanchor: isMobile ? 'center' : 'right',
        yanchor: 'top',
        orientation: isMobile ? 'h' : 'v'
      } as const
    };
  }, [windowWidth]);

  // Fetch data
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
          // Consolidate any buckets >= $500 into a single bucket
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
                    label: '>$500'
                  };
                  acc.push(consolidatedBucket);
                }
                consolidatedBucket.count += bucket.count;
              }
              return acc;
            },
            []
          );
          
          return {
            ...cluster,
            buckets: consolidatedBuckets
          };
        });

        // Sort to match CATEGORY_CONFIG order
        const sortedCategories = CATEGORY_CONFIG
          .map(config => processedCategories.find((cat: CategoryData) => cat.name === config.name))
          .filter((cat): cat is CategoryData => !!cat);

        setData(sortedCategories);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  // If you no longer want toggling from the legend, remove onClick entirely or disable legend events.
  // For example:
  const layoutOverrides = {
    // This disables itemclick in the legend so it no longer toggles traces:
    legend: {
      itemclick: false,
      itemdoubleclick: false
    }
  };

  // Instead, you could remove the legend completely with:
  // const layoutOverrides = { showlegend: false };

  // If you still want a legend for *visible* categories but never want them toggled, 
  // do "itemclick: false" and generate only the visible traces.

  // Let’s create a simple function to toggle categories from some external UI (if desired).
  // For example, if you have checkboxes or other toggles, you might do:
  const toggleCategory = (label: string) => {
    setVisibleTraces(prev => {
      const newSet = new Set(prev);
      if (newSet.has(label)) {
        newSet.delete(label);
        // Also clear the bin selection if it no longer has visible data
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

  // Handle bin selection
  const handleBinClick = useCallback((label: string) => {
    setSelectedLabel(prev => {
      if (prev === label) {
        return null; // deselect if already selected
      }
      // Only select if there's at least one visible trace with data
      const hasVisibleData = data.some(cluster => {
        const catConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
        if (!catConfig || !visibleTraces.has(catConfig.label)) return false;
        return cluster.buckets.some(b => b.label === label && b.count > 0);
      });
      return hasVisibleData ? label : null;
    });
  }, [data, visibleTraces]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[500px]">
        <p className="text-white text-base md:text-lg font-['Geist']">Loading...</p>
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
  const titleSuffix = selectedMarkout === 'brontes'
    ? '(Brontes)'
    : `(Markout ${selectedMarkout}s)`;

  const title = isMobile
    ? `Single Block LVR Histogram<br>Grouped by Category<br>${titleSuffix}`
    : `Single Block LVR Histogram Grouped by Category ${titleSuffix}`;

  // Build the final Plotly traces, but *only* for categories in visibleTraces.
  const traces = data.reduce((acc, cluster) => {
    // Find matching config
    const categoryConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
    if (!categoryConfig) return acc;

    // If the category is not in visibleTraces, skip it entirely
    if (!visibleTraces.has(categoryConfig.label)) {
      return acc;
    }

    // Sort buckets by bucketOrder
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
      // No 'legendonly' or 'false' here — if it's visible, we pass it in.
      // showlegend defaults to true if name is set, so it appears in the legend.
    });
    return acc;
  }, [] as Partial<Plotly.PlotData>[]);

  // Build the annotation if a bin is selected
  const responsiveLayout = getResponsiveLayout();
  const annotations = selectedLabel
    ? [
        {
          x: selectedLabel,
          y: Math.max(
            ...traces.map(trace => {
              // Each trace is definitely visible now
              const bucketIndex = bucketOrder.indexOf(selectedLabel);
              return (trace.y?.[bucketIndex] as number) || 0;
            })
          ),
          text: data
            .map(cluster => {
              // Only consider visible categories
              const catConfig = CATEGORY_CONFIG.find(c => c.name === cluster.name);
              if (!catConfig || !visibleTraces.has(catConfig.label)) return null;
              // Find the relevant bin
              const bucketData = cluster.buckets.find(b => b.label === selectedLabel);
              if (!bucketData || bucketData.count === 0) return null;

              const pct = (bucketData.count / cluster.total_observations) * 100;
              return {
                color: catConfig.color,
                name: catConfig.label,
                count: bucketData.count,
                pct
              };
            })
            .filter(Boolean)
            .map(item =>
              `<span style="color:${item!.color}">■</span> <b>${item!.name}</b>: ${item!.count.toLocaleString()} (${item!.pct.toFixed(2)}%)`
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
            family: 'Geist'
          },
          borderwidth: 2,
          borderpad: 4,
          ay: -40,
          ax: 0,
          align: 'left' as const
        }
      ]
    : [];

  // Layout
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
          family: 'Geist'
        },
        standoff: isMobile ? 25 : 30
      },
      tickfont: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.tick,
        family: 'Geist'
      },
      tickangle: isMobile ? -90 : -45,
      categoryorder: 'array' as const,
      categoryarray: bucketOrder,
      fixedrange: true,
      showgrid: true,
      gridcolor: '#30283A'
    },
    yaxis: {
      title: {
        text: 'Number of Blocks',
        font: {
          color: '#F651AE',
          size: responsiveLayout.fontSize.axis,
          family: 'Geist'
        },
        standoff: isMobile ? 40 : 50
      },
      tickfont: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.tick,
        family: 'Geist'
      },
      fixedrange: true,
      showgrid: true,
      gridcolor: '#30283A'
    },
    annotations,
    title: {
      text: title,
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: 'Geist'
      }
    },
    // If you want to hide the legend entirely, uncomment:
    // showlegend: false
  };

  return (
    <div className="w-full">
      <div className="mb-6 text-center">
        <p className="text-white/80 text-sm md:text-base font-['Geist'] bg-[#30283A]/50 inline-block px-4 py-2 rounded-lg">
          Click on the bin buttons to view exact counts and percentages. Toggle the category buttons to add/remove categories from view
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
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
        // onClick can be removed or repurposed if you like
      />

      {/* Bin Selection Buttons */}
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {bucketOrder.map(label => (
          <button
            key={label}
            onClick={() => handleBinClick(label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base ${
              selectedLabel === label
                ? 'bg-[#F651AE] text-black font-medium'
                : 'bg-[#30283A] text-white hover:bg-[#8247E5]/20'
            } hover:scale-105 cursor-pointer`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Example: External category toggles (optional) */}
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {CATEGORY_CONFIG.map(cfg => (
          <button
            key={cfg.label}
            onClick={() => toggleCategory(cfg.label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base ${
              visibleTraces.has(cfg.label)
                ? 'bg-[#F651AE] text-black font-medium'
                : 'bg-[#30283A] text-white hover:bg-[#8247E5]/20'
            } hover:scale-105 cursor-pointer`}
          >
            {cfg.label}
          </button>
        ))}
      </div>
    </div>
  );
};

export default CategoryHistogram;
