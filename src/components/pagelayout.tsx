import React from 'react';

interface PageLayoutProps {
  title: string;
  controls?: React.ReactNode;
  children: React.ReactNode;
}

const PageLayout: React.FC<PageLayoutProps> = ({ title, controls, children }) => {
  return (
    <div className="font-['Menlo'] px-4 sm:px-6 md:px-8 py-4 sm:py-6 md:py-8 bg-[#030304] min-h-screen">
      <div className="max-w-7xl mx-auto">
        {/* Header section with responsive spacing and font sizes */}
        <div className="flex flex-col items-center mb-4 sm:mb-6 md:mb-8">
          <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold text-[#F651AE] mb-4 text-center">
            {title}
          </h1>
          
          {/* Controls section with black background */}
          {controls && (
            <div className="w-full max-w-xl mx-auto">
              <div className="w-full flex flex-col sm:flex-row gap-4 justify-center items-center bg-[#030304] p-6 rounded-lg">
                {controls}
              </div>
            </div>
          )}
        </div>
        
        {/* Content section with responsive spacing */}
        <div className="space-y-6 sm:space-y-8 md:space-y-12">
          {children}
        </div>
      </div>
    </div>
  );
};

export default PageLayout;