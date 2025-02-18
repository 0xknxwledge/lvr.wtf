import React, { useState, useEffect } from 'react';
import names from '../../names';

interface NonZeroProportionResponse {
  pool_name: string;
  pool_address: string;
  non_zero_proportion: number;
  total_blocks: number;
  non_zero_blocks: number;
}

interface NonZeroProportionProps {
  poolAddress: string;
  selectedMarkout: string;
}

const NonZeroProportion: React.FC<NonZeroProportionProps> = ({ poolAddress, selectedMarkout }) => {
  const [data, setData] = useState<NonZeroProportionResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);

        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: selectedMarkout
        });

        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/non_zero_proportion?${params.toString()}`
        );
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData = await response.json();
        setData(jsonData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    if (poolAddress && selectedMarkout) {
      fetchData();
    }
  }, [poolAddress, selectedMarkout]);

  if (isLoading) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-white text-base md:text-lg">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-red-500 text-sm md:text-base px-4 py-2 bg-red-500/10 rounded">{error}</p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-white text-base md:text-lg">No data available</p>
        </div>
      </div>
    );
  }

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;
  const colors = {
    primary: '#F651AE',    // Pink accent
    secondary: '#8247E5'   // Purple accent
  };

  return (
    <div className="w-full">
      <div className="mb-8">
        <h2 className="text-[#F651AE] text-base md:text-lg text-center px-4 font-['Menlo']">
          Percentage of Blocks with Non-Zero LVR for {names[data.pool_address] || data.pool_name} {titleSuffix}
        </h2>
      </div>
      {/* Added padding to outer container to prevent hover effect from being cut off */}
      <div className="grid grid-cols-1 gap-4 max-w-xl mx-auto p-4">
        {/* Wrapper div with padding to handle hover scaling */}
        <div className="p-4">
          <div 
            className="relative flex flex-col items-center justify-center p-4 md:p-6 lg:p-8 rounded-xl overflow-hidden transition-all duration-300 hover:scale-105 min-h-[200px] transform-gpu"
            style={{
              background: `linear-gradient(135deg, rgba(48, 40, 58, 0.5), rgba(3, 3, 4, 0.8))`,
              border: `1px solid ${colors.secondary}20`,
              boxShadow: `0 4px 30px ${colors.secondary}10`
            }}
          >
            {/* Gradient background overlay */}
            <div 
              className="absolute inset-0 opacity-10 transition-opacity duration-300 hover:opacity-20"
              style={{
                background: `linear-gradient(135deg, ${colors.primary}, ${colors.secondary})`
              }}
            />
            
            {/* Content */}
            <div className="relative z-10 flex flex-col items-center w-full">
              <h3 className="text-sm md:text-base lg:text-lg font-medium mb-4 text-center text-white break-words max-w-full px-2">
                {names[data.pool_address] || data.pool_name}
              </h3>
              <p className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl font-bold mb-2 md:mb-4 text-center" 
                 style={{ color: colors.primary }}>
                {(data.non_zero_proportion * 100).toFixed(2)}
                <span className="text-xl md:text-2xl lg:text-3xl">%</span>
              </p>
              <div className="text-xs md:text-sm text-gray-400 mt-2 text-center w-full px-2">
                <span className="font-medium" style={{ color: colors.primary }}>
                  {data.non_zero_blocks.toLocaleString()}
                </span>
                <span className="mx-1">/</span>
                <span className="font-medium">
                  {data.total_blocks.toLocaleString()}
                </span>
                <br />
                <span className="text-gray-500 text-xs">blocks</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NonZeroProportion;