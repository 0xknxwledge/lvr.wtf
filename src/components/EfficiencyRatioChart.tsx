import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';

interface MarkoutRatio {
  markout_time: string;
  ratio: number;
  realized_lvr_cents: number;
  theoretical_lvr_cents: number;
}

interface LVRRatioResponse {
  ratios: MarkoutRatio[];
}

const EfficiencyRatioChart: React.FC = () => {
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

  const plotData = [{
    x: sortedData.map(d => `${d.markout_time}s`),
    y: sortedData.map(d => d.ratio),
    type: 'bar' as const,
    marker: {
      color: '#b4d838',
      opacity: 0.8,
    },
    hovertemplate: 
      'Ratio: %{y:.1f}%<br>' +
      'Realized: $%{customdata[0]:,.2f}<br>' +
      'Theoretical: $%{customdata[1]:,.2f}<extra></extra>',
    customdata: sortedData.map(d => [
      d.realized_lvr_cents / 100,
      d.theoretical_lvr_cents / 100
    ]),
  }];

  return (
    <div className="w-full">
      <Plot
        data={plotData}
        layout={{
          title: {
            text: 'LVR Efficiency Ratio by Markout Time',
            font: { color: '#b4d838', size: 16 },
            y: 0.95
          },
          xaxis: {
            title: {
              text: 'Markout Time',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickfont: { color: '#ffffff' },
            fixedrange: true,
          },
          yaxis: {
            title: {
              text: 'Efficiency Ratio (%)',
              font: { color: '#b4d838', size: 14 },
              standoff: 20
            },
            tickformat: '.1f',
            ticksuffix: '%',
            tickfont: { color: '#ffffff' },
            range: [0, 100],
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
          bargap: 0.3,
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

export default EfficiencyRatioChart;