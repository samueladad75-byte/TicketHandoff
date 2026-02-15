import { useState } from 'react';
import { summarizeWithLlm } from '../lib/tauri';
import type { ChecklistItem, LLMSummaryResult } from '../types';

export function useLLMSummary() {
  const [summary, setSummary] = useState<LLMSummaryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generate = async (
    checklist: ChecklistItem[],
    problemSummary: string
  ): Promise<LLMSummaryResult | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await summarizeWithLlm(checklist, problemSummary);
      setSummary(result);
      return result;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      setSummary(null);
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { summary, loading, error, generate };
}
