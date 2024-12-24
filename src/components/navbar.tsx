import React from 'react';
import { Link, useLocation } from 'react-router-dom';

function NavBar() {
    const location = useLocation();
  
    return (
      <nav className="w-full h-24 px-20 py-6 bg-[#0b0b0e] border-b border-[#1a1a1a] flex justify-between items-center">
        <div className="text-white text-[26px] font-semibold font-['General Sans'] leading-tight">
          <Link to="/">lvr.wtf</Link>
        </div>
        <div className="h-14 flex justify-between items-center">
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
          <div className="ml-10 flex items-center gap-6">
            <a
              href="https://fenbushi.vc"
              target="_blank"
              rel="noopener noreferrer"
              className="h-16 opacity-90 hover:opacity-100 transition-opacity duration-200"
            >
              <img src="/fenbushi_white.png" alt="Fenbushi Capital" className="h-full" />
            </a>
            <a
              href="https://sorellalabs.xyz"
              target="_blank"
              rel="noopener noreferrer"
              className="h-16 opacity-90 hover:opacity-100 transition-opacity duration-200"
            >
              <img src="/sorella.png" alt="Sorella Labs" className="h-full" />
            </a>
          </div>
        </div>
      </nav>
    );
}

export default NavBar;