import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';
import names from '../../names';
import { plotColors, createBaseLayout, commonConfig, fontConfig } from '../plotUtils';

interface PoolTotal {
  pool_name: string;
  pool_address: string;
  total_lvr_cents: number;
}

interface PoolTotalsPieChartProps {
  selectedMarkout: string;
}

const PoolTotalsPieChart: React.FC<PoolTotalsPieChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<PoolTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedSegment, setSelectedSegment] = useState<string | null>(null);
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
      height: isMobile ? 400 : (isTablet ? 500 : 600),
      margin: {
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        b: isMobile ? 60 : (isTablet ? 70 : 80),
        l: isMobile ? 40 : (isTablet ? 60 : 80),
        r: isMobile ? 40 : (isTablet ? 60 : 80),
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 13 : 14),
        text: isMobile ? 10 : (isTablet ? 11 : 12),
        hover: isMobile ? 10 : (isTablet ? 11 : 12)
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });
        
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/pool_totals?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        if (Array.isArray(jsonData.totals)) {
          setData(jsonData.totals);
        } else {
          throw new Error('Received invalid data format');
        }
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
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <div className="text-white text-base md:text-lg animate-pulse">Loading...</div>
      </div>
    );
  }

  if (error || !data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <div className="text-red-500 bg-red-500/10 px-4 py-2 rounded-lg text-sm md:text-base">
          {error || 'No data available'}
        </div>
      </div>
    );
  }

  const sortedData = [...data].sort((a, b) => b.total_lvr_cents - a.total_lvr_cents);
  const total = sortedData.reduce((sum, item) => sum + item.total_lvr_cents, 0);
  const values = sortedData.map(item => item.total_lvr_cents / 100);

  const isMobile = windowWidth < 768;
  const responsiveLayout = getResponsiveLayout();

  const labels = sortedData.map(item => {
    const poolName = names[item.pool_address] || item.pool_name;
    const percentage = ((item.total_lvr_cents / total) * 100).toFixed(1);
    return `${poolName} (${percentage}%)`;
  });

  const customColors = plotColors.chartColors;
  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  const title = isMobile ? 
    `Total LVR by Pool<br>${titleSuffix}` : 
    `Total LVR by Pool ${titleSuffix}`;

  const baseLayout = createBaseLayout(title);

  const layout: Partial<Layout> = {
    ...baseLayout,
    showlegend: false,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    autosize: true,
    font: { 
      color: '#FFFFFF', 
      family: fontConfig.family,
      size: responsiveLayout.fontSize.text
    },
    title: {
      text: title,
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    }
  };

  return (
    <div className="w-full h-full bg-black rounded-lg">
      <Plot
        data={[
          {
            values,
            labels,
            type: 'pie',
            textinfo: 'label',
            textposition: 'outside',
            automargin: true,
            marker: {
              colors: customColors,
              line: {
                color: '#000000',
                width: isMobile ? 1 : 2
              }
            },
            textfont: {
              color: '#FFFFFF',
              size: responsiveLayout.fontSize.text,
              family: fontConfig.family
            },
            hoverlabel: {
              bgcolor: plotColors.secondary,
              bordercolor: plotColors.accent,
              font: { 
                color: '#FFFFFF',
                size: responsiveLayout.fontSize.hover,
                family: fontConfig.family
              }
            },
            hovertemplate: '<b>%{label}</b><br>$%{value:,.2f}<extra></extra>'
          }
        ]}
        layout={layout}
        config={{
          ...commonConfig,
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
        onClick={(event) => {
          if (event.points && event.points[0]) {
            const point = event.points[0];
            const pointIndex = point.pointIndex as number;
            const pointLabel = labels[pointIndex];
            setSelectedSegment(selectedSegment === pointLabel ? null : pointLabel);
          }
        }}
      />
    </div>
  );
};

export default PoolTotalsPieChart;