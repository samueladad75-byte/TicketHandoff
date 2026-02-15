import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useEscalations } from '../hooks/useEscalations';
import { getApiConfig } from '../lib/tauri';
import type { EscalationSummary } from '../types';

export default function History() {
  const navigate = useNavigate();
  const [escalations, setEscalations] = useState<EscalationSummary[]>([]);
  const [jiraBaseUrl, setJiraBaseUrl] = useState<string>('');
  const { listEscalations, deleteEscalation, loading } = useEscalations();

  useEffect(() => {
    loadEscalations();
    loadConfig();
  }, []);

  const loadEscalations = async () => {
    const result = await listEscalations();
    setEscalations(result);
  };

  const loadConfig = async () => {
    try {
      const config = await getApiConfig();
      if (config) {
        setJiraBaseUrl(config.jiraBaseUrl);
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this escalation?')) {
      const success = await deleteEscalation(id);
      if (success) {
        loadEscalations();
      }
    }
  };

  const getStatusBadge = (status: string) => {
    const styles = {
      draft: 'bg-gray-100 text-gray-800',
      posted: 'bg-green-100 text-green-800',
      post_failed: 'bg-red-100 text-red-800',
    };
    const labels = {
      draft: 'Draft',
      posted: 'Posted',
      post_failed: 'Post Failed',
    };
    return (
      <span
        className={`px-2 py-1 text-xs font-medium rounded-full ${styles[status as keyof typeof styles] || styles.draft}`}
      >
        {labels[status as keyof typeof labels] || status}
      </span>
    );
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (loading && escalations.length === 0) {
    return <div className="text-center py-12">Loading...</div>;
  }

  if (escalations.length === 0) {
    return (
      <div className="text-center py-12">
        <h2 className="text-xl font-semibold text-gray-900 mb-2">No escalations yet</h2>
        <p className="text-gray-600 mb-6">Create your first escalation to get started.</p>
        <button
          onClick={() => navigate('/new')}
          className="inline-flex items-center px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          New Escalation
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Escalation History</h1>
        <button
          onClick={() => navigate('/new')}
          className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          New Escalation
        </button>
      </div>

      <div className="bg-white shadow-sm rounded-lg border border-gray-200 overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Ticket ID
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Problem
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Created
              </th>
              <th
                scope="col"
                className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {escalations.map((escalation) => (
              <tr key={escalation.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                  {escalation.ticketId}
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  <div className="max-w-md truncate">{escalation.problemSummary}</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  {getStatusBadge(escalation.status as unknown as string)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {formatDate(escalation.createdAt)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <div className="flex justify-end gap-3">
                    {escalation.status === 'posted' && jiraBaseUrl && (
                      <a
                        href={`${jiraBaseUrl}/browse/${escalation.ticketId}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-green-600 hover:text-green-900"
                      >
                        View on Jira
                      </a>
                    )}
                    {escalation.status === 'post_failed' && (
                      <button
                        onClick={() => navigate(`/new?id=${escalation.id}`)}
                        className="text-orange-600 hover:text-orange-900"
                      >
                        Retry
                      </button>
                    )}
                    <button
                      onClick={() => navigate(`/new?id=${escalation.id}`)}
                      className="text-blue-600 hover:text-blue-900"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDelete(escalation.id)}
                      className="text-red-600 hover:text-red-900"
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
