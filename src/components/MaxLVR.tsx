import React, { useState, useEffect } from 'react';
import names from '../names';

interface MaxLVRData {
  block_number: number;
  lvr_cents: number;
  pool_name: string;
}

interface MaxLVRDisplayProps {
  poolAddress: string;
  markoutTime: string;
}

const MaxLVRDisplay: React.FC<MaxLVRDisplayProps> = ({ poolAddress, markoutTime }) => {
  const [maxLVRData, setMaxLVRData] = useState<MaxLVRData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMaxLVR = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime
        });

        const response = await fetch(`http://127.0.0.1:3000/max_lvr?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        setMaxLVRData(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchMaxLVR();
  }, [poolAddress, markoutTime]);

  return (
    <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
      <h3 className="text-xl font-semibold mb-6">Maximum LVR</h3>
      
      {isLoading ? (
        <div className="text-white text-center py-8">Loading...</div>
      ) : error ? (
        <div className="text-red-500 text-center py-8">{error}</div>
      ) : maxLVRData ? (
        <div className="space-y-6">
          <div>
            <p className="text-[#b4d838] mb-2">Block Number</p>
            <p className="text-4xl font-semibold text-white">
              {maxLVRData.block_number.toLocaleString()}
            </p>
          </div>
          <hr className="border-[#333333]" />
          <div>
            <p className="text-[#b4d838] mb-2">Maximum LVR</p>
            <p className="text-4xl font-semibold text-white">
              ${(maxLVRData.lvr_cents / 100).toLocaleString(undefined, {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2
              })}
            </p>
          </div>
        </div>
      ) : (
        <div className="text-white text-center py-8">No data available</div>
      )}
    </div>
  );
};

export default MaxLVRDisplay;