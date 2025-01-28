import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Menu, X } from 'lucide-react';

const MobileNavigation = () => {
  const [isOpen, setIsOpen] = useState(false);
  const location = useLocation();

  const navItems = [
    { name: 'Overview', path: '/' },
    { name: 'Aggregate', path: '/aggregate' },
    { name: 'Pool', path: '/pool' },
    { name: 'Category', path: '/category' },
  ];

  const toggleMenu = () => setIsOpen(!isOpen);

  return (
    <div className="lg:hidden">
      <button
        onClick={toggleMenu}
        className="p-2 text-white hover:text-[#b4d838] transition-colors duration-200"
      >
        {isOpen ? <X size={24} /> : <Menu size={24} />}
      </button>

      {isOpen && (
        <div className="fixed inset-0 z-50 bg-[#0b0b0e]">
          <div className="flex flex-col h-full">
            {/* Header */}
            <div className="flex justify-between items-center p-6 border-b border-[#1a1a1a]">
              <Link 
                to="/" 
                className="text-2xl font-semibold text-white"
                onClick={() => setIsOpen(false)}
              >
                LVR.wtf
              </Link>
              <button
                onClick={toggleMenu}
                className="p-2 text-white hover:text-[#b4d838] transition-colors duration-200"
              >
                <X size={24} />
              </button>
            </div>

            {/* Navigation Links */}
            <nav className="flex-1 overflow-y-auto py-6">
              <ul className="space-y-4">
                {navItems.map((item) => (
                  <li key={item.name}>
                    <Link
                      to={item.path}
                      onClick={() => setIsOpen(false)}
                      className={`block px-6 py-3 text-lg ${
                        location.pathname === item.path
                          ? 'text-[#b4d838] bg-[#1a1a1a]'
                          : 'text-white hover:text-[#b4d838]'
                      }`}
                    >
                      {item.name}
                    </Link>
                  </li>
                ))}
              </ul>
            </nav>

            {/* Footer Links */}
            <div className="p-6 border-t border-[#1a1a1a]">
              <div className="flex flex-col space-y-4">
                <a
                  href="https://fenbushi.vc"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[#B2AC88] hover:text-[#b4d838] text-center"
                >
                  Visit Fenbushi Capital
                </a>
                <a
                  href="https://sorellalabs.xyz"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[#B2AC88] hover:text-[#b4d838] text-center"
                >
                  Visit Sorella Labs
                </a>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default MobileNavigation;