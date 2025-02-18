import React, { ReactNode } from 'react';

interface PlotContainerProps {
  children: ReactNode;
  className?: string;
}

const PlotContainer: React.FC<PlotContainerProps> = ({ children, className = '' }) => {
  return (
    <div className={`w-full mb-4 sm:mb-6 md:mb-8 last:mb-0 ${className}`}>
      <div className="bg-gradient-to-br from-[#30283A] via-[#8247E5]/10 to-[#F651AE]/10 
                    rounded-lg sm:rounded-xl md:rounded-2xl 
                    border border-[#8247E5]/20 
                    p-3 sm:p-6 md:p-8
                    hover:border-[#F651AE]/30 
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