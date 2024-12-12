import React from 'react';

export const markoutOptions = [
  { value: 'brontes', label: 'Observed' },
  { value: '-2.0', label: '-2.0s' },
  { value: '-1.5', label: '-1.5s' },
  { value: '-1.0', label: '-1.0s' },
  { value: '-0.5', label: '-0.5s' },
  { value: '0.0', label: '0s' },
  { value: '0.5', label: '+0.5s' },
  { value: '1.0', label: '+1.0s' },
  { value: '1.5', label: '+1.5s' },
  { value: '2.0', label: '+2.0s' },
];

interface MarkoutSelectProps {
  selectedMarkout: string;
  onChange: (value: string) => void;
}

const MarkoutSelect: React.FC<MarkoutSelectProps> = ({ selectedMarkout, onChange }) => {
  return (
    <select
      value={selectedMarkout}
      onChange={(e) => onChange(e.target.value)}
      className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded cursor-pointer appearance-none min-w-[160px]"
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
      {markoutOptions.map((option) => (
        <option 
          key={option.value} 
          value={option.value}
          className="bg-[#161616] text-white"
        >
          {option.label}
        </option>
      ))}
    </select>
  );
};

export default MarkoutSelect;