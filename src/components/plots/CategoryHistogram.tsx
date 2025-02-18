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

const CategoryHistogram: React.FC<CategoryHistogramProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedLabel, setSelectedLabel] = useState<string | null>(null);
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
        <p className="text-red-500 text-sm md:text-base font-['Geist']">{error || 'No data available'}</p>
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
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  const isMobile = windowWidth < 768;
  let title;
  if (isMobile) {
    title = `Single Block LVR Histogram<br>Grouped by Category<br>${titleSuffix}`;
  } else {
    title = `Single Block LVR Histogram Grouped by Category ${titleSuffix}`;
  }

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

  const responsiveLayout = getResponsiveLayout();

  const annotations = selectedLabel ? [{
    x: selectedLabel,
    y: Math.max(...traces.map(trace => {
      const bucketIndex = bucketOrder.indexOf(selectedLabel);
      return (trace.y?.[bucketIndex] as number) || 0;
    })),
    text: data
      .map((cluster, index) => {
        const bucketData = cluster.buckets.find(b => b.label === selectedLabel);
        if (!bucketData || bucketData.count === 0) return null;
        
        const percentage = (bucketData.count / cluster.total_observations) * 100;
        const categoryConfig = CATEGORY_CONFIG[index];
        
        return {
          color: categoryConfig.color,
          name: categoryConfig.label,
          count: bucketData.count,
          percentage: percentage,
          order: index
        };
      })
      .filter(Boolean)
      .sort((a, b) => a!.order - b!.order)
      .map(item => 
        `<span style="color:${item!.color}">■</span> <b>${item!.name}</b>: ${item!.count.toLocaleString()} (${item!.percentage.toFixed(2)}%)`
      )
      .join('<br>'),
    showarrow: true,
    arrowhead: 2,
    arrowsize: 1,
    arrowwidth: 2,
    arrowcolor: '#F651AE',
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
  }] : [];

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
    annotations: annotations,
    legend: {
      font: { 
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.legend,
        family: 'Geist' 
      },
      bgcolor: '#030304',
      bordercolor: '#30283A',
      ...responsiveLayout.legendPosition
    },
    title: {
      text: title,
      font: {
        color: '#F651AE',
        size: responsiveLayout.fontSize.title,
        family: 'Geist'
      }
    }
  };

  return (
    <div className="w-full">
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
      />
      
      <div className="flex flex-wrap justify-center mt-8 gap-4">
        {bucketOrder.map((label) => (
          <button
            key={label}
            onClick={() => setSelectedLabel(selectedLabel === label ? null : label)}
            className={`px-4 py-2 rounded-lg transition-all duration-200 text-sm md:text-base ${
              selectedLabel === label
                ? 'bg-[#F651AE] text-black font-medium'
                : 'bg-[#30283A] text-white hover:bg-[#8247E5]/20'
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
};

export default CategoryHistogram;