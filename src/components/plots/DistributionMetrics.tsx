import React, { useState, useEffect } from 'react';
import { AlertCircle } from 'lucide-react';

interface DistributionMetricsProps {
  poolAddress: string;
  markoutTime: string;
}

interface MetricData {
  pool_name: string;
  pool_address: string;
  markout_time: string;
  mean: number;
  std_dev: number;
  skewness: number;
  kurtosis: number;
}

const MetricCard: React.FC<{
  title: string;
  value: number;
  description: string;
  isCurrency?: boolean;
}> = ({ title, value, description, isCurrency = false }) => (
  <div className="bg-gradient-to-br from-[#0b0b0e] via-[#1a1a1a] to-[#B2AC88]/10 
                  rounded-lg p-4 border border-[#B2AC88]/20 hover:border-[#b4d838]/30 
                  transition-all duration-300 group relative">
    <div className="flex items-start justify-between mb-2">
      <h3 className="text-[#b4d838] text-sm font-medium">{title}</h3>
      <div className="relative group">
        <AlertCircle className="w-4 h-4 text-[#B2AC88] opacity-50 hover:opacity-100 transition-opacity cursor-help" />
        <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 w-48 p-2 
                      bg-[#161616] border border-[#B2AC88]/20 rounded shadow-lg text-xs text-white 
                      opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none">
          {description}
        </div>
      </div>
    </div>
    <p className="text-white text-2xl font-semibold mb-1">
      {isCurrency && '$'}{value.toFixed(2)}
    </p>
  </div>
);

const DistributionMetrics: React.FC<DistributionMetricsProps> = ({ 
  poolAddress, 
  markoutTime 
}) => {
  const [data, setData] = useState<MetricData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);

        const params = new URLSearchParams({
          pool_address: poolAddress,
          markout_time: markoutTime
        });

        const response = await fetch(
          `https://lvr-wtf-568975696472.us-central1.run.app/metrics?${params.toString()}`
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

    fetchData();
  }, [poolAddress, markoutTime]);

  if (isLoading) {
    return (
      <div className="w-full p-6 bg-black rounded-lg border border-[#212121]">
        <div className="h-32 flex items-center justify-center">
          <p className="text-white text-base">Loading metrics...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full p-6 bg-black rounded-lg border border-[#212121]">
        <div className="h-32 flex items-center justify-center">
          <p className="text-red-500 text-sm px-4 py-2 bg-red-500/10 rounded">
            {error}
          </p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="w-full p-6 bg-black rounded-lg border border-[#212121]">
        <div className="h-32 flex items-center justify-center">
          <p className="text-white text-base">No metrics available</p>
        </div>
      </div>
    );
  }

  const metrics = [
    {
      title: "Mean",
      value: data.mean,
      description: "Average LVR value across all blocks",
      isCurrency: true
    },
    {
      title: "Standard Deviation",
      value: data.std_dev,
      description: "Measure of LVR variability from the mean",
      isCurrency: true
    },
    {
      title: "Skewness",
      value: data.skewness,
      description: "Measure of distribution asymmetry. In this context, the higher the skew, the greater the mean is compared to the median"
    },
    {
      title: "Excess Kurtosis",
      value: data.kurtosis,
      description: "Measure of tail extremity compared to the standard normal distribution (which has an excess kurtosis of 0)"
    }
  ];

  const titleSuffix = markoutTime === 'brontes' ? 
    `for ${data.pool_name} (Observed)` : 
    `for ${data.pool_name} (Markout ${markoutTime}s)`;

  return (
    <div className="w-full p-6 bg-black rounded-lg border border-[#212121]">
      <h2 className="text-[#b4d838] text-lg mb-6 text-center">
        Distribution Metrics of Single-Block LVR {titleSuffix}*
      </h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {metrics.map((metric) => (
          <MetricCard
            key={metric.title}
            title={metric.title}
            value={metric.value}
            description={metric.description}
            isCurrency={metric.isCurrency}
          />
        ))}
      </div>
      <div className="mt-6 text-center">
        <p className="text-xs text-[#B2AC88]/80">
          *We compute central moments using the pairwise update algorithm defined in "Formulas for the Computation of Higher-Order Central Moments" by Pébay et al. 
          The displayed metrics are population-level statistics rather than sample estimates, with the population being all blocks with non-zero simulated/observed LVR since the Merge
        </p>
      </div>
    </div>
    
  );
};

export default DistributionMetrics;