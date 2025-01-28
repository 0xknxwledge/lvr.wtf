import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';
import dates from '../../dates';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface PercentileDataPoint {
  start_block: number;
  end_block: number;
  total_lvr_dollars: number;
  percentile_25_dollars: number;
  median_dollars: number;
  percentile_75_dollars: number;
}

interface PercentileBandResponse {
  pool_name: string;
  pool_address: string;
  markout_time: string;
  data_points: PercentileDataPoint[];
}

interface PercentileBandChartProps {
  poolAddress: string;
  markoutTime: string;
}

const PercentileBandChart: React.FC<PercentileBandChartProps> = ({
  poolAddress,
  markoutTime,
}) => {
  const [data, setData] = useState<PercentileBandResponse | null>(null);
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
        l: isMobile ? 60 : (isTablet ? 80 : 100),
        r: isMobile ? 50 : (isTablet ? 60 : 50),
        b: isMobile ? 140 : (isTablet ? 160 : 180),
        t: isMobile ? 80 : (isTablet ? 90 : 100),
        pad: 10,
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 10 : 12),
      },
      standoff: {
        x: isMobile ? 40 : (isTablet ? 50 : 60),
        y: isMobile ? 40 : (isTablet ? 50 : 60),
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);

        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime,
        });

        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/percentile_band?${params.toString()}`
        );
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonData: PercentileBandResponse = await response.json();
        const numDataPoints = jsonData.data_points.length;
        
        const startIndex = Math.max(0, dates.length - numDataPoints);
        const filteredDates = dates.slice(startIndex);
        
        (jsonData as any).filteredDates = filteredDates;

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
      <div className="flex items-center justify-center h-96">
        <p className="text-white font-['Menlo']">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500 font-['Menlo']">{error || 'No data available'}</p>
      </div>
    );
  }

  const { data_points } = data;
  const filteredDates: string[] = (data as any).filteredDates || [];

  const medianValues = data_points.map((d) => d.median_dollars);
  const percentile25Values = data_points.map((d) => d.percentile_25_dollars);
  const percentile75Values = data_points.map((d) => d.percentile_75_dollars);

  const titleSuffix =
    markoutTime === 'brontes'
      ? `${data.pool_name} (Observed LVR)`
      : `${data.pool_name} (Markout ${markoutTime}s)`;

  const title = shouldBreakTitle
    ? `Monthly LVR Percentile Bandplot<br>for ${titleSuffix}*`
    : `Monthly LVR Percentile Bandplot for ${titleSuffix}*`;

  const baseLayout = createBaseLayout(title);
  const responsiveLayout = getResponsiveLayout();

  const plotData: Array<Partial<Plotly.Data>> = [
    {
      x: [...filteredDates, ...filteredDates.slice().reverse()],
      y: [...percentile75Values, ...percentile25Values.slice().reverse()],
      fill: 'toself',
      fillcolor: 'rgba(180, 216, 56, 0.2)',
      line: { color: 'rgba(180, 216, 56, 0.5)' },
      name: '25th-75th Percentile',
      showlegend: false,
      type: 'scatter',
      mode: 'none',
      hoverinfo: 'skip',
    },
    {
      x: filteredDates,
      y: medianValues,
      type: 'scatter',
      mode: 'lines',
      name: 'Median',
      line: {
        color: plotColors.accent,
        width: 2,
      },
      showlegend: false,
      customdata: data_points.map((d) => [
        d.percentile_25_dollars,
        d.median_dollars,
        d.percentile_75_dollars,
        d.start_block,
        d.end_block,
        d.total_lvr_dollars,
      ]),
      hovertemplate:
        '<b>%{x}</b><br>' +
        'Blocks: %{customdata[3]} - %{customdata[4]}<br>' +
        'Total LVR: $%{customdata[5]:,.2f}<br>' +
        '75th Percentile: $%{customdata[2]:,.2f}<br>' +
        'Median: $%{customdata[1]:,.2f}<br>' +
        '25th Percentile: $%{customdata[0]:,.2f}' +
        '<extra></extra>',
    },
  ];

  const plotLayout = {
    ...baseLayout,
    title: {
      text: title,
      font: {
        color: plotColors.accent,
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family,
      },
    },
          xaxis: {
      ...baseLayout.xaxis,
      fixedrange: true,
      title: {
        text: 'Date Range (UTC)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis, 
          family: fontConfig.family 
        },
        standoff: responsiveLayout.standoff.x,
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: fontConfig.family 
      },
      tickangle: 45,
      automargin: true,
    },
          yaxis: {
      ...baseLayout.yaxis,
      fixedrange: true,
      title: {
        text: 'Daily Total LVR (USD)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis, 
          family: fontConfig.family 
        },
        standoff: responsiveLayout.standoff.y,
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: fontConfig.family 
      },
      automargin: true,
    },
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    hoverlabel: {
      bgcolor: '#424242',
      bordercolor: plotColors.accent,
      font: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: fontConfig.family 
      },
    },
  };

  return (
    <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
      <Plot
        data={plotData}
        layout={plotLayout}
        config={{
          ...commonConfig,
          scrollZoom: false,
          displayModeBar: false,
          toImageButtonOptions: {
            format: 'png',
            filename: `bandplot_${poolAddress}`,
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

export default PercentileBandChart;