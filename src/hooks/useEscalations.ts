import { useState } from 'react';
import {
  saveEscalation as saveTauri,
  getEscalation as getTauri,
  listEscalations as listTauri,
  deleteEscalation as deleteTauri,
} from '../lib/tauri';
import type { Escalation, EscalationInput, EscalationSummary } from '../types';

export function useEscalations() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const saveEscalation = async (input: EscalationInput): Promise<number | null> => {
    try {
      setLoading(true);
      setError(null);
      const id = await saveTauri(input);
      return id;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    } finally {
      setLoading(false);
    }
  };

  const getEscalation = async (id: number): Promise<Escalation | null> => {
    try {
      setLoading(true);
      setError(null);
      return await getTauri(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    } finally {
      setLoading(false);
    }
  };

  const listEscalations = async (): Promise<EscalationSummary[]> => {
    try {
      setLoading(true);
      setError(null);
      return await listTauri();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return [];
    } finally {
      setLoading(false);
    }
  };

  const deleteEscalation = async (id: number): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await deleteTauri(id);
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    } finally {
      setLoading(false);
    }
  };

  return {
    saveEscalation,
    getEscalation,
    listEscalations,
    deleteEscalation,
    loading,
    error,
  };
}
