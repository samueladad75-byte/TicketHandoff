import { useState, useEffect, useMemo } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { useForm } from 'react-hook-form';
import TemplateSelector from '../components/TemplateSelector';
import ChecklistUI from '../components/ChecklistUI';
import MarkdownPreview from '../components/MarkdownPreview';
import ReviewModal from '../components/ReviewModal';
import ConfidenceBadge from '../components/ConfidenceBadge';
import { useEscalations } from '../hooks/useEscalations';
import { useTicketData } from '../hooks/useTicketData';
import { useLLMSummary } from '../hooks/useLLMSummary';
import { renderMarkdown } from '../lib/tauri';
import type { Template, EscalationInput, ChecklistItem } from '../types';

interface FormData {
  ticketId: string;
  templateId: number | null;
  problemSummary: string;
  currentStatus: string;
  nextSteps: string;
}

export default function NewEscalation() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const escalationId = searchParams.get('id');

  const { register, watch, setValue } = useForm<FormData>({
    defaultValues: {
      ticketId: '',
      templateId: null,
      problemSummary: '',
      currentStatus: '',
      nextSteps: '',
    },
  });

  const [checklist, setChecklist] = useState<ChecklistItem[]>([]);
  const [markdown, setMarkdown] = useState('');
  const [showReviewModal, setShowReviewModal] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [llmSummary, setLlmSummary] = useState<string>('');
  const [llmConfidence, setLlmConfidence] = useState<string>('');
  const [llmConfidenceReason, setLlmConfidenceReason] = useState<string>('');
  const [showLlmSection, setShowLlmSection] = useState(false);
  const [attachedFiles, setAttachedFiles] = useState<Array<{ path: string; name: string; size: number }>>([]);

  const { saveEscalation, getEscalation } = useEscalations();
  const { ticket, loading: fetchingTicket, error: ticketError, fetch: fetchTicket } = useTicketData();
  const { loading: generatingSummary, error: llmError, generate: generateSummary } = useLLMSummary();

  const formData = watch();

  // Load existing escalation if editing
  useEffect(() => {
    if (escalationId) {
      loadEscalation(parseInt(escalationId));
    }
  }, [escalationId]);

  const loadEscalation = async (id: number) => {
    const escalation = await getEscalation(id);
    if (escalation) {
      setValue('ticketId', escalation.ticketId);
      setValue('templateId', escalation.templateId);
      setValue('problemSummary', escalation.problemSummary);
      setValue('currentStatus', escalation.currentStatus);
      setValue('nextSteps', escalation.nextSteps);
      setChecklist(escalation.checklist);
    }
  };

  // Fetch ticket from Jira
  const handleFetchFromJira = async () => {
    const ticketId = formData.ticketId.trim();
    if (!ticketId) return;

    const result = await fetchTicket(ticketId);
    if (result) {
      // Auto-populate problem summary from ticket
      setValue('problemSummary', result.summary);
    }
  };

  // Generate AI summary
  const handleGenerateSummary = async () => {
    if (!formData.problemSummary || checklist.length === 0) {
      alert('Please add a problem summary and at least one checklist item before generating a summary.');
      return;
    }

    const result = await generateSummary(checklist, formData.problemSummary);
    if (result) {
      setLlmSummary(result.summary);
      setLlmConfidence(result.confidence);
      setLlmConfidenceReason(result.confidenceReason);
      setShowLlmSection(true);
    }
  };

  // Attach files
  const handleAttachFiles = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: true,
        title: 'Select files to attach',
      });

      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        const fileInfos = await Promise.all(
          files.map(async (path) => {
            // Get file stats
            const { stat } = await import('@tauri-apps/plugin-fs');
            const stats = await stat(path);
            const name = path.split('/').pop() || path.split('\\').pop() || 'unknown';
            return {
              path,
              name,
              size: stats.size,
            };
          })
        );
        setAttachedFiles([...attachedFiles, ...fileInfos]);
      }
    } catch (error) {
      console.error('Failed to select files:', error);
      alert('Failed to select files: ' + error);
    }
  };

  const handleRemoveFile = (index: number) => {
    setAttachedFiles(attachedFiles.filter((_, i) => i !== index));
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  // Update checklist when template changes
  const handleTemplateChange = (template: Template | null) => {
    setValue('templateId', template?.id || null);
    if (template) {
      // Pre-populate checklist from template
      setChecklist(template.checklistItems.map((item) => ({ ...item, checked: false })));
    } else {
      setChecklist([]);
    }
  };

  // Generate markdown preview
  const generatePreview = async () => {
    setGenerating(true);
    try {
      const input: EscalationInput = {
        ticketId: formData.ticketId,
        templateId: formData.templateId,
        problemSummary: formData.problemSummary,
        checklist,
        currentStatus: formData.currentStatus,
        nextSteps: formData.nextSteps,
        llmSummary: llmSummary || null,
        llmConfidence: llmConfidence || null,
      };
      const result = await renderMarkdown(input);
      setMarkdown(result);
      setShowReviewModal(true);
    } catch (error) {
      console.error('Failed to generate markdown:', error);
      alert('Failed to generate preview: ' + error);
    } finally {
      setGenerating(false);
    }
  };

  const handleSaveDraft = async () => {
    const input: EscalationInput = {
      ticketId: formData.ticketId,
      templateId: formData.templateId,
      problemSummary: formData.problemSummary,
      checklist,
      currentStatus: formData.currentStatus,
      nextSteps: formData.nextSteps,
      llmSummary: llmSummary || null,
      llmConfidence: llmConfidence || null,
    };

    const id = await saveEscalation(input);
    if (id) {
      setShowReviewModal(false);
      navigate('/history');
    }
  };

  const handleSaveAndPost = async (): Promise<number | null> => {
    const input: EscalationInput = {
      ticketId: formData.ticketId,
      templateId: formData.templateId,
      problemSummary: formData.problemSummary,
      checklist,
      currentStatus: formData.currentStatus,
      nextSteps: formData.nextSteps,
      llmSummary: llmSummary || null,
      llmConfidence: llmConfidence || null,
    };

    try {
      const id = await saveEscalation(input);
      return id;
    } catch (error) {
      console.error('Failed to save escalation:', error);
      return null;
    }
  };

  // Real-time markdown preview (debounced)
  const livePreview = useMemo(() => {
    if (!formData.ticketId) return '';
    const input: EscalationInput = {
      ticketId: formData.ticketId,
      templateId: formData.templateId,
      problemSummary: formData.problemSummary,
      checklist,
      currentStatus: formData.currentStatus,
      nextSteps: formData.nextSteps,
      llmSummary: llmSummary || null,
      llmConfidence: llmConfidence || null,
    };
    // Generate markdown synchronously for preview (will be replaced with actual render)
    let preview = `## Escalation: ${input.ticketId}\n\n### Problem Summary\n${input.problemSummary}\n\n### Troubleshooting Steps\n${checklist.map((item) => `- [${item.checked ? 'x' : ' '}] ${item.text}`).join('\n')}\n\n### Current Status\n${input.currentStatus}\n\n### Next Steps\n${input.nextSteps}`;

    if (llmSummary) {
      preview += `\n\n### AI Summary\n${llmSummary}\n(Confidence: ${llmConfidence})`;
    }

    return preview;
  }, [formData, checklist, llmSummary, llmConfidence]);

  return (
    <div className="max-w-7xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">
        {escalationId ? 'Edit Escalation' : 'New Escalation'}
      </h1>

      <div className="grid grid-cols-2 gap-6">
        {/* Left Column - Form */}
        <div className="space-y-6">
          {/* Ticket ID with Fetch button */}
          <div>
            <label htmlFor="ticketId" className="block text-sm font-medium text-gray-700 mb-1">
              Ticket ID *
            </label>
            <div className="flex gap-2">
              <input
                {...register('ticketId', { required: true })}
                type="text"
                id="ticketId"
                placeholder="e.g., SUPPORT-1234"
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <button
                type="button"
                onClick={handleFetchFromJira}
                disabled={fetchingTicket || !formData.ticketId}
                className="px-4 py-2 text-sm font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
              >
                {fetchingTicket ? 'Fetching...' : 'Fetch from Jira'}
              </button>
            </div>
          </div>

          {/* Success message when ticket fetched */}
          {ticket && !ticketError && (
            <div className="p-3 rounded-md bg-green-50 border border-green-200 text-green-800 text-sm">
              <div className="font-medium">✓ Ticket fetched successfully</div>
              {ticket.reporter && (
                <div className="mt-1 text-green-700">
                  Reporter: {ticket.reporter.displayName}
                  {ticket.reporter.email && ` (${ticket.reporter.email})`}
                </div>
              )}
              {ticket.status && (
                <div className="mt-1 text-green-700">Status: {ticket.status}</div>
              )}
            </div>
          )}

          {/* Error message when fetch fails */}
          {ticketError && (
            <div className="p-3 rounded-md bg-red-50 border border-red-200 text-red-800 text-sm">
              <div className="font-medium">Could not fetch ticket: {ticketError}</div>
              <div className="mt-1 text-red-700">
                You can still fill out the form manually.
              </div>
            </div>
          )}

          <TemplateSelector value={formData.templateId} onChange={handleTemplateChange} />

          <div>
            <label htmlFor="problemSummary" className="block text-sm font-medium text-gray-700 mb-1">
              Problem Summary *
            </label>
            <textarea
              {...register('problemSummary', { required: true })}
              id="problemSummary"
              rows={3}
              placeholder="Brief description of the issue..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <ChecklistUI items={checklist} onChange={setChecklist} />

          {/* AI Summary Section */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-gray-700">
                AI Summary (Optional)
              </label>
              <button
                type="button"
                onClick={handleGenerateSummary}
                disabled={generatingSummary || checklist.length === 0 || !formData.problemSummary}
                className="px-3 py-1 text-xs font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {generatingSummary ? 'Generating...' : 'Generate AI Summary'}
              </button>
            </div>

            {llmError && (
              <div className="mb-3 p-3 rounded-md bg-yellow-50 border border-yellow-200 text-yellow-800 text-sm">
                <div className="font-medium">⚠️ LLM unavailable</div>
                <div className="mt-1 text-yellow-700">{llmError}</div>
                <div className="mt-1 text-yellow-700">You can still post without an AI summary.</div>
              </div>
            )}

            {showLlmSection && llmSummary && (
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <ConfidenceBadge
                    level={llmConfidence as 'High' | 'Medium' | 'Low'}
                    reason={llmConfidenceReason}
                  />
                  <span className="text-xs text-gray-500">AI-assisted summary — reviewed by L1 engineer</span>
                </div>
                <textarea
                  value={llmSummary}
                  onChange={(e) => setLlmSummary(e.target.value)}
                  rows={6}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
                  placeholder="AI-generated summary will appear here..."
                />
              </div>
            )}
          </div>

          <div>
            <label htmlFor="currentStatus" className="block text-sm font-medium text-gray-700 mb-1">
              Current Status
            </label>
            <textarea
              {...register('currentStatus')}
              id="currentStatus"
              rows={2}
              placeholder="Current state after troubleshooting..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div>
            <label htmlFor="nextSteps" className="block text-sm font-medium text-gray-700 mb-1">
              Next Steps
            </label>
            <textarea
              {...register('nextSteps')}
              id="nextSteps"
              rows={2}
              placeholder="Recommended actions for L2..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          {/* File Attachments Section */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-gray-700">
                Attachments (Optional)
              </label>
              <button
                type="button"
                onClick={handleAttachFiles}
                className="px-3 py-1 text-xs font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                Attach Files
              </button>
            </div>

            {attachedFiles.length > 0 && (
              <div className="space-y-2">
                {attachedFiles.map((file, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between p-2 bg-gray-50 border border-gray-200 rounded-md"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium text-gray-900 truncate">{file.name}</div>
                      <div className="text-xs text-gray-500">{formatFileSize(file.size)}</div>
                    </div>
                    <button
                      type="button"
                      onClick={() => handleRemoveFile(index)}
                      className="ml-2 text-red-600 hover:text-red-900"
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="flex gap-3">
            <button
              type="button"
              onClick={generatePreview}
              disabled={generating || !formData.ticketId}
              className="flex-1 px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              {generating ? 'Generating...' : 'Preview & Review'}
            </button>
            <button
              type="button"
              onClick={() => navigate('/history')}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Cancel
            </button>
          </div>
        </div>

        {/* Right Column - Live Preview */}
        <div>
          <h2 className="text-sm font-medium text-gray-700 mb-2">Live Preview</h2>
          <MarkdownPreview markdown={livePreview} />
        </div>
      </div>

      {showReviewModal && (
        <ReviewModal
          markdown={markdown}
          ticketId={formData.ticketId}
          attachedFiles={attachedFiles}
          onConfirm={handleSaveDraft}
          onCancel={() => setShowReviewModal(false)}
          onEdit={() => setShowReviewModal(false)}
          onSaveAndPost={handleSaveAndPost}
          onPostSuccess={() => {
            setShowReviewModal(false);
            navigate('/history');
          }}
        />
      )}
    </div>
  );
}
