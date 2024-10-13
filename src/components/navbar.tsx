import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import Overview from '../pages/overview';
import All from '../pages/all';
import Group from '../pages/group';
import Pair from '../pages/pair';

function NavBar() {
    const location = useLocation();
  
    return (
      <nav className="w-full h-24 px-20 py-6 bg-[#0b0b0e] border-b border-[#1a1a1a] flex justify-between items-center">
        <div className="text-white text-[26px] font-semibold font-['General Sans'] leading-tight">
          <Link to="/">LVR.wtf</Link>
        </div>
        <div className="h-14 flex justify-between items-center">
          <div className="h-5 flex justify-center items-center gap-10">
            <Link 
              to="/" 
              className={`text-lg font-medium font-['General Sans'] leading-tight ${location.pathname === '/' ? 'text-[#b4d838]' : 'text-white'}`}
            >
              Overview
            </Link>
            <Link 
              to="/all" 
              className={`text-lg font-medium font-['General Sans'] leading-tight ${location.pathname !== '/' ? 'text-[#b4d838]' : 'text-white'}`}
            >
              Data
            </Link>
          </div>
          <div className="ml-10">
            <a
              href="https://sorellalabs.com"
              target="_blank"
              rel="noopener noreferrer"
              className="px-6 py-4 bg-[#b4d838] rounded-[156px] border text-black text-lg font-medium font-['General Sans'] leading-tight flex items-center"
            >
              <span className="mr-2">⟳</span> Visit Sorella Labs
            </a>
          </div>
        </div>
      </nav>
    );
}

export default NavBar;
