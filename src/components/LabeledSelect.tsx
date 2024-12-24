import React from 'react';

interface SelectWrapperProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<{ value: string; label: string }>;
  id: string;
}

const SelectWrapper: React.FC<SelectWrapperProps> = ({ label, value, onChange, options, id }) => {
  return (
    <div className="flex flex-col gap-2">
      <label 
        htmlFor={id}
        className="text-[#b4d838] text-sm font-medium"
      >
        {label}
      </label>
      <select
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="px-4 py-2 bg-[#161616] text-white border border-[#8B9556] rounded cursor-pointer appearance-none min-w-[160px] 
                  hover:border-[#b4d838] focus:border-[#b4d838] focus:outline-none transition-colors duration-200"
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
        {options.map((option) => (
          <option 
            key={option.value} 
            value={option.value}
            className="bg-[#161616] text-white hover:bg-[#1a1a1a]"
          >
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
};

interface MarkoutSelectProps {
  selectedMarkout: string;
  onChange: (value: string) => void;
}

export const MarkoutSelect: React.FC<MarkoutSelectProps> = ({ selectedMarkout, onChange }) => {
  const markoutOptions = [
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

  return (
    <SelectWrapper
      label="Markout Time"
      value={selectedMarkout}
      onChange={onChange}
      options={markoutOptions}
      id="markout-select"
    />
  );
};

interface PoolSelectProps {
  selectedPool: string;
  onChange: (value: string) => void;
  names: Record<string, string>;
}

export const PoolSelect: React.FC<PoolSelectProps> = ({ selectedPool, onChange, names }) => {
  const poolOptions = Object.entries(names).map(([address, name]) => ({
    value: address,
    label: name,
  }));

  return (
    <SelectWrapper
      label="Uniswap Pool"
      value={selectedPool}
      onChange={onChange}
      options={poolOptions}
      id="pool-select"
    />
  );
};