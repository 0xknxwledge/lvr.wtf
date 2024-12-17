import React from 'react';

const PageLayout = ({ 
  title, 
  controls, 
  children 
}: { 
  title: string;
  controls?: React.ReactNode;
  children: React.ReactNode;
}) => {
  return (
    <div className="px-8 py-8 bg-[#030304] min-h-screen">
      <div className="max-w-7xl mx-auto">
        {/* Header section with centered title */}
        <div className="flex flex-col items-center mb-8">
          <h1 className="text-4xl font-bold text-white mb-4">{title}</h1>
          {controls && (
            <div className="flex gap-4 items-center">
              {controls}
            </div>
          )}
        </div>
        
        {/* Content section */}
        <div className="space-y-12">
          {children}
        </div>
      </div>
    </div>
  );
};

export default PageLayout;