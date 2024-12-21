import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="mt-16">
      <div className="bg-[#101010] px-20 py-10">
        <div className="flex justify-between items-center">
          <div>
            <p className="text-[#98a1b2] text-sm mb-3">Built with ❤️ by Fenbushi's Research team and Sorella Labs</p>
          </div>
          <div className="flex gap-12">
            <a 
              href="https://fenbushi.vc/" 
              className="text-[#98a1b2] text-sm underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              Visit Fenbushi Capital
            </a>
            <a 
              href="https://sorellalabs.xyz/" 
              className="text-[#98a1b2] text-sm underline"
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