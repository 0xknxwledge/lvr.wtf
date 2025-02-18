import React from 'react';
import { Link, useLocation } from 'react-router-dom';

interface DataLayoutProps {
  children: React.ReactNode;
}

function DataLayout({ children }: DataLayoutProps) {
  const location = useLocation();

  const navItems = [
    { name: 'Overview', path: '/' },
    { name: 'Aggregate', path: '/aggregate' },
    { name: 'Pool', path: '/pool' },
    { name: 'Category', path: '/category' },
  ];

  return (
    <div className="font-['Menlo'] flex bg-[#030304] min-h-screen">
      {/* Fixed width sidebar - visible on desktop, hidden on mobile */}
      <div className="hidden lg:block fixed w-64">
        <nav className="bg-[#030304] w-64 h-screen">
          <div className="p-4">
            <ul>
              {navItems.slice(1).map((item) => (
                <li key={item.name} className="mb-4 relative">
                  <Link
                    to={item.path}
                    className={`block py-2 px-4 text-lg transition-all duration-200 ${
                      location.pathname === item.path
                        ? 'text-[#F651AE] font-semibold'
                        : 'text-white hover:text-[#F651AE]'
                    }`}
                  >
                    {item.name}
                    {location.pathname === item.path && (
                      <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#F651AE]" />
                    )}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </nav>
      </div>
      
      {/* Main content area with margin for desktop sidebar */}
      <div className="flex-1 lg:ml-64">
        <main>
          {children}
        </main>
      </div>
    </div>
  );
}

export default DataLayout;