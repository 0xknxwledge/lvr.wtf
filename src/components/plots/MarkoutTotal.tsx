import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Data, Layout } from 'plotly.js';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface MarkoutTotal {
  markout_time: string;
  total_dollars: number;
}

interface TotalLVRResponse {
  markout_totals: MarkoutTotal[];
}

const MarkoutTotals: React.FC = () => {
  const [data, setData] = useState<MarkoutTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  // Handle window resize
  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const isMobile = windowWidth <= 768;
  const isTablet = windowWidth >= 768 && windowWidth < 1024;

  // Calculate responsive dimensions
  const getResponsiveLayout = useCallback(() => {
    return {
      height: isMobile ? 400 : (isTablet ? 500 : 600),
      margin: {
        l: isMobile ? 80 : (isTablet ? 100 : 120),
        r: isMobile ? 40 : (isTablet ? 50 : 60),
        b: isMobile ? 80 : (isTablet ? 90 : 100),
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        pad: 10
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
        
        const response = await fetch('https://lvr-wtf-568975696472.us-central1.run.app/markout_totals');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: TotalLVRResponse = await response.json();
        
        // Filter out Brontes data and sort by markout time as numeric value
        const filteredData = jsonData.markout_totals
          .filter(item => item.markout_time !== 'brontes')
          .sort((a, b) => parseFloat(a.markout_time) - parseFloat(b.markout_time));
        
        setData(filteredData);
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
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-white text-base md:text-lg font-['Geist']">Loading (may take up to 30 seconds)...</p>
      </div>
    );
  }

  if (error || !data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-red-500 text-sm md:text-base font-['Geist']">{error || 'No data available'}</p>
      </div>
    );
  }

  // Format the x-axis labels for better readability
  const xLabels = data.map(item => {
    const numValue = parseFloat(item.markout_time);
    if (numValue === 0) return '0s';
    return numValue > 0 ? `+${numValue}s` : `${numValue}s`;
  });

  const responsiveLayout = getResponsiveLayout();
  const title = 'Cumulative Total LVR by Markout Time';

  const baseLayout = createBaseLayout(title);

  const layout: Partial<Layout> = {
    ...baseLayout,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    paper_bgcolor: '#030304',
    plot_bgcolor: '#030304',
    xaxis: {
      ...baseLayout.xaxis,
      title: {
        text: 'Markout Time (seconds)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: 20
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      showgrid: false,
      fixedrange: true,
      tickangle: 0,
      categoryorder: 'array',
      categoryarray: xLabels
    },
    yaxis: {
      ...baseLayout.yaxis,
      title: {
        text: 'Cumulative Total LVR since Merge (USD)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: 40
      },
      tickformat: '$,.0f',
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      showgrid: true,
      gridcolor: '#212121',
      zeroline: false,
      fixedrange: true
    },
    showlegend: false,
    title: {
      text: `<b>${title}</b>`,
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    },
    hoverlabel: {
      bgcolor: '#30283A',
      bordercolor: plotColors.accent,
      font: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      }
    },
    hovermode: 'closest'
  };

  // Find the markout time with the maximum LVR
  const maxLVRItem = data.reduce((max, current) => 
    current.total_dollars > max.total_dollars ? current : max, data[0]);


  return (
    <div className="w-full bg-[#030304] rounded-lg border border-[#8247E5]/20 p-6">
      <Plot
        data={[
          {
            x: data.map(item => item.markout_time),
            y: data.map(item => item.total_dollars),
            type: 'scatter',
            mode: 'lines+markers',
            line: {
              color: plotColors.accent,
              width: 2,
              shape: 'spline',
              smoothing: 0.3
            },
            marker: {
              color: plotColors.accent,
              size: 8,
              opacity: 0.8
            },
            hovertemplate: 'Markout: %{x}s<br>Total LVR: $%{y:,.2f}<extra></extra>'
          }
        ]}
        layout={layout}
        config={{
          ...commonConfig,
          responsive: true,
          displayModeBar: false
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
      />
    </div>
  );
};

export default MarkoutTotals;