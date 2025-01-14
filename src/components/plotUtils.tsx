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

// Font configuration
export const fontConfig = {
  family: 'Menlo',
  sizes: {
    title: 16,
    axisTitle: 14,
    axisLabel: 10,
    legend: 12,
    annotation: 12,
    hover: 12
  }
};

// Helper function to create base layout
export function createBaseLayout(title: string): Partial<Layout> {
  return {
    title: {
      text: title,
      font: { 
        family: fontConfig.family,
        color: plotColors.accent,
        size: fontConfig.sizes.title 
      },
      x: 0.5,
      y: 0.95
    },
    paper_bgcolor: '#000000',
    plot_bgcolor: '#000000',
    font: {
      family: fontConfig.family
    },
    xaxis: {
      title: {
        font: { 
          family: fontConfig.family,
          color: plotColors.accent,
          size: fontConfig.sizes.axisTitle 
        },
        standoff: 20
      },
      tickfont: { 
        family: fontConfig.family,
        color: '#ffffff',
        size: fontConfig.sizes.axisLabel 
      },
      showgrid: false,
      gridcolor: '#212121',
      zeroline: false
    },
    yaxis: {
      title: {
        font: { 
          family: fontConfig.family,
          color: plotColors.accent,
          size: fontConfig.sizes.axisTitle 
        },
        standoff: 40
      },
      tickfont: { 
        family: fontConfig.family,
        color: '#ffffff',
        size: fontConfig.sizes.axisLabel
      },
      showgrid: true,
      gridcolor: '#212121',
      zeroline: false
    },
    showlegend: true,
    legend: {
      font: { 
        family: fontConfig.family,
        color: '#ffffff',
        size: fontConfig.sizes.legend
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
        family: fontConfig.family,
        color: '#ffffff',
        size: fontConfig.sizes.hover
      }
    },
    hovermode: 'closest'
  };
}

// Helper function for creating annotation configurations
export function createAnnotationConfig(overrides = {}) {
  return {
    font: {
      family: fontConfig.family,
      color: '#ffffff',
      size: fontConfig.sizes.annotation
    },
    bgcolor: '#424242',
    bordercolor: plotColors.accent,
    borderwidth: 2,
    borderpad: 4,
    ...overrides
  };
}

// Helper function for creating hover template configurations
export function createHoverTemplate(includeFields: string[] = []): string {
  const baseTemplate = '<b>%{x}</b><br>';
  const fields = includeFields.map(field => `${field}: %{${field}:,.2f}<br>`).join('');
  return baseTemplate + fields + '<extra></extra>';
}

// Helper function for creating common chart configurations
export const commonConfig = {
  responsive: true,
  displayModeBar: false,
  scrollZoom: false
} as const;

// Helper function for creating pie chart configurations
export function createPieChartLayout(title: string): Partial<Layout> {
  const baseLayout = createBaseLayout(title);
  return {
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
        family: fontConfig.family,
        color: '#FFFFFF',
        size: fontConfig.sizes.title
      }
    }]
  };
}

// Helper function for creating bar chart configurations
export function createBarChartLayout(title: string, withLegend = true): Partial<Layout> {
  const baseLayout = createBaseLayout(title);
  return {
    ...baseLayout,
    barmode: 'group',
    showlegend: withLegend,
    height: 500,
    margin: { l: 120, r: 50, b: 160, t: 80 },
    xaxis: {
      ...baseLayout.xaxis,
      tickangle: 45
    }
  };
}