import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';
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

// Beta regression helper functions
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
      <div className="flex items-center justify-center h-96">
        <p className="text-white font-['Menlo']">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500 font-['Menlo']">{error}</p>
      </div>
    );
  }

  const sortedData = [...data].sort((a, b) => {
    const aNum = parseFloat(a.markout_time);
    const bNum = parseFloat(b.markout_time);
    return aNum - bNum;
  });

  // Calculate beta regression
  const xValues = sortedData.map(d => parseFloat(d.markout_time));
  const yValues = sortedData.map(d => d.ratio);
  const [alpha, beta] = betaRegression(xValues, yValues);

  const xRange = [...Array(100)].map((_, i) => {
    const min = Math.min(...xValues);
    const max = Math.max(...xValues);
    return min + (i * (max - min) / 99);
  });

  const yPred = xRange.map(x => invLogit(alpha + beta * x) * 100);

  const mainTrace: Partial<Data> = {
    x: sortedData.map(d => parseFloat(d.markout_time)),
    y: sortedData.map(d => d.ratio),
    type: 'scatter',
    mode: 'lines+markers',
    name: 'Observed Ratio',
    line: {
      color: plotColors.primary,
      width: 3,
    },
    marker: {
      color: plotColors.primary,
      size: 8,
    },
    hovertemplate: 
      '<b>Markout: %{x}s</b><br>' +
      'Capture Efficiency: %{y:.1f}%<br>' +
      'Realized: $%{customdata[0]:,.2f}<br>' +
      'Theoretical: $%{customdata[1]:,.2f}' +
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
      width: 2,
      dash: 'dot',
    },
    hoverinfo: 'skip',
    showlegend: false
  };

  const meanX = xValues.reduce((a, b) => a + b, 0) / xValues.length;
  const meanP = invLogit(alpha + beta * meanX);
  const marginalEffect = beta * meanP * (1 - meanP) * 100;

  const maxY = Math.max(...yValues, ...yPred);
  const yPadding = (maxY * 0.15); // 15% padding above the highest point

  const title = 'Ratio between Total Realized/Maximal LVR by Markout Time';
  const baseLayout = createBaseLayout(title);

  return (
    <div className="w-full">
      <Plot
        data={[mainTrace, betaRegressionTrace]}
        layout={{
          ...baseLayout,
          xaxis: {
            ...baseLayout.xaxis,
            title: {
              text: 'Markout Time (seconds)',
              font: { color: plotColors.accent, size: fontConfig.sizes.axisTitle, family: fontConfig.family },
              standoff: 20
            },
            tickfont: { color: '#ffffff', family: fontConfig.family },
            showgrid: false,
            gridcolor: '#212121',
            zeroline: true,
            zerolinecolor: plotColors.secondary,
          },
          yaxis: {
            ...baseLayout.yaxis,
            title: {
              text: 'Observed/Simulated Ratio',
              font: { color: plotColors.accent, size: fontConfig.sizes.axisTitle, family: fontConfig.family },
              standoff: 40
            },
            tickfont: { color: '#ffffff', family: fontConfig.family },
            tickformat: '.1f',
            ticksuffix: '%',
            showgrid: true,
            gridcolor: '#212121',
            range: [0, maxY + yPadding], // Extend range to accommodate annotation
          },
          height: 600,
          margin: { l: 120, r: 50, b: 80, t: 100, pad: 4 },
          annotations: [{
            ...createAnnotationConfig({
              x: 0,
              y: maxY + (yPadding / 2), // Position annotation in the middle of the padding
              xref: 'x',
              yref: 'y',
              text: `Avg. Marginal Effect: ${marginalEffect.toFixed(2)}pp/s`,
              showarrow: false,
              font: { 
                color: plotColors.accent,
                family: fontConfig.family,
                size: fontConfig.sizes.annotation
              },
              bgcolor: 'rgba(0,0,0,0.7)',
              borderpad: 4,
            })
          }],
          showlegend: false,
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: plotColors.accent,
            font: { 
              family: fontConfig.family,
              color: '#ffffff',
              size: fontConfig.sizes.hover
            }
          },
          hovermode: 'closest'
        }}
        config={{
          ...commonConfig,
          scrollZoom: false,
          modeBarButtonsToRemove: ['zoomIn2d', 'zoomOut2d', 'autoScale2d', 'resetScale2d']
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default RealizedRatioChart;