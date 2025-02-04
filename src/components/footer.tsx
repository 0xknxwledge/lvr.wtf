import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="font-['Menlo'] mt-16">
      <div className="bg-[#101010] px-4 md:px-20 py-10">
        <div className="flex flex-col items-center space-y-6">
          <div className="flex flex-col space-y-2">
            <p className="text-[#98a1b2] text-sm text-center">
              Built with <span className="text-red-500">❤️</span> by
            </p>
            <p className="text-[#98a1b2] text-sm text-center">
              Fenbushi's Research team and Sorella Labs
            </p>
          </div>

          <div className="flex gap-8">
            <a 
              href="https://fenbushi.vc/" 
              className="text-[#98a1b2] text-sm underline hover:text-[#b4d838] transition-colors duration-200"
              target="_blank"
              rel="noopener noreferrer"
            >
              Visit Fenbushi Capital
            </a>
            <a 
              href="https://sorellalabs.xyz/" 
              className="text-[#98a1b2] text-sm underline hover:text-[#b4d838] transition-colors duration-200"
              target="_blank"
              rel="noopener noreferrer"
            >
              Visit Sorella Labs
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;