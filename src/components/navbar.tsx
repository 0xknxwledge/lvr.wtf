import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Menu, X } from 'lucide-react';

const MobileNavigation = ({ isOpen, setIsOpen }: { isOpen: boolean; setIsOpen: (isOpen: boolean) => void }) => {
  const location = useLocation();
  const isOverviewPage = location.pathname === '/';

  const navItems = isOverviewPage ? [
    { name: 'Overview', path: '/' },
    { name: 'Data', path: '/aggregate' }
  ] : [
    { name: 'Overview', path: '/' },
    { name: 'Aggregate', path: '/aggregate' },
    { name: 'Pool', path: '/pool' },
    { name: 'Category', path: '/category' }
  ];

  return (
    <div className={`fixed inset-0 z-40 bg-[#0b0b0e] pt-14 transform transition-transform duration-300 ${
      isOpen ? 'translate-x-0' : '-translate-x-full'
    }`}>
      <nav className="h-full flex flex-col font-['Menlo']">
        <div className="flex-1 py-6">
          <ul className="space-y-2">
            {navItems.map((item) => (
              <li key={item.name} className="relative">
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
                  {location.pathname === item.path && (
                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#b4d838]" />
                  )}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      </nav>
    </div>
  );
};

const NavBar = () => {
  const [isOpen, setIsOpen] = useState(false);
  const location = useLocation();
  const isOverviewPage = location.pathname === '/';

  return (
    <header className="sticky top-0 z-50 bg-[#0b0b0e] border-b border-[#1a1a1a] font-['Menlo']">
      <div className="max-w-full mx-auto flex items-center justify-between px-4 lg:px-8 pb-1">
        <div className="flex items-center h-16">
          {/* Left section with logo - aligned with sidebar content */}
          <Link to="/" className="text-2xl font-semibold text-white lg:ml-1">
            LVR.wtf
          </Link>
        </div>

        {/* Right section with navigation and partner logos */}
        <div className="flex items-center space-x-6">
          {/* Desktop Navigation Links */}
          <div className="hidden lg:flex items-center space-x-6">
            <Link 
              to="/" 
              className={`text-lg ${
                location.pathname === '/'
                  ? 'text-[#b4d838]'
                  : 'text-[#B2AC88] hover:text-[#b4d838] transition-colors duration-200'
              }`}
            >
              Overview
            </Link>
            <Link 
              to="/aggregate" 
              className={`text-lg ${
                location.pathname.includes('/aggregate') || location.pathname.includes('/pool') || location.pathname.includes('/category')
                  ? 'text-[#b4d838]'
                  : 'text-[#B2AC88] hover:text-[#b4d838] transition-colors duration-200'
              }`}
            >
              Dashboard
            </Link>
          </div>

          {/* Partner Logos */}
          <a
            href="https://fenbushi.vc"
            target="_blank"
            rel="noopener noreferrer"
            className="h-12 opacity-90 hover:opacity-100 transition-opacity duration-200"
          >
            <img 
              src="/fenbushi_white.png" 
              alt="Fenbushi Capital" 
              className="h-full"
            />
          </a>
          <a
            href="https://sorellalabs.xyz"
            target="_blank"
            rel="noopener noreferrer"
            className="h-12 opacity-90 hover:opacity-100 transition-opacity duration-200"
          >
            <img 
              src="/sorella.png" 
              alt="Sorella Labs" 
              className="h-full"
            />
          </a>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setIsOpen(!isOpen)}
            className="lg:hidden p-2 text-white hover:text-[#b4d838] transition-colors duration-200"
          >
            {isOpen ? <X size={24} /> : <Menu size={24} />}
          </button>
        </div>
      </div>

      {/* Mobile Navigation */}
      <div className="lg:hidden">
        <MobileNavigation isOpen={isOpen} setIsOpen={setIsOpen} />
      </div>
    </header>
  );
};

export default NavBar;