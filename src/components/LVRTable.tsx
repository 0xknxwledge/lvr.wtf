import React, { useState, useEffect, useCallback } from 'react';
import Plot from 'react-plotly.js';
import { Data } from 'plotly.js';

interface LVRDataItem {
  block_number: number;
  total_lvr: number;
}

interface APIResponse {
  data: LVRDataItem[];
  total_pages: number;
  current_page: number;
  last_queried_block: number;
}

// Custom type for our table trace
interface TableTrace {
  type: 'table';
  header: {
    values: string[][];
    align: string[];
    line: { width: number; color: string };
    fill: { color: string };
    font: { family: string; size: number; color: string };
  };
  cells: {
    values: (string | number)[][];
    align: string[];
    line: { color: string; width: number };
    fill: { color: string[] };
    font: { family: string; size: number; color: string };
  };
}

const LVRTable: React.FC = () => {
  const [lvrData, setLVRData] = useState<LVRDataItem[]>([]);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [totalPages, setTotalPages] = useState<number>(1);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async (page: number) => {
    try {
      setIsLoading(true);
      setError(null);
      console.log(`Attempting to fetch LVR table data for page ${page}...`);
      const response = await fetch(`http://127.0.0.1:5000/lvr_table?page=${page}`);
      if (!response.ok) {
        if (response.status === 404) {
          throw new Error('Page not found');
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data: APIResponse = await response.json();
      console.log('LVR table data fetched successfully:', data);
      setLVRData(data.data);
      setTotalPages(data.total_pages);
      setIsLoading(false);
    } catch (err) {
      console.error('Error fetching LVR data:', err);
      setError(err instanceof Error ? err.message : 'An unexpected error occurred');
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData(currentPage);
  }, [currentPage, fetchData]);

  const handlePageChange = (newPage: number) => {
    if (newPage !== currentPage && newPage >= 1 && newPage <= totalPages) {
      setCurrentPage(newPage);
    }
  };

  if (isLoading) return <div className="text-white">Loading...</div>;
  if (error) return <div className="text-white bg-red-600 p-4 rounded">{error}</div>;

  const tableTrace: TableTrace = {
    type: 'table',
    header: {
      values: [['<b>Block Number</b>'], ['<b>Brontes\' Observed LVR</b>']],
      align: ['left', 'right'],
      line: { width: 1, color: '#212121' },
      fill: { color: '#0f0f13' },
      font: { family: 'Arial', size: 14, color: '#b4d838' }
    },
    cells: {
      values: [
        lvrData.map(item => item.block_number),
        lvrData.map(item => `$${item.total_lvr.toFixed(2)}`)
      ],
      align: ['left', 'right'],
      line: { color: '#212121', width: 1 },
      fill: { color: ['#0f0f13', '#0b0b0e'] },
      font: { family: 'Arial', size: 12, color: '#ffffff' }
    }
  };

  return (
    <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
      <Plot
        data={[tableTrace as Data]}
        layout={{
          autosize: true,
          height: 600,
          margin: { l: 50, r: 50, b: 100, t: 50, pad: 4 },
          paper_bgcolor: '#0f0f13',
          plot_bgcolor: '#0f0f13',
          font: { color: '#ffffff' },
        }}
        config={{
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%', height: '100%' }}
      />
      <div className="flex justify-between items-center mt-4">
        <button 
          onClick={() => handlePageChange(currentPage - 1)} 
          disabled={currentPage === 1 || isLoading}
          className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded disabled:opacity-50"
        >
          Previous
        </button>
        <span className="text-white">Page {currentPage} of {totalPages}</span>
        <button 
          onClick={() => handlePageChange(currentPage + 1)} 
          disabled={currentPage === totalPages || isLoading}
          className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded disabled:opacity-50"
        >
          Next
        </button>
      </div>
    </div>
  );
};

export default LVRTable;