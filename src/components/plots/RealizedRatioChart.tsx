import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import { Data, Layout } from 'plotly.js';
import { createBaseLayout, plotColors, fontConfig, commonConfig, createAnnotationConfig } from '../plotUtils';

interface MarkoutRatio {
  markout_time: string;
  ratio: number;
  realized_lvr_cents: number;
  theoretical_lvr_cents: number;
}

interface LVRRatioResponse {
  ratios: MarkoutRatio[];
}

function logit(p: number): number {
  p = Math.max(0.0001, Math.min(0.9999, p));
  return Math.log(p / (1 - p));
}

function invLogit(x: number): number {
  return 1 / (1 + Math.exp(-x));
}

function betaRegression(x: number[], y: number[]): [number, number] {
  const logitY = y.map(val => logit(val / 100));
  const meanX = x.reduce((a, b) => a + b, 0) / x.length;
  const meanLogitY = logitY.reduce((a, b) => a + b, 0) / logitY.length;
  
  const numerator = x.reduce((sum, xi, i) => sum + (xi - meanX) * (logitY[i] - meanLogitY), 0);
  const denominator = x.reduce((sum, xi) => sum + Math.pow(xi - meanX, 2), 0);
  const beta = numerator / denominator;
  const alpha = meanLogitY - beta * meanX;
  
  return [alpha, beta];
}

