import React from 'react';
import { Link, useLocation } from 'react-router-dom';

const SidebarNavigation: React.FC = () => {
  const location = useLocation();

  const navItems = [
    { name: 'Aggegrate', path: '/aggregate' },
    { name: 'Pool', path: '/pool' },
    { name: 'Cluster', path: '/cluster' },
  ];

  return (
    <nav className="bg-[#030304] w-64 h-screen">
      <div className="p-4">
        <ul>
          {navItems.map((item) => (
            <li key={item.name} className="mb-4 relative">
              <Link
                to={item.path}
                className={`block py-2 px-4 text-lg ${
                  location.pathname === item.path
                    ? 'text-[#b4d838] font-semibold'
                    : 'text-gray-300 hover:text-white'
                }`}
              >
                {item.name}
                {location.pathname === item.path && (
                  <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#b4d838]" />
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