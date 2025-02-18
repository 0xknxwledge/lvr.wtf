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

// Updated color palette to match site theme
const CLUSTER_COLORS: Record<DisplayName, ColorSet> = {
  "Stable Pairs": { 
    primary: '#F651AE',
    secondary: '#D33D97'
  },
  "WBTC-WETH": { 
    primary: '#8247E5',
    secondary: '#6A35CC'
  },
  "USDC-WETH": { 
    primary: '#BA8EF7',
    secondary: '#A276E5'
  },
  "USDT-WETH": { 
    primary: '#30283A',
    secondary: '#1E1825'
  },
  "DAI-WETH": { 
    primary: '#FF84C9',
    secondary: '#E66DB2'
  },
  "USDC-WBTC": { 
    primary: '#644AA0',
    secondary: '#4C3587'
  },
  "Altcoin-WETH": { 
    primary: '#9B6FE8',
    secondary: '#835AD0'
  }
};

const DEFAULT_COLORS: ColorSet = {
  primary: '#F651AE',
  secondary: '#8247E5'
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
          <p className="text-white font-['Geist']">Loading...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full">
        <div className="flex items-center justify-center h-48">
          <p className="text-red-500 font-['Geist']">{error}</p>
        </div>
      </div>
    );
  }

  const titleSuffix = selectedMarkout === 'brontes' ? 
    '(Observed)' : 
    `(Markout ${selectedMarkout}s)`;

  const getColorsForCluster = (clusterName: string): ColorSet => {
    const displayName = CLUSTER_DISPLAY_NAMES[clusterName as ClusterName];
    return displayName ? CLUSTER_COLORS[displayName] : DEFAULT_COLORS;
  };

  return (
    <div className="w-full">
      <div className="mb-8">
        <h2 className="text-[#FFFFFF] text-base md:text-lg text-center px-4 font-['Geist']">
          Percentage of Blocks with Non-Zero LVR by Category {titleSuffix}
        </h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-6 lg:gap-8 p-4">
        {data.map((item) => {
          const colors = getColorsForCluster(item.name);
          const displayName = CLUSTER_DISPLAY_NAMES[item.name as ClusterName] || item.name;
          const percentageValue = item.non_zero_proportion * 100;

          return (
            <div key={item.name} className="p-4">
              <div 
                className="relative flex flex-col items-center justify-center p-4 md:p-6 lg:p-8 rounded-xl overflow-hidden transition-all duration-300 hover:scale-105 min-h-[200px] transform-gpu"
                style={{
                  background: `linear-gradient(135deg, rgba(48, 40, 58, 0.5), rgba(3, 3, 4, 0.8))`,
                  border: `1px solid ${colors.primary}20`,
                  boxShadow: `0 4px 30px ${colors.primary}10`
                }}
              >
                <div 
                  className="absolute inset-0 opacity-10 transition-opacity duration-300 hover:opacity-20"
                  style={{
                    background: `linear-gradient(135deg, ${colors.primary}, ${colors.secondary})`
                  }}
                />
                
                <div className="relative z-10 flex flex-col items-center w-full">
                  <div className="w-full mb-4">
                    <h3 className="text-sm md:text-base lg:text-lg font-medium text-center text-white break-words">
                      {displayName}
                    </h3>
                  </div>

                  <div className="flex items-baseline justify-center mb-2 md:mb-4">
                    <span 
                      className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl font-bold"
                      style={{ color: colors.primary }}
                    >
                      {percentageValue.toFixed(2)}
                    </span>
                    <span 
                      className="text-xl md:text-2xl lg:text-3xl ml-1"
                      style={{ color: colors.primary }}
                    >
                      %
                    </span>
                  </div>

                  <div className="flex flex-col items-center text-xs md:text-sm text-gray-400 mt-2">
                    <div className="flex items-center justify-center space-x-1">
                      <span 
                        className="font-medium"
                        style={{ color: colors.primary }}
                      >
                        {item.non_zero_observations.toLocaleString()}
                      </span>
                      <span>/</span>
                      <span className="font-medium">
                        {item.total_observations.toLocaleString()}
                      </span>
                    </div>
                    <span className="text-gray-500 text-xs mt-1">
                      blocks
                    </span>
                  </div>
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