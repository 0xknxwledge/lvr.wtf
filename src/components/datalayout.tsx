import React from 'react';
import SidebarNavigation from './sidebar';

interface DataLayoutProps {
  children: React.ReactNode;
}

function DataLayout({ children }: DataLayoutProps) {
  return (
    <div className="font-['Menlo'] flex bg-[#030304] min-h-screen">
      {/* Fixed width sidebar */}
      <div className="fixed w-64">
        <SidebarNavigation />
      </div>
      
      {/* Main content area with consistent margin */}
      <div className="flex-1 ml-64">
        <main>
          {children}
        </main>
      </div>
    </div>
  );
}

export default DataLayout;