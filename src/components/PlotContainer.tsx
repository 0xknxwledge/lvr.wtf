import React, { ReactNode } from 'react';

interface PlotContainerProps {
  children: ReactNode;
}

const PlotContainer: React.FC<PlotContainerProps> = ({ children }) => {
  return (
    <div className="w-full mb-8 last:mb-0">
      <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 rounded-2xl border border-[#B2AC88]/20 p-8 hover:border-[#B2AC88]/30 transition-colors duration-300">
        {children}
      </div>
    </div>
  );
};

export default PlotContainer;