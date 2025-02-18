import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Data, Layout } from 'plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface QuartilePlotResponse {
  markout_time: string;
  pool_name: string;
  pool_address: string;
  percentile_25_cents: number;
  median_cents: number;
  percentile_75_cents: number;
}

interface QuartilePlotProps {
  poolAddress: string;
  markoutTime: string;
}

const QuartilePlot: React.FC<QuartilePlotProps> = ({ poolAddress, markoutTime }) => {
  const [data, setData] = useState<QuartilePlotResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [windowWidth, setWindowWidth] = useState(window.innerWidth);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const isMobile = windowWidth <= 768;
  const isTablet = windowWidth >= 768 && windowWidth < 1024;
  const shouldBreakTitle = isMobile || isTablet;

  const getResponsiveLayout = useCallback(() => {
    return {
      height: isMobile ? 400 : 600,
      margin: {
        l: isMobile ? 80 : (isTablet ? 100 : 120),
        r: isMobile ? 30 : (isTablet ? 40 : 50),
        b: isMobile ? 40 : (isTablet ? 45 : 50),
        t: isMobile ? 80 : (isTablet ? 90 : 100),
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        annotation: isMobile ? 10 : (isTablet ? 11 : 12)
      },
      whiskerWidth: isMobile ? 0.3 : 0.4,
      lineWidth: {
        primary: isMobile ? 1.5 : 2,
        secondary: isMobile ? 1 : 1.5
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime,
        });

        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/quartile_plot?${params.toString()}`
        );
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const jsonData: QuartilePlotResponse = await response.json();
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [poolAddress, markoutTime]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-white text-base md:text-lg font-['Menlo']">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-red-500 text-sm md:text-base font-['Menlo']">{error || 'No data available'}</p>
      </div>
    );
  }

  const maxY = data.percentile_75_cents / 100;
  const responsiveLayout = getResponsiveLayout();

  const plotData: Data[] = [
    // IQR box
    {
      type: 'scatter',
      y: [
        data.percentile_25_cents / 100,
        data.percentile_25_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_75_cents / 100,
        data.percentile_25_cents / 100,
      ],
      x: [-responsiveLayout.whiskerWidth, responsiveLayout.whiskerWidth, 
          responsiveLayout.whiskerWidth, -responsiveLayout.whiskerWidth, 
          -responsiveLayout.whiskerWidth],
      fill: 'toself',
      fillcolor: `${plotColors.accent}33`,
      line: { color: plotColors.accent, width: responsiveLayout.lineWidth.secondary },
      mode: 'lines',
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Bottom whisker
    {
      type: 'scatter',
      y: [0, data.percentile_25_cents / 100],
      x: [0, 0],
      mode: 'lines',
      line: { 
        color: plotColors.accent, 
        width: responsiveLayout.lineWidth.secondary, 
        dash: 'dot' 
      },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Top whisker
    {
      type: 'scatter',
      y: [data.percentile_75_cents / 100, maxY * 1.1],
      x: [0, 0],
      mode: 'lines',
      line: { 
        color: plotColors.accent, 
        width: responsiveLayout.lineWidth.secondary, 
        dash: 'dot' 
      },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Bottom whisker horizontal lines
    {
      type: 'scatter',
      y: [data.percentile_25_cents / 100, data.percentile_25_cents / 100],
      x: [-responsiveLayout.whiskerWidth, responsiveLayout.whiskerWidth],
      mode: 'lines',
      line: { color: plotColors.accent, width: responsiveLayout.lineWidth.primary },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Top whisker horizontal lines
    {
      type: 'scatter',
      y: [data.percentile_75_cents / 100, data.percentile_75_cents / 100],
      x: [-responsiveLayout.whiskerWidth, responsiveLayout.whiskerWidth],
      mode: 'lines',
      line: { color: plotColors.accent, width: responsiveLayout.lineWidth.primary },
      showlegend: false,
      hoverinfo: 'skip' as const,
    },
    // Median line (in red)
    {
      type: 'scatter',
      y: [data.median_cents / 100, data.median_cents / 100],
      x: [-responsiveLayout.whiskerWidth, responsiveLayout.whiskerWidth],
      mode: 'lines',
      line: { color: '#ff4444', width: responsiveLayout.lineWidth.primary },
      showlegend: false,
      hoverinfo: 'skip' as const,
    }
  ];

  const poolName = names[data.pool_address] || data.pool_name;
  const titleSuffix = markoutTime === 'brontes' ? '(Observed)' : `(Markout ${markoutTime}s)`;

  const title = shouldBreakTitle 
    ? `Single-Block LVR<br>Interquartile Plot for<br>${poolName}<br>${titleSuffix}*`
    : `Single-Block LVR Interquartile Plot for ${poolName} ${titleSuffix}*`;

  const baseLayout = createBaseLayout(title);

  const annotations = [
    {
      y: data.percentile_25_cents / 100,
      x: 0.5, // Changed from 1.0 to 0.5
      text: `25th Percentile<br>$${(data.percentile_25_cents / 100).toFixed(2)}`, // Added $
      showarrow: false,
      font: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.annotation, 
        family: fontConfig.family 
      },
      align: 'left' as const,
      xanchor: 'left' as const,
    },
    {
      y: data.median_cents / 100,
      x: -0.5, // Changed from -1.0 to -0.5
      text: `Median<br>$${(data.median_cents / 100).toFixed(2)}`, // Added $
      showarrow: false,
      font: { 
        color: '#ff4444', 
        size: responsiveLayout.fontSize.annotation, 
        family: fontConfig.family 
      },
      align: 'right' as const,
      xanchor: 'right' as const,
    },
    {
      y: data.percentile_75_cents / 100,
      x: 0.5, // Changed from 1.0 to 0.5
      text: `75th Percentile<br>$${(data.percentile_75_cents / 100).toFixed(2)}`, // Added $
      showarrow: false,
      font: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.annotation, 
        family: fontConfig.family 
      },
      align: 'left' as const,
      xanchor: 'left' as const,
    }
  ];

  return (
    <div className="w-full h-full bg-black rounded-lg border border-[#212121] p-6">
      <Plot
        data={plotData}
        layout={{
          ...baseLayout,
          showlegend: false,
          xaxis: {
            ...baseLayout.xaxis,
            showticklabels: false,
            zeroline: false,
            fixedrange: true,
            showgrid: false,
            range: [-1.2, 1.2],
          },
          yaxis: {
            ...baseLayout.yaxis,
            title: {
              text: 'Single-Block LVR (USD)',
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
            zeroline: false,
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
            range: [0, maxY * 1.1],
            automargin: true,
          },
          height: responsiveLayout.height,
          margin: responsiveLayout.margin,
          annotations: annotations,
          title: {
            text: title,
            font: {
              color: plotColors.accent,
              size: responsiveLayout.fontSize.title,
              family: fontConfig.family
            }
          },
        }}
        config={{
          ...commonConfig,
          responsive: true,
          toImageButtonOptions: {
            format: 'png',
            filename: `quartile_plot_${poolAddress}`,
            height: responsiveLayout.height,
            width: windowWidth,
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
      />
      <div className="mt-4 pl-4 text-center">
      <p className="text-[#8247E5]/80 text-sm font-['Menlo']">
          *The distribution here represents blocks with non-zero LVR. Due to the high volume of non-zero blocks, we estimate percentile values through the t-digest data structure and online algorithm rather than directly interpolating from the complete dataset
        </p>
      </div>
    </div>
  );
};

export default QuartilePlot;