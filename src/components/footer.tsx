import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="font-['Menlo'] mt-16">
      <div className="bg-[#161616] px-4 md:px-20 py-10">
        <div className="flex flex-col items-center space-y-6">
          <div className="flex flex-col space-y-2">
            <p className="text-white text-sm text-center">
              Built with <span className="text-[#F651AE]">❤️</span> by
            </p>
            <p className="text-white text-sm text-center">
              Fenbushi's Research team and Sorella Labs
            </p>
          </div>

          <div className="flex gap-8">
            <a 
              href="https://fenbushi.vc/" 
              className="text-white text-sm underline hover:text-[#F651AE] transition-colors duration-200"
              target="_blank"
              rel="noopener noreferrer"
            >
              Visit Fenbushi Capital
            </a>
            <a 
              href="https://sorellalabs.xyz/" 
              className="text-white text-sm underline hover:text-[#F651AE] transition-colors duration-200"
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