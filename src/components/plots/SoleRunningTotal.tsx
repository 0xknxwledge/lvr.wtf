import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../../names';
import { createBaseLayout, plotColors, fontConfig, commonConfig } from '../plotUtils';

interface RunningTotal {
  block_number: number;
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
        const rawData = await response.json();
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
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[600px] flex items-center justify-center">
          <div className="text-white text-lg font-['Menlo']">Loading...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[600px] flex items-center justify-center">
          <div className="text-white bg-red-600 p-4 rounded font-['Menlo']">{error}</div>
        </div>
      </div>
    );
  }

  if (!data || data.length === 0) {
    return (
      <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
        <div className="h-[600px] flex items-center justify-center">
          <div className="text-white text-lg font-['Menlo']">No data available</div>
        </div>
      </div>
    );
  }

  const poolName = names[poolAddress] || `${poolAddress.slice(0, 6)}...${poolAddress.slice(-4)}`;
  const titleSuffix = markoutTime === 'brontes' ? '(Observed)' : `(Markout ${markoutTime}s)`;
  
  // Calculate y-axis range and appropriate tick spacing
  const maxY = Math.max(...data.map(point => point.running_total_cents / 100));
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxY)));
  const tickSpacing = magnitude / 2;
  const numTicks = Math.ceil(maxY / tickSpacing);

  const title = `Cumulative LVR over Time for ${poolName} ${titleSuffix}`;
  const baseLayout = createBaseLayout(title);

  return (
    <div className="w-full bg-black rounded-2xl border border-[#212121] p-6">
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
              width: 2,
            },
            hoverinfo: 'x+y' as const,
            hoverlabel: {
              bgcolor: '#424242',
              bordercolor: plotColors.accent,
              font: { color: '#ffffff', size: fontConfig.sizes.hover, family: fontConfig.family }
            },
            showlegend: false // Remove trace legend
          }
        ]}
        layout={{
          ...baseLayout,
          xaxis: {
            ...baseLayout.xaxis,
            title: {
              text: 'Block Number',
              font: { color: plotColors.accent, size: fontConfig.sizes.axisTitle, family: fontConfig.family },
              standoff: 20
            },
            tickformat: ',d',
            tickfont: { color: '#ffffff', family: fontConfig.family },
            showgrid: true,
            gridcolor: '#212121',
            automargin: true,
            tickangle: 0
          },
          yaxis: {
            ...baseLayout.yaxis,
            tickformat: '$,.2f',
            tickfont: { color: '#ffffff', family: fontConfig.family },
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
          height: 600,
          margin: { 
            l: 50,
            r: 120,
            b: 100,
            t: 80,
            pad: 10
          },
          hovermode: 'closest'
        }}
        config={{
          ...commonConfig,
          displayModeBar: true,
          displaylogo: false,
          modeBarButtonsToAdd: ['zoomIn2d', 'zoomOut2d', 'autoScale2d'],
          modeBarButtonsToRemove: ['lasso2d', 'select2d'],
          toImageButtonOptions: {
            format: 'png',
            filename: `running_total_lvr_${poolAddress}`,
            height: 600,
            width: 1200,
            scale: 2
          }
        }}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};

export default SoleRunningTotal;