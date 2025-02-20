import { Layout } from 'plotly.js';

// Define color constants with new palette
export const plotColors = {
  primary: '#8247E5',     // Main purple from image
  secondary: '#30283A',   // Dark purple from image
  accent: '#F651AE',      // Pink from image
  chartColors: [
    '#8247E5',   // Main purple
    '#F651AE',   // Pink
    '#2F69ED',   // Blue
    '#30283A',   // Dark purple
    '#9B6FE8',   // Light purple (derived)
    '#FF7BC5',   // Light pink (derived)
    '#5785F0'    // Light blue (derived)
  ],
  categoryPalette: {
    'Stable Pairs': '#8247E5',      
    'WBTC-WETH':    '#F651AE',      
    'USDC-WETH':    '#2F69ED',      
    'USDT-WETH':    '#30283A',      
    'DAI-WETH':     '#9B6FE8',      
    'USDC-WBTC':    '#FF7BC5',      
    'Altcoin-WETH': '#5785F0'
  }
};

// Font configuration
//  - family: normal-weight Geist
//  - familyBold: same 'Geist', but will use 700 weight
export const fontConfig = {
  family: 'Geist',      
  familyBold: 'Geist',  // The same family name, since we declared both weights in input.css
  sizes: {
    title: 16,
    axisTitle: 14,
    axisLabel: 10,
    legend: 12,
    annotation: 12,
    hover: 12
  }
};

// Helper function to create a base layout with bold titles
export function createBaseLayout(title: string): Partial<Layout> {
  return {
    title: {
      text: title,
      font: { 
        // Chart title uses the bold variant
        family: fontConfig.familyBold,
        color: plotColors.accent,
        size: fontConfig.sizes.title 
      },
      x: 0.5,
      y: 0.95
    },
    paper_bgcolor: '#000000',
    plot_bgcolor: '#000000',
    // General text uses normal-weight Geist
    font: {
      family: fontConfig.family
    },
    xaxis: {
      // Axis titles also bold
      title: {
        font: { 
          family: fontConfig.familyBold,
          color: plotColors.accent,
          size: fontConfig.sizes.axisTitle 
        },
        standoff: 20
      },
      // Axis labels remain normal weight
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
      // Axis titles also bold
      title: {
        font: { 
          family: fontConfig.familyBold,
          color: plotColors.accent,
          size: fontConfig.sizes.axisTitle 
        },
        standoff: 40
      },
      // Axis labels remain normal weight
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
      // Legend font is normal weight
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
      // If you want annotation text bold, set `familyBold` here too
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
  const fields = includeFields.map(
    field => `${field}: %{${field}:,.2f}<br>`
  ).join('');
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
