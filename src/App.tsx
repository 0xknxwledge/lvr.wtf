import React from 'react';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import NavBar from './components/navbar';
import Footer from './components/footer';
import Overview from './pages/overview';
import Aggregate from './pages/aggregate';
import Category from './pages/category';
import Pool from './pages/pool';
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
          <Route path="/aggregate" element={<DataLayout><Aggregate /></DataLayout>} />
          <Route path="/category" element={<DataLayout><Category /></DataLayout>} />
          <Route path="/pool" element={<DataLayout><Pool /></DataLayout>} />
        </Routes>
      </div>
    </Router>
  );
}

export default App;