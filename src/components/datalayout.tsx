import React from 'react';
import SidebarNavigation from './sidebar';

interface DataLayoutProps {
  children: React.ReactNode;
}

function DataLayout({ children }: DataLayoutProps) {
  return (
    <div className="flex h-screen overflow-hidden">
      <SidebarNavigation />
      <main className="flex-1 overflow-y-auto overscroll-contain bg-[#030304]">
        {children}
      </main>
    </div>
  );
}

export default DataLayout;