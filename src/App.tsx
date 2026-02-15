import { BrowserRouter, Routes, Route, Link } from 'react-router';
import { ToastProvider } from './contexts/ToastContext';
import Home from './pages/Home';
import NewEscalation from './pages/NewEscalation';
import History from './pages/History';
import Settings from './pages/Settings';
import './styles/main.css';

function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
      <div className="min-h-screen bg-gray-50">
        <nav className="bg-white shadow-sm border-b">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex space-x-8">
                <Link to="/" className="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-900">
                  Home
                </Link>
                <Link to="/new" className="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-500 hover:text-gray-900">
                  New Escalation
                </Link>
                <Link to="/history" className="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-500 hover:text-gray-900">
                  History
                </Link>
                <Link to="/settings" className="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-500 hover:text-gray-900">
                  Settings
                </Link>
              </div>
            </div>
          </div>
        </nav>
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/new" element={<NewEscalation />} />
            <Route path="/history" element={<History />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
    </ToastProvider>
  );
}

export default App;
