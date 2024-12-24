import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { plotColors, createBaseLayout, commonConfig } from '../plotUtils';

interface CategoryData {
  name: string;
  total_lvr_cents: number;
}

interface CategoryPieResponse {
  clusters: CategoryData[];
  total_lvr_cents: number;
}

interface CategoryPieChartProps {
  selectedMarkout: string;
}

const CategoryPieChart: React.FC<CategoryPieChartProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryPieResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Custom color palette - more distinct shades while maintaining theme
  const pieColors = [
    '#B2AC88',  // Sage green
    '#4A5D23',  // Deep forest green
    '#D4E7A5',  // Light sage
    '#2D3A15',  // Very dark forest
    '#98B147',  // Medium sage
    '#6B705C',  // Muted olive
    '#C2C5AA'   // Light olive
  ];

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        const params = new URLSearchParams({ markout_time: selectedMarkout });
        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/pie?${params.toString()}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-white">Loading...</p>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-red-500">{error || 'No data available'}</p>
      </div>
    );
  }

  const values = data.clusters.map(cluster => cluster.total_lvr_cents / 100);
  const percentages = data.clusters.map(cluster => 
    (cluster.total_lvr_cents / data.total_lvr_cents) * 100
  );
  
  const labels = data.clusters.map((cluster, index) => 
    `${cluster.name} (${percentages[index].toFixed(1)}%)`
  );

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  const baseLayout = createBaseLayout(`Total LVR by Category ${titleSuffix}`);

  return (
    <Plot
      data={[
        {
          values,
          labels,
          type: 'pie',
          textinfo: 'label',
          textposition: 'outside',
          automargin: true,
          marker: {
            colors: pieColors,
            line: {
              color: '#000000',
              width: 2
            }
          },
          textfont: {
            color: '#FFFFFF',
            size: 12
          },
          hoverlabel: {
            bgcolor: '#424242',
            font: { color: '#ffffff' }
          },
          hovertemplate: '<b>%{label}</b><br>$%{value:,.2f}<extra></extra>'
        }
      ]}
      layout={{
        ...baseLayout,
        showlegend: false,
        height: 500,
        margin: { t: 50, b: 50, l: 50, r: 50 },
        annotations: [{
          text: '',
          showarrow: false,
          x: 0.5,
          y: 1.1,
          xref: 'paper',
          yref: 'paper',
          font: {
            color: '#FFFFFF',
            size: 16
          }
        }]
      }}
      config={commonConfig}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

export default CategoryPieChart;