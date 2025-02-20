import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';
import dates from '../../dates';

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
        <p className="text-white font-['Geist']">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500 font-['Geist']">{error || 'No data available'}</p>
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
      ? `${data.pool_name} (Brontes)`
      : `${data.pool_name} (Markout ${markoutTime}s)`;

  const title = shouldBreakTitle
    ? `Monthly LVR Percentile Bandplot<br>for ${titleSuffix}*`
    : `Monthly LVR Percentile Bandplot for ${titleSuffix}*`;

  const responsiveLayout = getResponsiveLayout();

  const plotData: Array<Partial<Plotly.Data>> = [
    {
      x: [...filteredDates, ...filteredDates.slice().reverse()],
      y: [...percentile75Values, ...percentile25Values.slice().reverse()],
      fill: 'toself',
      fillcolor: 'rgba(130, 71, 229, 0.2)', // #8247E5 with opacity
      line: { color: 'rgba(130, 71, 229, 0.5)' }, // #8247E5 with opacity
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
        color: '#F651AE', // Site's pink accent
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

  const layout = {
    paper_bgcolor: '#030304',
    plot_bgcolor: '#030304',
    title: {
      text: title,
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: 'Geist',
      },
    },
    xaxis: {
      title: {
        text: 'Date Range (UTC)',
        font: { 
          color: '#F651AE', 
          size: responsiveLayout.fontSize.axis, 
          family: 'Geist' 
        },
        standoff: responsiveLayout.standoff.x,
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: 'Geist' 
      },
      tickangle: 45,
      showgrid: true,
      gridcolor: '#30283A',
      automargin: true,
      fixedrange: true,
    },
    yaxis: {
      title: {
        text: 'Daily Total LVR (USD)',
        font: { 
          color: '#F651AE', 
          size: responsiveLayout.fontSize.axis, 
          family: 'Geist' 
        },
        standoff: responsiveLayout.standoff.y,
      },
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: 'Geist' 
      },
      showgrid: true,
      gridcolor: '#30283A',
      automargin: true,
      fixedrange: true,
    },
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    hoverlabel: {
      bgcolor: '#30283A',
      bordercolor: '#F651AE',
      font: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick, 
        family: 'Geist' 
      },
    },
  };

  return (
    <div className="w-full bg-[#030304] rounded-lg border border-[#8247E5]/20 p-6">
      <Plot
        data={plotData}
        layout={layout}
        config={{
          responsive: true,
          displayModeBar: false,
          scrollZoom: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
      <div className="mt-4 pl-4 text-center">
        <p className="text-[#8247E5] text-sm font-['Geist']">
          *We excluded days (i.e, 7200-block-long chunks) that had zero LVR. The percentile values here are computed directly from linear interpolation
        </p>
      </div>
    </div>
  );
};

export default PercentileBandChart;