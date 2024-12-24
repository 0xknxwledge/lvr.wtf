import { Layout } from 'plotly.js';

// Define color constants
export const plotColors = {
  primary: '#B2AC88',     // Sage green
  secondary: '#8B9556',   // Olive green
  accent: '#b4d838',      // Bright lime
  chartColors: [
    '#b4d838',   // Primary lime
    '#9fc732',   // Secondary lime
    '#8ab62c',   // Tertiary lime
    '#75a526',   // Deep lime
    '#609420',   // Forest lime
    '#4b831a',   // Dark lime
    '#367214'    // Very dark lime
  ],
  categoryPalette: {
    'Stable Pairs': '#B2AC88',      // Sage green
    'WBTC-WETH': '#8B9556',         // Olive green
    'USDC-WETH': '#4A5D23',         // Deep forest green
    'USDT-WETH': '#6B705C',         // Muted olive
    'DAI-WETH': '#A3B18A',          // Light sage
    'USDC-WBTC': '#588157',         // Forest green
    'Altcoin-WETH': '#344E41'       // Dark forest green
  }
};

// Helper function to create base layout
export function createBaseLayout(title: string): Partial<Layout> {
  return {
    title: {
      text: title,
      font: { 
        color: plotColors.accent,
        size: 16 
      },
      x: 0.5,
      y: 0.95
    },
    paper_bgcolor: '#000000',
    plot_bgcolor: '#000000',
    xaxis: {
      title: {
        font: { 
          color: plotColors.accent,
          size: 14 
        },
        standoff: 20
      },
      tickfont: { 
        color: '#ffffff',
        size: 10 
      },
      showgrid: false,
      gridcolor: '#212121',
      zeroline: false
    },
    yaxis: {
      title: {
        font: { 
          color: plotColors.accent,
          size: 14 
        },
        standoff: 40
      },
      tickfont: { 
        color: '#ffffff' 
      },
      showgrid: true,
      gridcolor: '#212121',
      zeroline: false
    },
    showlegend: true,
    legend: {
      font: { 
        color: '#ffffff' 
      },
      bgcolor: '#000000',
      bordercolor: '#212121'
    },
    margin: {
      l: 80,
      r: 50,
      b: 80,
      t: 100,
      pad: 4
    },
    hoverlabel: {
      bgcolor: '#424242',
      bordercolor: plotColors.accent,
      font: { 
        color: '#ffffff',
        size: 12 
      }
    },
    hovermode: 'closest'
  };
}

// Helper function for common hover template settings
export function createHoverTemplate(includeFields: string[] = []): string {
  const baseTemplate = '<b>%{x}</b><br>';
  const fields = includeFields.map(field => `${field}: %{${field}:,.2f}<br>`).join('');
  return baseTemplate + fields + '<extra></extra>';
}

// Helper function for setting up common chart configurations
export const commonConfig = {
  responsive: true,
  displayModeBar: false,
  scrollZoom: false
} as const;