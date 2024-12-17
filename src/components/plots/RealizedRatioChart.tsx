import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';

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
  // Avoid exact 0 or 1 values
  p = Math.max(0.0001, Math.min(0.9999, p));
  return Math.log(p / (1 - p));
}

function invLogit(x: number): number {
  return 1 / (1 + Math.exp(-x));
}

// Perform beta regression
function betaRegression(x: number[], y: number[]): [number, number] {
  // Convert percentages to proportions and apply logit transform
  const logitY = y.map(val => logit(val / 100));
  
  // Calculate means
  const meanX = x.reduce((a, b) => a + b, 0) / x.length;
  const meanLogitY = logitY.reduce((a, b) => a + b, 0) / logitY.length;
  
  // Calculate beta (slope)
  const numerator = x.reduce((sum, xi, i) => sum + (xi - meanX) * (logitY[i] - meanLogitY), 0);
  const denominator = x.reduce((sum, xi) => sum + Math.pow(xi - meanX, 2), 0);
  const beta = numerator / denominator;
  
  // Calculate alpha (intercept)
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
        const response = await fetch('http://127.0.0.1:3000/ratios?start_block=15537392&end_block=20000000');
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
      <div className="flex items-center justify-center h-[600px] bg-[#000000] rounded-lg border border-[#212121]">
        <div className="text-white text-lg">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-[#000000] rounded-lg border border-[#212121]">
        <div className="text-white bg-red-600 p-4 rounded">{error}</div>
      </div>
    );
  }

  // Sort data by markout time for consistent display
  const sortedData = [...data].sort((a, b) => {
    const aNum = parseFloat(a.markout_time);
    const bNum = parseFloat(b.markout_time);
    return aNum - bNum;
  });

  const mainTrace: Partial<Data> = {
    x: sortedData.map(d => parseFloat(d.markout_time)),
    y: sortedData.map(d => d.ratio),
    type: 'scatter',
    mode: 'lines+markers',
    line: {
      color: '#b4d838',
      width: 3,
    },
    marker: {
      color: '#b4d838',
      size: 8,
    },
    hovertemplate: 
      'Markout: %{x}s<br>' +
      'Capture Efficiency: %{y:.1f}%<br>' +
      'Realized: $%{customdata[0]:,.2f}<br>' +
      'Theoretical: $%{customdata[1]:,.2f}<extra></extra>',
    customdata: sortedData.map(d => [
      d.realized_lvr_cents / 100,
      d.theoretical_lvr_cents / 100
    ]),
  };

  // Calculate beta regression
  const xValues = sortedData.map(d => parseFloat(d.markout_time));
  const yValues = sortedData.map(d => d.ratio);
  const [alpha, beta] = betaRegression(xValues, yValues);

  // Generate points for the beta regression curve
  const xRange = [...Array(100)].map((_, i) => {
    const min = Math.min(...xValues);
    const max = Math.max(...xValues);
    return min + (i * (max - min) / 99);
  });

  const yPred = xRange.map(x => invLogit(alpha + beta * x) * 100);

  const betaRegressionTrace: Partial<Data> = {
    x: xRange,
    y: yPred,
    type: 'scatter',
    mode: 'lines',
    name: 'Beta Regression',
    line: {
      color: 'rgba(180, 216, 56, 0.3)',
      width: 2,
      dash: 'dash',
    },
    hoverinfo: 'skip',
  };

  const plotData: Partial<Data>[] = [mainTrace, betaRegressionTrace];

  // Calculate average marginal effect at mean markout time
  const meanX = xValues.reduce((a, b) => a + b, 0) / xValues.length;
  const meanP = invLogit(alpha + beta * meanX);
  const marginalEffect = beta * meanP * (1 - meanP) * 100; // Convert to percentage points

  return (
    <div className="w-full">
      <Plot
        data={plotData}
        layout={{
          title: {
            text: 'LVR Capture Efficiency',
            font: { color: '#b4d838', size: 16 },
            y: 0.95
          },
          xaxis: {
            title: {
              text: 'Markout Time (seconds)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            zeroline: true,
            zerolinecolor: '#404040',
            gridcolor: '#212121',
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Observed/Theoretical Ratio',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: '.1f',
            ticksuffix: '%',
            tickfont: { color: '#ffffff' },
            range: [0, Math.max(...yValues) * 1.1],
            fixedrange: true,
            showgrid: true,
            gridcolor: '#212121',
          },
          autosize: true,
          height: 600,
          margin: { l: 80, r: 50, b: 80, t: 100, pad: 4 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          font: { color: '#ffffff' },
          hovermode: 'closest',
          hoverlabel: {
            bgcolor: '#424242',
            bordercolor: '#b4d838',
            font: { color: '#ffffff' }
          },
          showlegend: false,
          annotations: [{
            x: 0,
            y: invLogit(alpha) * 100,
            xref: 'x',
            yref: 'y',
            text: `Avg. Marginal Effect: ${marginalEffect.toFixed(2)}pp/s`,
            showarrow: false,
            font: { color: '#b4d838' },
            bgcolor: 'rgba(0,0,0,0.7)',
            borderpad: 4,
          }]
        }}
        config={{
          responsive: true,
          displayModeBar: false,
          scrollZoom: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default RealizedRatioChart;