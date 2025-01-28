import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import MobileNavigation from './mobilenav';

function NavBar() {
    const location = useLocation();
  
    return (
      <nav className="font-['Menlo'] w-full h-auto md:h-24 px-4 md:px-20 py-4 md:py-6 bg-[#0b0b0e] border-b border-[#1a1a1a]">
        <div className="flex justify-between items-center">
          {/* Logo - visible on all screens */}
          <div className="text-white text-xl md:text-[26px] font-semibold font-['General Sans'] leading-tight">
            <Link to="/">LVR.wtf</Link>
          </div>

          {/* Desktop Navigation - hidden on mobile */}
          <div className="hidden lg:flex h-14 justify-between items-center">
            <div className="h-5 flex justify-center items-center gap-10">
              <Link 
                to="/" 
                className={`text-lg font-medium font-['General Sans'] leading-tight transition-colors duration-200 
                  ${location.pathname === '/' 
                    ? 'text-[#b4d838]' 
                    : 'text-[#B2AC88] hover:text-[#8B9556]'}`}
              >
                Overview
              </Link>
              <Link 
                to="/aggregate" 
                className={`text-lg font-medium font-['General Sans'] leading-tight transition-colors duration-200
                  ${location.pathname !== '/' 
                    ? 'text-[#b4d838]' 
                    : 'text-[#B2AC88] hover:text-[#8B9556]'}`}
              >
                Data
              </Link>
            </div>
            
            {/* Partner logos - desktop only */}
            <div className="ml-10 flex items-center gap-6">
              <a
                href="https://fenbushi.vc"
                target="_blank"
                rel="noopener noreferrer"
                className="h-16 opacity-90 hover:opacity-100 transition-opacity duration-200"
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
                className="h-16 opacity-90 hover:opacity-100 transition-opacity duration-200"
              >
                <img 
                  src="/sorella.png" 
                  alt="Sorella Labs" 
                  className="h-full"
                />
              </a>
            </div>
          </div>

          {/* Mobile Navigation - visible only on mobile/tablet */}
          <div className="lg:hidden">
            <MobileNavigation />
          </div>
        </div>
      </nav>
    );
}

export default NavBar;