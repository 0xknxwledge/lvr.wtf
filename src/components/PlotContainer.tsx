import React, { ReactNode } from 'react';

interface PlotContainerProps {
  children: ReactNode;
  className?: string;
}

const PlotContainer: React.FC<PlotContainerProps> = ({ children, className = '' }) => {
  return (
    <div className={`w-full mb-4 sm:mb-6 md:mb-8 last:mb-0 ${className}`}>
      <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 
                    rounded-lg sm:rounded-xl md:rounded-2xl 
                    border border-[#B2AC88]/20 
                    p-3 sm:p-6 md:p-8
                    hover:border-[#B2AC88]/30 
                    transition-colors duration-300">
        <div className="w-full overflow-x-auto">
          <div className="min-w-[300px]">
            {children}
          </div>
        </div>
      </div>
    </div>
  );
};

export default PlotContainer;