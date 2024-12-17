import React from 'react';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import NavBar from './components/navbar';
import Footer from './components/footer';
import Overview from './pages/overview';
import All from './pages/all';
import Cluster from './pages/cluster';
import Pair from './pages/pair';
import DataLayout from './components/datalayout';

function App() {
  return (
    <Router>
      <div className="App bg-[#030304] min-h-screen text-white flex flex-col">
        <NavBar />
        <Routes>
          <Route path="/" element={
            <>
              <div className="flex-grow">
                <Overview />
              </div>
              <Footer />
            </>
          } />
          <Route path="/all" element={<DataLayout><All /></DataLayout>} />
          <Route path="/cluster" element={<DataLayout><Cluster /></DataLayout>} />
          <Route path="/pair" element={<DataLayout><Pair /></DataLayout>} />
        </Routes>
      </div>
    </Router>
  );
}

export default App;