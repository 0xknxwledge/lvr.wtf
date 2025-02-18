import React from 'react';
import { Link, useLocation } from 'react-router-dom';

const SidebarNavigation: React.FC = () => {
  const location = useLocation();

  const navItems = [
    { name: 'Aggregate', path: '/aggregate' },
    { name: 'Pool', path: '/pool' },
    { name: 'Category', path: '/category' },
  ];

  return (
    <nav className="font-['Geist'] bg-[#30283A] w-64 h-screen">
      <div className="p-4">
        <ul>
          {navItems.map((item) => (
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
  );
};

export default SidebarNavigation;