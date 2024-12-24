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
      <div className="mb-12">
        <h2 className="text-white text-2xl text-center font-medium tracking-wide">
          Percentage of Blocks with Non-Zero LVR by Category {titleSuffix}
        </h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8">
        {data.map((item) => {
          const colors = getColorsForCluster(item.name);
          const displayName = CLUSTER_DISPLAY_NAMES[item.name as ClusterName] || item.name;
          
          return (
            <div 
              key={item.name}
              className="relative flex flex-col items-center justify-center p-8 rounded-xl overflow-hidden transition-all duration-300 hover:scale-105"
              style={{
                background: `linear-gradient(135deg, rgba(45, 45, 45, 0.5), rgba(20, 20, 20, 0.8))`,
                border: `1px solid ${colors.primary}20`,
                boxShadow: `0 4px 30px ${colors.primary}10`
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
              <div className="relative z-10 flex flex-col items-center">
                <h3 className="text-lg font-medium mb-6 text-center text-white">
                  {displayName}
                </h3>
                <p className="text-6xl font-bold mb-4" style={{ color: colors.primary }}>
                  {(item.non_zero_proportion * 100).toFixed(2)}
                  <span className="text-3xl">%</span>
                </p>
                <div className="text-sm text-gray-400 mt-2 text-center">
                  <span className="font-medium" style={{ color: colors.primary }}>
                    {item.non_zero_observations.toLocaleString()}
                  </span>
                  <span className="mx-2">/</span>
                  <span className="font-medium">
                    {item.total_observations.toLocaleString()}
                  </span>
                  <br />
                  <span className="text-gray-500">blocks</span>
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