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

        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/non_zero_proportion?${params.toString()}`);
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
      <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-red-500">{error}</p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-white">No data available</p>
        </div>
      </div>
    );
  }

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="w-full bg-black rounded-lg border border-[#212121] p-6">
      <div className="mb-4">
        <h2 className="text-[#b4d838] text-base text-center">
          Percentage of Blocks with Non-Zero LVR for {names[data.pool_address] || data.pool_name} {titleSuffix}
        </h2>
      </div>
      <div className="flex flex-col items-center justify-center">
        <p className="text-5xl font-semibold text-[#b4d838]">
          {(data.non_zero_proportion * 100).toFixed(2)}%
        </p>
        <p className="text-sm text-gray-500 mt-2">
          {data.non_zero_blocks.toLocaleString()} / {data.total_blocks.toLocaleString()} blocks
        </p>
      </div>
    </div>
  );
};

export default NonZeroProportion;