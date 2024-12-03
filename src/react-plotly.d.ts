declare module 'react-plotly.js' {
    import * as Plotly from 'plotly.js';
    import * as React from 'react';
  
    interface PlotParams {
      data: Plotly.Data[];
      layout?: Partial<Plotly.Layout>;
      frames?: Plotly.Frame[];
      config?: Partial<Plotly.Config>;
      style?: React.CSSProperties;
      useResizeHandler?: boolean;
      onInitialized?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
      onUpdate?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
      onPurge?: (figure: Plotly.Figure, graphDiv: HTMLElement) => void;
      onError?: (err: Error) => void;
      onClick?: (event: Plotly.PlotMouseEvent) => void;
      onHover?: (event: Plotly.PlotMouseEvent) => void;
      onUnhover?: (event: Plotly.PlotMouseEvent) => void;
      onSelected?: (event: Plotly.PlotSelectionEvent) => void;
      onDeselect?: (event: Plotly.PlotSelectionEvent) => void;
      onDoubleClick?: (event: Plotly.PlotMouseEvent) => void;
    }
  
    class Plot extends React.Component<PlotParams> {}
    export default Plot;
  }