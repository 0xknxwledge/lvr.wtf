import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import names from '../names';

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

  if (!data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-[#000000] rounded-lg border border-[#212121]">
        <div className="text-white text-lg">No data available</div>
      </div>
    );
  }

  const poolName = names[poolAddress] || `${poolAddress.slice(0, 6)}...${poolAddress.slice(-4)}`;
  const titleSuffix = markoutTime === 'brontes' ? '(Observed)' : `(Markout ${markoutTime}s)`;

  return (
    <div className="w-full">
      <Plot
        data={[
          {
            x: data.map(point => point.block_number),
            y: data.map(point => point.running_total_cents / 100), // Convert cents to dollars
            type: 'scatter',
            mode: 'lines',
            name: `${poolName} ${titleSuffix}`,
            line: {
              color: '#b4d838',
              width: 2,
            },
            hoverinfo: 'x+y' as const,
            hoverlabel: {
              bgcolor: '#424242',
              bordercolor: '#b4d838',
              font: { color: '#ffffff' }
            },
          }
        ]}
        layout={{
          title: {
            text: `Running Total LVR for ${poolName} ${titleSuffix}`,
            font: { color: '#b4d838', size: 16 },
            x: 0.5,
            y: 0.95,
          },
          xaxis: {
            title: {
              text: 'Block Number',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: ',d',
            tickfont: { color: '#ffffff' },
            showgrid: true,
            gridcolor: '#212121',
          },
          yaxis: {
            title: {
              text: 'Running Total LVR ($)',
              font: { color: '#b4d838', size: 14 },
              standoff: 30
            },
            tickformat: '$,.2f',
            tickfont: { color: '#ffffff' },
            showgrid: true,
            gridcolor: '#212121',
          },
          showlegend: false,
          autosize: true,
          height: 600,
          margin: { l: 80, r: 50, b: 80, t: 80 },
          paper_bgcolor: '#000000',
          plot_bgcolor: '#000000',
          hovermode: 'closest',
        }}
        config={{
          responsive: true,
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