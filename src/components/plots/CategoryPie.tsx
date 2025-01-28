import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { plotColors, createBaseLayout, commonConfig } from '../plotUtils';

interface CategoryData {
  name: string;
  total_lvr_cents: number;
}

interface CategoryPieResponse {
  clusters: CategoryData[];
  total_lvr_cents: number;
}

interface CategoryPieChartProps {
  selectedMarkout: string;
}

const CATEGORY_CONFIG = [
  { name: "Stable Pairs",   color: '#E2DFC9' },  // Light cream
  { name: "WBTC-WETH",      color: '#738C3A' },  // Medium olive
  { name: "USDC-WETH",      color: '#A4C27B' },  // Sage green
  { name: "USDT-WETH",      color: '#2D3A15' },  // Dark forest
  { name: "DAI-WETH",       color: '#BAC7A7' },  // Light sage
  { name: "USDC-WBTC",      color: '#4A5D23' },  // Deep forest
  { name: "Altcoin-WETH",   color: '#8B9556' }   // Muted olive
] as const;

const CategoryPieChart: React.FC<CategoryPieChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryPieResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/pie?${params.toString()}`);
        
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

  // Calculate responsive values
  const isMobile = windowWidth < 768;
  const isTablet = windowWidth >= 768 && windowWidth <= 1024;
  const isSmallScreen = isMobile || isTablet;

  const responsiveLayout = {
    height: isMobile ? 400 : isTablet ? 450 : 500,
    margin: {
      t: isMobile ? 30 : 50,
      b: isMobile ? 60 : 50,
      l: isMobile ? 30 : 50,
      r: isMobile ? 30 : 50
    },
    textFont: {
      size: isMobile ? 10 : isTablet ? 12 : 14
    }
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

  // Sort and map data according to CATEGORY_CONFIG order
  const sortedData = CATEGORY_CONFIG.map(config => {
    const categoryData = data.clusters.find(cluster => cluster.name === config.name);
    if (!categoryData) return null;
    
    const percentage = ((categoryData.total_lvr_cents / data.total_lvr_cents) * 100);
    return {
      value: categoryData.total_lvr_cents / 100, // Convert to dollars
      label: `${config.name} (${percentage.toFixed(1)}%)`,
      color: config.color
    };
  }).filter((item): item is NonNullable<typeof item> => item !== null);

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  // Create title with conditional line break
  const title = isSmallScreen ? 
    `Total LVR<br>by Category ${titleSuffix}` :
    `Total LVR by Category ${titleSuffix}`;

  const baseLayout = createBaseLayout(title);

  return (
    <Plot
      data={[
        {
          values: sortedData.map(d => d.value),
          labels: sortedData.map(d => d.label),
          type: 'pie',
          textinfo: 'label',
          textposition: 'outside',
          automargin: true,
          marker: {
            colors: sortedData.map(d => d.color),
            line: {
              color: '#000000',
              width: 2
            }
          },
          textfont: {
            color: '#FFFFFF',
            size: responsiveLayout.textFont.size
          },
          hoverlabel: {
            bgcolor: '#424242',
            font: { color: '#ffffff' }
          },
          hovertemplate: '<b>%{label}</b><br>$%{value:,.2f}<extra></extra>'
        }
      ]}
      layout={{
        ...baseLayout,
        showlegend: false,
        height: responsiveLayout.height,
        margin: responsiveLayout.margin,
        annotations: [{
          text: '',
          showarrow: false,
          x: 0.5,
          y: 1.1,
          xref: 'paper',
          yref: 'paper',
          font: {
            color: '#FFFFFF',
            size: 16
          }
        }]
      }}
      config={commonConfig}
      style={{ width: '100%', height: '100%' }}
      useResizeHandler={true}
    />
  );
};

export default CategoryPieChart;