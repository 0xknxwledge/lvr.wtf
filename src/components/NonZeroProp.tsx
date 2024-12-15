import React, { useState, useEffect } from 'react';
import names from '../names';

interface NonZeroProportionResponse {
  pool_name: string;
  pool_address: string;
  non_zero_proportion: number;
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
        });

        params.append('markout_time', selectedMarkout);

        const response = await fetch(`http://127.0.0.1:3000/non_zero_proportion?${params.toString()}`);
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
      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-red-500">{error}</p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
        <div className="flex items-center justify-center h-48">
          <p className="text-white">No data available</p>
        </div>
      </div>
    );
  }

  const percentage = (data.non_zero_proportion * 100).toFixed(2);
  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
      <h3 className="text-xl font-semibold mb-4">Proportion of Blocks with LVR {titleSuffix}</h3>
      <div className="flex items-center justify-center h-[180px]">
        <p className="text-7xl font-semibold">{percentage}%</p>
      </div>
    </div>
  );
};

export default NonZeroProportion;