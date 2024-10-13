import React from 'react';

function Pair() {
  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-bold">All Pairs</h1>
        <button className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded">
          Select Markout
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">LVR for All Pairs</h2>
          {/* Add chart component here */}
        </div>
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">Proportion of total LVR (each pair)</h2>
          {/* Add pie chart component here */}
        </div>
      </div>

      <div className="mb-8">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-4xl font-semibold">Individual Pairs</h2>
          <div className="space-x-4">
            <button className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded">
              Select Pair
            </button>
            <button className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded">
              Select Markout
            </button>
          </div>
        </div>
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h3 className="text-xl font-semibold mb-4">Single Block LVR</h3>
          {/* Add bar chart component here */}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h3 className="text-xl font-semibold mb-4">LVR for Pair</h3>
          {/* Add box plot component here */}
        </div>
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h3 className="text-xl font-semibold mb-4">Max LVR</h3>
          <div className="space-y-4">
            <div>
              <p className="text-[#b4d838]">Block Number</p>
              <p className="text-4xl font-semibold">20,592,430</p>
            </div>
            <hr className="border-[#333333]" />
            <div>
              <p className="text-[#b4d838]">Amount</p>
              <p className="text-4xl font-semibold">$12.5M</p>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6 mb-8">
        <h3 className="text-xl font-semibold mb-4">Correlation of Median LVR Between Pairs</h3>
        {/* Add correlation matrix component here */}
      </div>

      <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
        <h3 className="text-xl font-semibold mb-4">Total LVR (across 22 pairs)</h3>
        {/* Add line chart component here */}
      </div>
    </div>
  );
}

export default Pair;