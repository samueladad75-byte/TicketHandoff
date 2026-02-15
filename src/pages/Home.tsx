import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useEscalations } from '../hooks/useEscalations';
import type { EscalationSummary } from '../types';

export default function Home() {
  const navigate = useNavigate();
  const [recentEscalations, setRecentEscalations] = useState<EscalationSummary[]>([]);
  const { listEscalations } = useEscalations();

  useEffect(() => {
    loadRecent();
  }, []);

  const loadRecent = async () => {
    const all = await listEscalations();
    setRecentEscalations(all.slice(0, 5));
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="text-center py-12">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">Ticket Handoff Assistant</h1>
        <p className="text-lg text-gray-600 mb-8">
          Create structured escalations in minutes, not hours.
        </p>
        <button
          onClick={() => navigate('/new')}
          className="inline-flex items-center px-6 py-3 text-base font-medium text-white bg-blue-600 border border-transparent rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          New Escalation
        </button>
      </div>

      {recentEscalations.length > 0 && (
        <div className="mt-12">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-semibold text-gray-900">Recent Escalations</h2>
            <button
              onClick={() => navigate('/history')}
              className="text-sm text-blue-600 hover:text-blue-800 font-medium"
            >
              View all →
            </button>
          </div>
          <div className="bg-white shadow-sm rounded-lg border border-gray-200 divide-y divide-gray-200">
            {recentEscalations.map((escalation) => (
              <div
                key={escalation.id}
                onClick={() => navigate(`/new?id=${escalation.id}`)}
                className="px-6 py-4 hover:bg-gray-50 cursor-pointer"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-gray-900">{escalation.ticketId}</span>
                      <span
                        className={`px-2 py-0.5 text-xs font-medium rounded-full ${
                          escalation.status === 'posted'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-gray-100 text-gray-800'
                        }`}
                      >
                        {escalation.status === 'draft' ? 'Draft' : 'Posted'}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 line-clamp-2">
                      {escalation.problemSummary}
                    </p>
                  </div>
                  <div className="text-sm text-gray-500 ml-4">
                    {formatDate(escalation.createdAt)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
