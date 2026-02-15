import { useState } from 'react';
import { fetchJiraTicket } from '../lib/tauri';
import type { JiraTicket } from '../types';

export function useTicketData() {
  const [ticket, setTicket] = useState<JiraTicket | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetch = async (ticketId: string): Promise<JiraTicket | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await fetchJiraTicket(ticketId);
      setTicket(result);
      return result;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      setTicket(null);
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { ticket, loading, error, fetch };
}
