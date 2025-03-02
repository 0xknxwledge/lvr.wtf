import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import type { Data, Layout, Dash } from 'plotly.js';
import { plotColors, fontConfig, commonConfig } from '../plotUtils';

interface RunningTotal {
  block_number: number;
  markout: string;
  running_total_cents: number;
}

interface EventAnnotation {
  blockStart: number;
  blockEnd: number;
  text: string;
  description: string;
}

const EVENT_ANNOTATIONS: EventAnnotation[] = [
  {
    blockStart: 16689392,
    blockEnd: 16696592,
    text: "SEC charges Terraform Labs; DOJ indicts SBF",
    description: "Late February 2023"
  },
  {
    blockStart: 16790192,
    blockEnd: 16797392,
    text: "USDC Depeg",
    description: "March 9-10, 2023"
  },
  {
    blockStart: 18669392,
    blockEnd: 18669392,
    text: "Microstrategy buys $600M of BTC",
    description: "Late November 2023"
  }
];

const markoutColors: Record<string, string> = {
  'brontes': plotColors.accent,
  '-2.0': '#FF6B6B',
  '-1.5': '#4ECDC4',
  '-1.0': '#45B7D1',
  '-0.5': '#96CEB4',
  '0.0': '#FFBE0B',
  '0.5': '#FF006E',
  '1.0': '#8338EC',
  '1.5': '#3A86FF',
  '2.0': '#FB5607'
};

const AnnotatedRunningTotal: React.FC = () => {
  const [data, setData] = useState<RunningTotal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
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
      height: isMobile ? 500 : 700,
      margin: {
        l: isMobile ? 60 : (isTablet ? 80 : 100),
        r: isMobile ? 90 : (isTablet ? 105 : 120),
        b: isMobile ? 60 : (isTablet ? 80 : 100),
        t: isMobile ? 100 : (isTablet ? 120 : 140),
        pad: 4
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        legend: isMobile ? 8 : (isTablet ? 10 : 12),
        annotation: isMobile ? 10 : (isTablet ? 12 : 14)
      },
      standoff: {
        x: isMobile ? 20 : (isTablet ? 25 : 30),
        y: isMobile ? 5 : (isTablet ? 10 : 20)
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
      <div className="flex items-center justify-center h-[500px] md:h-[700px] bg-black rounded-lg border border-[#212121]">
        <div className="text-white text-base md:text-lg font-['Geist']">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[500px] md:h-[700px] bg-black rounded-lg border border-[#212121]">
        <div className="text-red-500 p-3 md:p-4 rounded text-sm md:text-base font-['Geist']">{error}</div>
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

  const plotData: Data[] = Object.values(groupedData).map(series => {
    const isBrontes = series.markout === 'brontes';
    
    return {
      x: series.x,
      y: series.y,
      type: 'scatter',
      mode: 'lines',
      name: isBrontes ? 'Brontes' : `${series.markout}s`,
      line: {
        color: markoutColors[series.markout],
        width: isBrontes ? (isMobile ? 2 : 3) : (isMobile ? 1 : 2),
        dash: isBrontes ? undefined : ('solid' as Dash)
      },
      opacity: isBrontes ? 1 : 0.8,
      hoverinfo: 'x+y+name' as const,
      hoverlabel: {
        bgcolor: '#424242',
        bordercolor: markoutColors[series.markout],
        font: { 
          color: '#ffffff', 
          family: fontConfig.family, 
          size: responsiveLayout.fontSize.tick 
        }
      }
    };
  });

  // Create event annotations - positioned below the curve
  const annotations = EVENT_ANNOTATIONS.map((event) => {
    const seriesData = Object.values(groupedData)[0];
    const xIndex = seriesData.x.findIndex(x => x >= event.blockStart);
    const yValue = seriesData.y[xIndex];
  
    return {
      x: event.blockStart,
      y: yValue,
      text: `<b>${event.text}</b><br>${event.description}`,
      showarrow: true,
      arrowhead: 2,
      arrowsize: 1,
      arrowwidth: 2,
      arrowcolor: '#FFFFFF',
      ax: 60,
      ay: 60,
      font: {
        size: responsiveLayout.fontSize.annotation,
        color: '#ffffff',
        family: fontConfig.family
      },
      bgcolor: '#424242',
      bordercolor: plotColors.accent,
      borderwidth: 2,
      borderpad: 4
    };
  });

  // ***** ONLY CHANGE HERE: Wrap the text in <b>...</b> for a bold title *****
  const layout: Partial<Layout> = {
    paper_bgcolor: '#000000',
    plot_bgcolor: '#000000',
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    autosize: true,
    showlegend: true,
    xaxis: {
      title: {
        text: 'Block Number',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        }
      },
      tickformat: ',d',
      tickfont: { 
        color: '#ffffff', 
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      tickangle: isMobile ? 45 : 0,
      showgrid: true,
      gridcolor: '#30283A',
      gridwidth: 1,
      zeroline: false,
      range: [15537393, 20000000]
    },
    yaxis: {
      title: {
        text: 'Total LVR (USD)',
        font: { 
          color: plotColors.accent,
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family,
          weight: 700
        },
        standoff: responsiveLayout.standoff.y
      },
      tickformat: '$,.0f',
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      side: 'right',
      showgrid: true,
      gridcolor: '#30283A',
      gridwidth: 1,
      zeroline: false,
    },
    legend: {
      x: isMobile ? 0 : 0,
      y: isMobile ? -0.2 : 1,
      orientation: isMobile ? 'h' : 'v',
      xanchor: isMobile ? 'left' : 'left',
      yanchor: isMobile ? 'top' : 'auto',
      bgcolor: '#000000',
      bordercolor: '#212121',
      font: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.legend,
        family: fontConfig.family
      }
    },
    annotations: annotations,
    title: {
      text: '<b>Cumulative LVR over Time</b>',
      font: {
        color: '#FFFFFF',
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    },
    hovermode: 'closest'
  };

  return (
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
          filename: 'annotated_running_total_lvr',
          height: responsiveLayout.height,
          width: windowWidth,
          scale: 2
        }
      }}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default AnnotatedRunningTotal;
