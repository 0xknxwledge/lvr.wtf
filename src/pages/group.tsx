import React from 'react';

function Group() {
  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-bold">Group Data</h1>
        <button className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded">
          Select Markout
        </button>
      </div>

      <div className="space-y-8">
        <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
          <h2 className="text-xl font-semibold mb-4">Single Block LVR</h2>
          {/* Add bar chart component here */}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-4">Group LVR (correlation matrix)</h2>
            {/* Add correlation matrix component here */}
          </div>
          <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-4">LVR for Group (box plot)</h2>
            {/* Add box plot component here */}
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-4">Proportion of total LVR (each pair)</h2>
            {/* Add pie chart component here */}
          </div>
          <div className="bg-[#0f0f13] rounded-2xl border border-[#212121] p-6">
            <h2 className="text-xl font-semibold mb-4">Total LVR</h2>
            <div className="flex justify-end mb-4">
              <button className="px-4 py-2 bg-[#161616] text-white border border-[#b4d838] rounded">
                Daily
              </button>
            </div>
            {/* Add stacked bar chart component here */}
          </div>
        </div>
      </div>
    </div>
  );
}

export default Group;