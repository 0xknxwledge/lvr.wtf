import React from 'react';
import SidebarNavigation from './sidebar';

interface DataLayoutProps {
  children: React.ReactNode;
}

function DataLayout({ children }: DataLayoutProps) {
  return (
    <div className="flex bg-[#030304]">
      <div className="fixed w-64">
        <SidebarNavigation />
      </div>
      <div className="ml-64 w-full">
        <main className="w-full">
          {children}
        </main>
      </div>
    </div>
  );
}

export default DataLayout;