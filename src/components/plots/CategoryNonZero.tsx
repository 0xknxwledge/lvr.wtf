import React, { useState, useEffect } from 'react';

interface CategoryNonZero {
  name: string;
  total_observations: number;
  non_zero_observations: number;
  non_zero_proportion: number;
}

interface CategoryNonZeroResponse {
  clusters: CategoryNonZero[];
}

interface CategoryNonZeroProps {
  selectedMarkout: string;
}

const CLUSTER_DISPLAY_NAMES: { [key: string]: string } = {
  "stable": "Stable Pairs",
  "wbtc_weth": "WBTC-WETH",
  "usdc_weth": "USDC-WETH",
  "usdt_weth": "USDT-WETH",
  "dai_weth": "DAI-WETH",
  "usdc_wbtc": "USDC-WBTC",
  "altcoin_weth": "Altcoin-WETH"
};

const CategoryNonZero: React.FC<CategoryNonZeroProps> = ({ selectedMarkout }) => {
  const [data, setData] = useState<CategoryNonZero[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);

        const params = new URLSearchParams({
          markout_time: selectedMarkout
        });

        const response = await fetch(`https://lvr-wtf-568975696472.us-central1.run.app/clusters/nonzero?${params.toString()}`);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const jsonData: CategoryNonZeroResponse = await response.json();
        const sortedData = [...jsonData.clusters].sort((a, b) => b.non_zero_proportion - a.non_zero_proportion);
        setData(sortedData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [selectedMarkout]);

  if (isLoading) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-white">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-red-500">{error}</p>
        </div>
      </div>
    );
  }

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed LVR)' : 
    `(Markout ${selectedMarkout}s)`;

  return (
    <div className="w-full">
      <div className="mb-4">
        <h2 className="text-[#b4d838] text-base text-center">
          Percentage of Blocks with Non-Zero LVR by Category {titleSuffix}
        </h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {data.map((item) => (
          <div 
            key={item.name}
            className="flex flex-col items-center justify-center"
          >
            <h3 className="text-lg font-medium mb-4 text-center text-gray-300">
              {CLUSTER_DISPLAY_NAMES[item.name] || item.name}
            </h3>
            <p className="text-5xl font-semibold text-[#b4d838]">
              {(item.non_zero_proportion * 100).toFixed(2)}%
            </p>
            <p className="text-sm text-gray-500 mt-2">
              {item.non_zero_observations.toLocaleString()} / {item.total_observations.toLocaleString()} blocks
            </p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default CategoryNonZero;