import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface PoolMaxLVR {
  pool_name: string;
  pool_address: string;
  block_number: number;
  lvr_cents: number;
}

interface MaxLVRChartProps {
  selectedMarkout: string;
}

const MaxLVRChart: React.FC<MaxLVRChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<PoolMaxLVR[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const getResponsiveLayout = useCallback(() => {
    const isMobile = windowWidth <= 768;
    const isTablet = windowWidth >= 768 && windowWidth < 1024;

    return {
      height: isMobile ? 500 : (isTablet ? 550 : 600),
      margin: {
        l: isMobile ? 60 : (isTablet ? 80 : 100),
        r: isMobile ? 30 : (isTablet ? 40 : 50),
        b: isMobile ? 140 : (isTablet ? 150 : 160),
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        pad: 10
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 11 : 12),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        hover: isMobile ? 10 : (isTablet ? 11 : 12)
      },
      barWidth: isMobile ? 0.6 : 0.8
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });
        
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/max_lvr?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData.pools);
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
      <div className="flex items-center justify-center h-[500px] md:h-[600px]">
        <p className="text-white text-base md:text-lg font-['Geist']">Loading...</p>
      </div>
    );
  }

  if (error || !data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[500px] md:h-[600px]">
        <p className="text-red-500 text-sm md:text-base font-['Geist']">{error || 'No data available'}</p>
      </div>
    );
  }

  const sortedData = [...data].sort((a, b) => b.lvr_cents - a.lvr_cents);
  const maxY = Math.max(...sortedData.map(d => d.lvr_cents / 100));
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  const isMobile = windowWidth <= 768;
  const title = isMobile ?
    `Maximum Single-Block LVR<br>by Pool ${titleSuffix}` :
    `Maximum Single-Block LVR by Pool ${titleSuffix}`;

  const baseLayout = createBaseLayout(title);
  const responsiveLayout = getResponsiveLayout();

  const layout: Partial<Layout> = {
    ...baseLayout,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    autosize: true,
    showlegend: false,
    xaxis: {
      ...baseLayout.xaxis,
      title: {
        text: '',
        font: { 
          color: plotColors.accent,
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        }
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      tickangle: 45,
      fixedrange: true,
      automargin: true
    },
    yaxis: {
      ...baseLayout.yaxis,
      title: {
        text: 'Maximum Single-Block LVR (USD)',
        font: { 
          color: plotColors.accent,
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 30 : 40
      },
      tickformat: '$,.2f',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      fixedrange: true,
      showgrid: true,
      gridcolor: '#212121',
      zeroline: false,
      nticks: numTicks,
      range: [0, maxY * 1.1],
      automargin: true
    },
    hoverlabel: {
      bgcolor: '#424242',
      bordercolor: plotColors.accent,
      font: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.hover,
        family: fontConfig.family 
      },
      namelength: 0
    },
    hovermode: 'x unified',
    hoverdistance: 50,
    bargap: 0.2,
    title: {
      text: title,
      font: {
        color: plotColors.accent,
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    }
  };

  return (
    <div className="w-full h-full">
      <Plot
        data={[
          {
            x: sortedData.map(d => names[d.pool_address] || d.pool_name),
            y: sortedData.map(d => d.lvr_cents / 100),
            type: 'bar',
            marker: {
              color: plotColors.accent,
              opacity: 0.8,
            },
            hovertemplate:
              '<b>%{x}</b><br>' +
              'Maximum LVR: $%{y:,.2f}<br>' +
              'Block: %{customdata:,d}' +
              '<extra></extra>',
            customdata: sortedData.map(d => d.block_number),
            width: responsiveLayout.barWidth,
            showlegend: false,
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
      />
    </div>
  );
};

export default MaxLVRChart;