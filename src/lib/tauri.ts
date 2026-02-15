import { invoke } from '@tauri-apps/api/core';
import type {
  Template,
  Escalation,
  EscalationInput,
  EscalationSummary,
  JiraTicket,
  LLMSummaryResult,
  ApiConfig,
  ChecklistItem,
} from '../types';

// Templates
export const listTemplates = () => invoke<Template[]>('list_templates');
export const getTemplate = (id: number) => invoke<Template>('get_template', { id });

// Escalations
export const saveEscalation = (input: EscalationInput) =>
  invoke<number>('save_escalation', { input });
export const getEscalation = (id: number) => invoke<Escalation>('get_escalation', { id });
export const listEscalations = () => invoke<EscalationSummary[]>('list_escalations');
export const deleteEscalation = (id: number) => invoke<void>('delete_escalation', { id });
export const renderMarkdown = (input: EscalationInput) =>
  invoke<string>('render_markdown', { input });
export const postEscalation = (id: number, filePaths: string[]) =>
  invoke<void>('post_escalation', { id, filePaths });
export const retryPostEscalation = (id: number, filePaths: string[]) =>
  invoke<void>('retry_post_escalation', { id, filePaths });

// Tickets
export const fetchJiraTicket = (ticketId: string) =>
  invoke<JiraTicket>('fetch_jira_ticket', { ticketId });
export const postToJira = (ticketId: string, comment: string) =>
  invoke<void>('post_to_jira', { ticketId, comment });
export const attachFilesToJira = (ticketId: string, filePaths: string[]) =>
  invoke<void>('attach_files_to_jira', { ticketId, filePaths });

// LLM
export const summarizeWithLlm = (checklist: ChecklistItem[], problemSummary: string) =>
  invoke<LLMSummaryResult>('summarize_with_llm', { checklist, problemSummary });

// Settings
export const saveApiConfig = (config: ApiConfig) =>
  invoke<void>('save_api_config', { config });
export const getApiConfig = () => invoke<ApiConfig | null>('get_api_config');
export const testJiraConnection = () => invoke<string>('test_jira_connection');