const RealizedRatioChart: React.FC = () => {
  const [data, setData] = useState<MarkoutRatio[]>([]);
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
      height: isMobile ? 400 : 600,
      margin: {
        l: isMobile ? 60 : (isTablet ? 90 : 120),
        r: isMobile ? 30 : (isTablet ? 40 : 50),
        b: isMobile ? 50 : (isTablet ? 65 : 80),
        t: isMobile ? 70 : (isTablet ? 85 : 100),
        pad: 4
      },
      fontSize: {
        title: isMobile ? 12 : (isTablet ? 14 : 16),
        axis: isMobile ? 10 : (isTablet ? 12 : 14),
        tick: isMobile ? 8 : (isTablet ? 9 : 10),
        annotation: isMobile ? 10 : (isTablet ? 11 : 12)
      },
      markerSize: isMobile ? 6 : 8,
      lineWidth: {
        primary: isMobile ? 2 : 3,
        secondary: isMobile ? 1.5 : 2
      }
    };
  }, [windowWidth]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const response = await fetch('https://lvr-wtf-568975696472.us-central1.run.app/ratios?start_block=15537392&end_block=20000000');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const rawData: LVRRatioResponse = await response.json();
        setData(rawData.ratios);
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
        <p className="text-white text-base md:text-lg font-['Menlo']">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[400px] md:h-[600px]">
        <p className="text-red-500 text-sm md:text-base font-['Menlo']">{error}</p>
      </div>
    );
  }

  const sortedData = [...data].sort((a, b) => {
    const aNum = parseFloat(a.markout_time);
    const bNum = parseFloat(b.markout_time);
    return aNum - bNum;
  });

  const xValues = sortedData.map(d => parseFloat(d.markout_time));
  const yValues = sortedData.map(d => d.ratio);
  const [alpha, beta] = betaRegression(xValues, yValues);

  const xRange = [...Array(100)].map((_, i) => {
    const min = Math.min(...xValues);
    const max = Math.max(...xValues);
    return min + (i * (max - min) / 99);
  });

  const yPred = xRange.map(x => invLogit(alpha + beta * x) * 100);
  const responsiveLayout = getResponsiveLayout();
  const isMobile = windowWidth <= 768;

  const mainTrace: Partial<Data> = {
    x: sortedData.map(d => parseFloat(d.markout_time)),
    y: sortedData.map(d => d.ratio),
    type: 'scatter',
    mode: 'lines+markers',
    name: 'Observed Ratio',
    line: {
      color: plotColors.primary,
      width: responsiveLayout.lineWidth.primary,
    },
    marker: {
      color: plotColors.primary,
      size: responsiveLayout.markerSize,
    },
    hovertemplate: 
      '<b>Markout: %{x}s</b><br>' +
      'Capture Efficiency: %{y:.1f}%<br>' +
      'Realized: $%{customdata[0]:,.2f}<br>' +
      'Theoretical Maximum: $%{customdata[1]:,.2f}' +
      '<extra></extra>',
    customdata: sortedData.map(d => [
      d.realized_lvr_cents / 100,
      d.theoretical_lvr_cents / 100
    ]),
    showlegend: false
  };

  const betaRegressionTrace: Partial<Data> = {
    x: xRange,
    y: yPred,
    type: 'scatter',
    mode: 'lines',
    name: 'Beta Regression',
    line: {
      color: plotColors.accent,
      width: responsiveLayout.lineWidth.secondary,
      dash: 'dot',
    },
    hoverinfo: 'skip',
    showlegend: false
  };

  const meanX = xValues.reduce((a, b) => a + b, 0) / xValues.length;
  const meanP = invLogit(alpha + beta * meanX);
  const marginalEffect = beta * meanP * (1 - meanP) * 100;

  const maxY = Math.max(...yValues, ...yPred);
  const yPadding = (maxY * (isMobile ? 0.1 : 0.15));

  const title = 'Ratio between Total<br>Realized/Maximal LVR by Markout Time*';
  const baseLayout = createBaseLayout(title);

  const layout: Partial<Layout> = {
    ...baseLayout,
    height: responsiveLayout.height,
    margin: responsiveLayout.margin,
    xaxis: {
      ...baseLayout.xaxis,
      title: {
        text: 'Markout Time (seconds)',
        font: { 
          color: plotColors.accent, 
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 15 : 20
      },
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      showgrid: false,
      gridcolor: '#212121',
      zeroline: true,
      zerolinecolor: plotColors.secondary,
    },
    yaxis: {
      ...baseLayout.yaxis,
      title: {
        text: 'Observed/Simulated Ratio',
        font: { 
          color: plotColors.accent,
          size: responsiveLayout.fontSize.axis,
          family: fontConfig.family 
        },
        standoff: isMobile ? 30 : 40
      },
      tickfont: { 
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick,
        family: fontConfig.family 
      },
      tickformat: '.1f',
      ticksuffix: '%',
      showgrid: true,
      gridcolor: '#212121',
      range: [0, maxY + yPadding],
    },
    title: {
      text: isMobile ? title : title.replace('<br>', ' '),
      font: {
        color: plotColors.accent,
        size: responsiveLayout.fontSize.title,
        family: fontConfig.family
      }
    },
    annotations: [{
      ...createAnnotationConfig({
        x: 0,
        y: maxY + (yPadding / 2),
        xref: 'x',
        yref: 'y',
        text: `Avg. Marginal Effect: ${marginalEffect.toFixed(2)}% for every +1 second`,
        showarrow: false,
        font: { 
          color: plotColors.accent,
          family: fontConfig.family,
          size: responsiveLayout.fontSize.annotation
        },
        bgcolor: 'rgba(0,0,0,0.7)',
        borderpad: isMobile ? 3 : 4,
      })
    }],
    showlegend: false,
    hoverlabel: {
      bgcolor: '#424242',
      bordercolor: plotColors.accent,
      font: { 
        family: fontConfig.family,
        color: '#ffffff',
        size: responsiveLayout.fontSize.tick
      }
    },
    hovermode: 'closest'
  };

  return (
    <div className="w-full">
      <Plot
        data={[mainTrace, betaRegressionTrace]}
        layout={layout}
        config={{
          ...commonConfig,
          scrollZoom: false,
          modeBarButtonsToRemove: ['zoomIn2d', 'zoomOut2d', 'autoScale2d', 'resetScale2d'],
          toImageButtonOptions: {
            format: 'png',
            filename: 'realized_ratio_chart',
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

export default RealizedRatioChart;