import React, { useState, useEffect } from 'react';
import { plotColors } from '../plotUtils';

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

interface ColorSet {
  primary: string;
  secondary: string;
}

type ClusterName = 
  | "stable"
  | "wbtc_weth"
  | "usdc_weth"
  | "usdt_weth"
  | "dai_weth"
  | "usdc_wbtc"
  | "altcoin_weth";

type DisplayName = 
  | "Stable Pairs"
  | "WBTC-WETH"
  | "USDC-WETH"
  | "USDT-WETH"
  | "DAI-WETH"
  | "USDC-WBTC"
  | "Altcoin-WETH";

const CLUSTER_DISPLAY_NAMES: Record<ClusterName, DisplayName> = {
  "stable": "Stable Pairs",
  "wbtc_weth": "WBTC-WETH",
  "usdc_weth": "USDC-WETH",
  "usdt_weth": "USDT-WETH",
  "dai_weth": "DAI-WETH",
  "usdc_wbtc": "USDC-WBTC",
  "altcoin_weth": "Altcoin-WETH"
};

const CLUSTER_COLORS: Record<DisplayName, ColorSet> = {
  "Stable Pairs": { 
    primary: '#E2DFC9',
    secondary: '#d4d1b8'
  },
  "WBTC-WETH": { 
    primary: '#738C3A',
    secondary: '#5d7030'
  },
  "USDC-WETH": { 
    primary: '#A4C27B',
    secondary: '#8ba665'
  },
  "USDT-WETH": { 
    primary: '#2D3A15',
    secondary: '#1a2209'
  },
  "DAI-WETH": { 
    primary: '#BAC7A7',
    secondary: '#a1b189'
  },
  "USDC-WBTC": { 
    primary: '#4A5D23',
    secondary: '#384819'
  },
  "Altcoin-WETH": { 
    primary: '#8B9556',
    secondary: '#737b47'
  }
};

const DEFAULT_COLORS: ColorSet = {
  primary: '#B2AC88',
  secondary: '#8B9556'
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

  const getColorsForCluster = (clusterName: string): ColorSet => {
    const displayName = CLUSTER_DISPLAY_NAMES[clusterName as ClusterName];
    return displayName ? CLUSTER_COLORS[displayName] : DEFAULT_COLORS;
  };

  return (
    <div className="w-full">
      <div className="mb-8">
        <h2 className="text-[#b4d838] text-base md:text-lg text-center px-4">
          Percentage of Blocks with Non-Zero LVR by Category {titleSuffix}
        </h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-6 lg:gap-8">
        {data.map((item) => {
          const colors = getColorsForCluster(item.name);
          const displayName = CLUSTER_DISPLAY_NAMES[item.name as ClusterName] || item.name;
          const percentageValue = item.non_zero_proportion * 100;

          return (
            <div 
              key={item.name}
              className="relative flex flex-col items-center justify-center p-4 md:p-6 lg:p-8 rounded-xl overflow-hidden transition-all duration-300 hover:scale-105 min-h-[200px]"
              style={{
                background: `linear-gradient(135deg, rgba(45, 45, 45, 0.5), rgba(20, 20, 20, 0.8))`,
                border: `1px solid ${colors.primary}20`,
                boxShadow: `0 4px 30px ${colors.primary}10`
              }}
            >
              <div className="relative z-10 flex flex-col items-center w-full">
                <h3 className="text-sm md:text-base lg:text-lg font-medium mb-4 text-center text-white break-words max-w-full px-2">
                  {displayName}
                </h3>
                <p className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl font-bold mb-2 md:mb-4 text-center" 
                   style={{ color: colors.primary }}>
                  {percentageValue.toFixed(2)}
                  <span className="text-xl md:text-2xl lg:text-3xl">%</span>
                </p>
                <div className="text-xs md:text-sm text-gray-400 mt-2 text-center w-full">
                  <div className="flex justify-center items-center space-x-1 break-words px-2">
                    <span className="font-medium whitespace-nowrap" style={{ color: colors.primary }}>
                      {item.non_zero_observations.toLocaleString()}
                    </span>
                    <span>/</span>
                    <span className="font-medium whitespace-nowrap">
                      {item.total_observations.toLocaleString()}
                    </span>
                  </div>
                  <span className="block mt-1 text-gray-500 text-xs">blocks</span>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default CategoryNonZero;