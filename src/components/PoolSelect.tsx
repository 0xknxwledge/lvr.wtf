import React from 'react';
import names from '../names';

interface PoolSelectProps {
  selectedPool: string;
  onChange: (value: string) => void;
}

const PoolSelect: React.FC<PoolSelectProps> = ({ selectedPool, onChange }) => {
  return (
    <select
      value={selectedPool}
      onChange={(e) => onChange(e.target.value)}
      className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded cursor-pointer min-w-[200px]"
      style={{
        WebkitAppearance: 'none',
        MozAppearance: 'none',
        backgroundImage: `url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23b4d838' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e")`,
        backgroundRepeat: 'no-repeat',
        backgroundPosition: 'right 8px center',
        backgroundSize: '16px',
        paddingRight: '32px'
      }}
    >
      {Object.entries(names).map(([address, name]) => (
        <option key={address} value={address} className="bg-[#161616] text-white">
          {name}
        </option>
      ))}
    </select>
  );
};

export default PoolSelect;