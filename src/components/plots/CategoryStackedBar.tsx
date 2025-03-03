import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data, Layout } from 'plotly.js';

interface MonthlyData {
  time_range: string;
  cluster_totals: { [key: string]: number };
  total_lvr_cents: number;
}

interface CategoryStackedBarResponse {
  monthly_data: MonthlyData[];
  clusters: string[];
}

interface CategoryStackedBarProps {
  selectedMarkout: string;
}

// Updated colors to match the new theme
const CATEGORY_CONFIG = [
  { name: "Stable Pairs",   color: '#F651AE' },   // Pink
  { name: "WBTC-WETH",      color: '#8247E5' },   // Purple
  { name: "USDC-WETH",      color: '#BA8EF7' },   // Light Purple
  { name: "USDT-WETH",      color: '#30283A' },   // Dark Purple
  { name: "DAI-WETH",       color: '#FF84C9' },   // Light Pink
  { name: "USDC-WBTC",      color: '#644AA0' },   // Medium Purple
  { name: "Altcoin-WETH",   color: '#9B6FE8' }    // Lavender
] as const;

const CategoryStackedBar: React.FC<CategoryStackedBarProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryStackedBarResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const isMobile = windowWidth < 768;
  const isTablet = windowWidth >= 768 && windowWidth <= 1024;

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/monthly?${params.toString()}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData);
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
        <p className="text-white">Loading (may take up to 30 seconds)...</p>
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

  // Create traces in the order defined by CATEGORY_CONFIG
  const traces = CATEGORY_CONFIG.map((category) => {
    const categoryData = {
      name: category.name,
      x: data.monthly_data.map(d => d.time_range),
      y: data.monthly_data.map(d => d.cluster_totals[category.name] / 100), // Convert cents to dollars
      type: 'bar' as const,
      marker: {
        color: category.color
      },
      hovertemplate: 
        '<b>%{fullData.name}</b><br>' +
        '$%{y:,.2f}' +
        '<extra></extra>'
    };
    return categoryData;
  });

  const titleSuffix = selectedMarkout === 'brontes' 
    ? '(Brontes)' 
    : `(Markout ${selectedMarkout}s)`;

  let title;
  if (isMobile) {
    title = `Monthly Total LVR<br>by Category<br>${titleSuffix}`;
  } else if (isTablet) {
    title = `Monthly Total LVR by Category<br>${titleSuffix}`;
  } else {
    title = `Monthly Total LVR by Category ${titleSuffix}`;
  }

  const layout: Partial<Layout> = {
    paper_bgcolor: '#030304',
    plot_bgcolor: '#030304',
    barmode: 'stack',
    xaxis: {
      title: {
        text: 'Date Range (UTC)',
        font: { color: '#F651AE', size: isMobile ? 12 : 14, family: 'Geist' },
        standoff: isMobile ? 15 : 20
      },
      tickfont: { color: '#FFFFFF', size: isMobile ? 8 : 10, family: 'Geist' },
      tickangle: isMobile ? -90 : -45,
      fixedrange: true,
      showgrid: true,
      gridcolor: '#30283A'
    },
    yaxis: {
      title: {
        text: 'Total LVR (USD)',
        font: { color: '#F651AE', size: isMobile ? 12 : 14, family: 'Geist' },
        standoff: isMobile ? 20 : 40
      },
      tickfont: { color: '#FFFFFF', size: isMobile ? 8 : 12, family: 'Geist' },
      tickformat: '$,.0f',
      fixedrange: true,
      showgrid: true,
      gridcolor: '#30283A'
    },
    showlegend: true,
    legend: {
      font: { color: '#FFFFFF', size: isMobile ? 10 : 12, family: 'Geist' },
      bgcolor: '#030304',
      bordercolor: '#30283A',
      x: isMobile ? 0.5 : 1,
      y: isMobile ? -0.6 : 1.1,
      xanchor: isMobile ? 'center' : 'right',
      yanchor: 'top',
      orientation: isMobile ? 'h' : 'v'
    },
    height: isMobile ? 650 : 500,
    margin: { 
      l: isMobile ? 150 : 180,
      r: isMobile ? 20 : 50, 
      b: isMobile ? 220 : 160,
      t: isMobile ? 80 : 80
    },
    hovermode: 'x unified',
    hoverlabel: {
      bgcolor: '#30283A',
      bordercolor: '#F651AE',
      font: { color: '#FFFFFF', size: isMobile ? 10 : 12, family: 'Geist' }
    },
    title: {
      text: `<b>${title}</b>`,
      font: {
        color: '#FFFFFF',
        size: isMobile ? 14 : 16,
        family: 'Geist'
      }
    }
  };

  return (
    <div className="w-full h-full">
      <div className="mb-6 text-center">
        <p className="text-white/80 text-sm md:text-base font-['Geist'] bg-[#30283A]/50 inline-block px-4 py-2 rounded-lg">
          Click on categories in the legend to remove/add them from view
        </p>
    </div>
      <Plot
        data={traces}
        layout={layout}
        config={{
          responsive: true,
          displayModeBar: false,
          scrollZoom: false
        }}
        style={{ width: '100%', height: '100%', minHeight: '500px' }}
      />
    </div>
  );
};

export default CategoryStackedBar;