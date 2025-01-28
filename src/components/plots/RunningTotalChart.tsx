import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Dash, Layout } from 'plotly.js';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface RunningTotal {
  block_number: number;
  markout: string;
  running_total_cents: number;
}

const RunningTotalChart: React.FC = () => {
  const [data, setData] = useState<RunningTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  // Color scheme for different markout times
  const markoutColors = {
    'brontes': plotColors.accent,  // Bright lime green for observed
    '-2.0': '#FF6B6B',     // Red
    '-1.5': '#4ECDC4',     // Turquoise
    '-1.0': '#45B7D1',     // Light blue
    '-0.5': '#96CEB4',     // Sage green
    '0.0': '#FFBE0B',      // Yellow
    '0.5': '#FF006E',      // Pink
    '1.0': '#8338EC',      // Purple
    '1.5': '#3A86FF',      // Blue
    '2.0': '#FB5607'       // Orange
  };

  // Handle window resize
  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Calculate responsive dimensions
  const getResponsiveLayout = useCallback(() => {
    const isMobile = windowWidth < 768;
    const isTablet = windowWidth >= 768 && windowWidth < 1024;

    return {
      height: isMobile ? 400 : 600,
      margin: {
        l: isMobile ? 40 : (isTablet ? 60 : 80),
        r: isMobile ? 70 : (isTablet ? 85 : 100),
        b: isMobile ? 60 : (isTablet ? 80 : 100),
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        pad: 4
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        legend: isMobile ? 8 : (isTablet ? 10 : 12)
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch('https://lvr-wtf-568975696472.us-central1.run.app/running_total?aggregate=true');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const rawData: RunningTotal[] = await response.json();
        setData(rawData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px] bg-black rounded-lg border border-[#212121]">
        <div className="text-white text-base md:text-lg font-['Menlo']">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px] bg-black rounded-lg border border-[#212121]">
        <div className="text-white bg-red-600 p-3 md:p-4 rounded text-sm md:text-base font-['Menlo']">{error}</div>
      </div>
    );
  }

  const groupedData = data.reduce((acc, item) => {
    if (!acc[item.markout]) {
      acc[item.markout] = {
        x: [],
        y: [],
        markout: item.markout
      };
    }
    acc[item.markout].x.push(item.block_number);
    acc[item.markout].y.push(item.running_total_cents / 100);
    return acc;
  }, {} as Record<string, { x: number[]; y: number[]; markout: string }>);

  const responsiveLayout = getResponsiveLayout();
  const isMobile = windowWidth <= 768;

  const plotData = Object.values(groupedData).map(series => {
    const isBrontes = series.markout === 'brontes';
    
    return {
      x: series.x,
      y: series.y,
      type: 'scatter' as const,
      mode: 'lines' as const,
      name: isBrontes ? 'Observed' : `${series.markout}s`,
      line: {
        color: markoutColors[series.markout as keyof typeof markoutColors],
        width: isBrontes ? (isMobile ? 2 : 3) : (isMobile ? 1 : 2),
        dash: isBrontes ? undefined : ('solid' as Dash)
      },
      opacity: isBrontes ? 1 : 0.8,
      hoverinfo: 'x+y+name' as const,
      hoverlabel: {
        bgcolor: '#424242',
        bordercolor: markoutColors[series.markout as keyof typeof markoutColors],
        font: { 
          color: '#ffffff', 
          family: fontConfig.family, 
          size: responsiveLayout.fontSize.tick 
        }
      }
    };
  });

  const title = 'Cumulative LVR over Time';
  const baseLayout = createBaseLayout(title);

  const layout: Partial<Layout> = {
    ...baseLayout,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    xaxis: {
      ...baseLayout.xaxis,
      title: {
        text: 'Block Number',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 15 : 20
      },
      tickformat: ',d',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      tickangle: isMobile ? 45 : 0
    },
    yaxis: {
      ...baseLayout.yaxis,
      title: {
        text: 'Total LVR (USD)',
        font: { 
          color: plotColors.accent,
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 30 : 40
      },
      tickformat: '$,.0f',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      side: 'right',
      showgrid: false,
      rangemode: 'tozero'
    },
    showlegend: true,
    legend: {
      x: isMobile ? 0 : 0,
      y: isMobile ? -0.2 : 1,
      orientation: isMobile ? 'h' : 'v' as const,
      xanchor: isMobile ? 'left' : 'left' as const,
      yanchor: isMobile ? 'top' : 'auto' as const,
      bgcolor: '#000000',
      bordercolor: '#212121',
      font: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.legend,
        family: fontConfig.family
      }
    },
    title: {
      text: title,
      font: {
        color: plotColors.accent,
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    },
    hoverlabel: {
      font: { 
        family: fontConfig.family,
        size: responsiveLayout.fontSize.tick
      }
    },
    hovermode: 'closest'
  };

  return (
    <div className="w-full">
      <Plot
        data={plotData}
        layout={layout}
        config={{
          ...commonConfig,
          displayModeBar: true,
          displaylogo: false,
          modeBarButtonsToAdd: ['zoomIn2d', 'zoomOut2d', 'autoScale2d'],
          modeBarButtonsToRemove: ['lasso2d', 'select2d'],
          toImageButtonOptions: {
            format: 'png',
            filename: 'running_total_lvr',
            height: responsiveLayout.height,
            width: windowWidth,
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default RunningTotalChart;