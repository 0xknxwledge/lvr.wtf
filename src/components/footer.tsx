import React from 'react';

function Footer() {
  return (
    <footer className="mt-16">
      <div className="bg-[#101010] px-20 py-10">
        <div className="flex justify-between items-center">
          <div>
            <p className="text-[#98a1b2] text-sm mb-3">Proudly built by Fenbushi's Research team and Sorella Labs</p>
            <p className="text-[#98a1b2] text-[1.625rem] font-semibold">LVR.wtf</p>
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
      <div className="bg-[#0b0b0e] px-20 py-5 flex justify-between items-center">
        <p className="text-[#98a1b2] text-sm">© Fenbushi Capital 2024 | All Rights Reserved</p>
        <div className="flex gap-5">
          <a
            href="https://discord.gg/T9XsKu25"
            target="_blank"
            rel="noopener noreferrer"
            aria-label="Join our Discord"
          >
            <img src="/discord.svg" alt="Discord" className="w-6 h-6" />
          </a>
          <a
            href="https://x.com/SorellaLabs"
            target="_blank"
            rel="noopener noreferrer"
            aria-label="Follow us on X"
          >
            <img src="/X.svg" alt="X" className="w-6 h-6" />
          </a>
        </div>
      </div>
    </footer>
  );
}

export default Footer;