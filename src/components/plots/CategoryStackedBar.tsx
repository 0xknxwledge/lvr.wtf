import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { plotColors, createBaseLayout, commonConfig } from '../plotUtils';

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

// Consistent category configuration with defined order and colors
const CATEGORY_CONFIG = [
  { name: "Stable Pairs",   color: '#E2DFC9' },  // Light cream
  { name: "WBTC-WETH",      color: '#738C3A' },  // Medium olive
  { name: "USDC-WETH",      color: '#A4C27B' },  // Sage green
  { name: "USDT-WETH",      color: '#2D3A15' },  // Dark forest
  { name: "DAI-WETH",       color: '#BAC7A7' },  // Light sage
  { name: "USDC-WBTC",      color: '#4A5D23' },  // Deep forest
  { name: "Altcoin-WETH",   color: '#8B9556' }   // Muted olive
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

  // Create traces in the order defined by CATEGORY_CONFIG
  const traces = CATEGORY_CONFIG.map((category, index) => {
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

  // Split title into two lines for mobile
  const titleSuffix = selectedMarkout === 'brontes' 
    ? '(Observed)' 
    : `(Markout ${selectedMarkout}s)`;

  let title;
  if (isMobile) {
    title = `Monthly Total LVR<br>by Category<br>${titleSuffix}`;
  } else if (isTablet) {
    title = `Monthly Total LVR by Category<br>${titleSuffix}`;
  } else {
    title = `Monthly Total LVR by Category ${titleSuffix}`;
  }

  const baseLayout = createBaseLayout(title);

  return (
    <Plot
      data={traces}
      layout={{
        ...baseLayout,
        barmode: 'stack',
        xaxis: {
          ...baseLayout.xaxis,
          title: {
            text: 'Date Range (UTC)',
            font: { color: '#FFFFFF', size: isMobile ? 12 : 14 },
            standoff: isMobile ? 15 : 20
          },
          tickfont: { color: '#FFFFFF', size: isMobile ? 8 : 10 },
          tickangle: isMobile ? -90 : -45,
          fixedrange: true,
        },
        yaxis: {
          ...baseLayout.yaxis,
          title: {
            text: 'Total LVR (USD)',
            font: { color: '#FFFFFF', size: isMobile ? 12 : 14 },
            standoff: isMobile ? 40 : 100
          },
          tickfont: { color: '#FFFFFF', size: isMobile ? 8 : 12 },
          tickformat: '$,.0f',
          fixedrange: true,
          showgrid: true,
          gridcolor: '#212121',
        },
        showlegend: true,
        legend: {
          font: { color: '#FFFFFF', size: isMobile ? 10 : 12 },
          bgcolor: '#000000',
          bordercolor: '#212121',
          x: isMobile ? 0.5 : 1,
          y: isMobile ? -0.6 : 1.1, // Move legend further down on mobile
          xanchor: isMobile ? 'center' : 'right',
          yanchor: 'top',
          orientation: isMobile ? 'h' : 'v'
        },
        height: isMobile ? 650 : 500,
        margin: { 
          l: isMobile ? 120 : 150,
          r: isMobile ? 20 : 50, 
          b: isMobile ? 220 : 160, // Increased bottom margin for mobile
          t: isMobile ? 80 : 80 // Increased top margin for title
        },
        hovermode: 'x unified',
        hoverlabel: {
          bgcolor: '#424242',
          bordercolor: '#b4d838',
          font: { color: '#FFFFFF', size: isMobile ? 10 : 12 }
        }
      }}
      config={{
        ...commonConfig,
        responsive: true
      }}
      style={{ width: '100%', height: '100%', minHeight: '500px' }}
    />
  );
};

export default CategoryStackedBar;