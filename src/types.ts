export interface Template {
  id: number;
  name: string;
  description: string;
  category: string;
  checklistItems: ChecklistItem[];
  l2Team: string | null;
}

export interface ChecklistItem {
  text: string;
  checked: boolean;
}

export interface Escalation {
  id: number;
  ticketId: string;
  templateId: number | null;
  problemSummary: string;
  checklist: ChecklistItem[];
  currentStatus: string;
  nextSteps: string;
  llmSummary: string | null;
  llmConfidence: string | null;
  markdownOutput: string | null;
  status: 'draft' | 'posted' | 'post_failed';
  postedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface EscalationSummary {
  id: number;
  ticketId: string;
  problemSummary: string;
  status: 'draft' | 'posted' | 'post_failed';
  createdAt: string;
}

export interface EscalationInput {
  ticketId: string;
  templateId: number | null;
  problemSummary: string;
  checklist: ChecklistItem[];
  currentStatus: string;
  nextSteps: string;
  llmSummary: string | null;
  llmConfidence: string | null;
}

export interface JiraTicket {
  key: string;
  summary: string;
  description: string | null;
  status: string;
  reporter: { displayName: string; email: string | null } | null;
  assignee: { displayName: string; email: string | null } | null;
  comments: { author: string; body: string; created: string }[];
}

export interface LLMSummaryResult {
  summary: string;
  confidence: string;
  confidenceReason: string;
}

export interface ApiConfig {
  jiraBaseUrl: string;
  jiraEmail: string;
  jiraApiToken: string;
  ollamaEndpoint: string;
  ollamaModel: string;
}
