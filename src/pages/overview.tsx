import React from 'react';
import { Link } from 'react-router-dom';

function Overview() {
  return (
    <div className="py-12">
      <h1 className="text-6xl font-bold mb-12">Overview</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="bg-gradient-to-br from-[#0b0b0e] to-[#4b5c10] rounded-3xl border border-[#b4d838] p-10">
          <h2 className="text-4xl font-semibold mb-8">What is LVR?</h2>
          <hr className="border-[#3a3a3a] mb-8" />
          <p className="text-base">
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam eu turpis molestie, dictum est a, mattis tellus. Sed dignissim, metus nec fringilla accumsan, risus sem sollicitudin lacus, ut interdum tellus elit sed risus. Maecenas eget condimentum velit, sit amet feugiat lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Praesent auctor purus luctus enim egestas, ac scelerisque ante pulvinar.
          </p>
        </div>
        <div className="bg-gradient-to-br from-[#0b0b0e] to-[#70881d] rounded-3xl border border-[#b4d838] p-10">
          <h2 className="text-4xl font-semibold mb-8">Methodology</h2>
          <hr className="border-[#3a3a3a] mb-8" />
          <p className="text-base">
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam eu turpis molestie, dictum est a, mattis tellus. Sed dignissim, metus nec fringilla accumsan, risus sem sollicitudin lacus, ut interdum tellus elit sed risus. Maecenas eget condimentum velit, sit amet feugiat lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Praesent auctor purus luctus enim egestas, ac scelerisque ante pulvinar.
          </p>
        </div>
      </div>
      <div className="mt-12">
        <Link to="/all" className="inline-flex items-center px-6 py-4 rounded-[9.75rem] border border-[#b4d838] text-[#b4d838] text-lg font-medium">
          <span className="mr-2">⟳</span> Access Data Dashboard
        </Link>
      </div>
    </div>
  );
}

export default Overview;