import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Layout } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface RunningTotal {
  block_number: number;
  markout: string;
  pool_name: string | null;
  pool_address: string | null;
  running_total_cents: number;
}


interface SoleRunningTotalProps {
  poolAddress: string;
  markoutTime: string;
}

const SoleRunningTotal: React.FC<SoleRunningTotalProps> = ({ poolAddress, markoutTime }) => {
  const [data, setData] = useState<RunningTotal[]>([]);
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
  const shouldBreakTitle = isMobile || isTablet;

  // Calculate responsive dimensions
  const getResponsiveLayout = useCallback(() => {

    return {
      height: isMobile ? 400 : 600,
      margin: {
        l: isMobile ? 40 : (isTablet ? 45 : 50),
        r: isMobile ? 80 : (isTablet ? 100 : 120),
        b: isMobile ? 60 : (isTablet ? 80 : 100),
        t: isMobile ? 60 : (isTablet ? 70 : 80),
        pad: 10
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10)
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          pool: poolAddress,
          aggregate: 'false',
          markout_time: markoutTime
        });

        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/running_total?${params.toString()}`);
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

    if (poolAddress && markoutTime) {
      fetchData();
    }
  }, [poolAddress, markoutTime]);

  if (isLoading) {
    return (
      <div className="w-full bg-black rounded-lg md:rounded-2xl border border-[#212121] p-4 md:p-6">
        <div className="h-[400px] md:h-[600px] flex items-center justify-center">
          <div className="text-white text-base md:text-lg font-['Menlo']">Loading...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full bg-black rounded-lg md:rounded-2xl border border-[#212121] p-4 md:p-6">
        <div className="h-[400px] md:h-[600px] flex items-center justify-center">
          <div className="text-white bg-red-600 p-3 md:p-4 rounded text-sm md:text-base font-['Menlo']">{error}</div>
        </div>
      </div>
    );
  }

  if (!data || data.length === 0) {
    return (
      <div className="w-full bg-black rounded-lg md:rounded-2xl border border-[#212121] p-4 md:p-6">
        <div className="h-[400px] md:h-[600px] flex items-center justify-center">
          <div className="text-white text-base md:text-lg font-['Menlo']">No data available</div>
        </div>
      </div>
    );
  }

  const poolName = data[0].pool_name;
  const titleSuffix = markoutTime === 'brontes' ? '(Observed)' : `(Markout ${markoutTime}s)`;
  
  const maxY = Math.max(...data.map(point => point.running_total_cents / 100));
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);
  

  const title = shouldBreakTitle
    ? `Cumulative LVR over Time <br> for ${poolName} ${titleSuffix}`
    : `Cumulative LVR over Time for ${poolName} ${titleSuffix}`;
  const baseLayout = createBaseLayout(title);
  const responsiveLayout = getResponsiveLayout();

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
        standoff: windowWidth <= 768 ? 15 : 20
      },
      tickformat: ',d',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      showgrid: true,
      gridcolor: '#212121',
      automargin: true,
      tickangle: windowWidth <= 768 ? 45 : 0
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
        standoff: windowWidth <= 768 ? 30 : 40
      },
      tickformat: '$,.2f',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      showgrid: true,
      gridcolor: '#212121',
      nticks: numTicks,
      range: [0, maxY * 1.1],
      automargin: true,
      side: 'right',
      ticklabelposition: 'outside right'
    },
    showlegend: false,
    autosize: true,
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
    <div className="w-full bg-black rounded-lg md:rounded-2xl border border-[#212121] p-4 md:p-6">
      <Plot
        data={[
          {
            x: data.map(point => point.block_number),
            y: data.map(point => point.running_total_cents / 100),
            type: 'scatter',
            mode: 'lines',
            name: `${poolName} ${titleSuffix}`,
            line: {
              color: plotColors.accent,
              width: windowWidth <= 768 ? 1.5 : 2,
            },
            hoverinfo: 'x+y' as const,
            hoverlabel: {
              bgcolor: '#424242',
              bordercolor: plotColors.accent,
              font: { 
                color: '#ffffff', 
                size: responsiveLayout.fontSize.tick,
                family: fontConfig.family 
              }
            },
            showlegend: false
          }
        ]}
        layout={layout}
        config={{
          ...commonConfig,
          displayModeBar: true,
          displaylogo: false,
          modeBarButtonsToAdd: ['zoomIn2d', 'zoomOut2d', 'autoScale2d'],
          modeBarButtonsToRemove: ['lasso2d', 'select2d'],
          toImageButtonOptions: {
            format: 'png',
            filename: `running_total_lvr_${poolAddress}`,
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

export default SoleRunningTotal;