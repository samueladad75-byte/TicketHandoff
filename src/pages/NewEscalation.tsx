import { useState, useEffect, useMemo } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { useForm } from 'react-hook-form';
import TemplateSelector from '../components/TemplateSelector';
import ChecklistUI from '../components/ChecklistUI';
import MarkdownPreview from '../components/MarkdownPreview';
import ReviewModal from '../components/ReviewModal';
import { useEscalations } from '../hooks/useEscalations';
import { useTicketData } from '../hooks/useTicketData';
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

  const { saveEscalation, getEscalation } = useEscalations();
  const { ticket, loading: fetchingTicket, error: ticketError, fetch: fetchTicket } = useTicketData();

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
        llmSummary: null,
        llmConfidence: null,
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
      llmSummary: null,
      llmConfidence: null,
    };

    const id = await saveEscalation(input);
    if (id) {
      setShowReviewModal(false);
      navigate('/history');
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
      llmSummary: null,
      llmConfidence: null,
    };
    // Generate markdown synchronously for preview (will be replaced with actual render)
    return `## Escalation: ${input.ticketId}\n\n### Problem Summary\n${input.problemSummary}\n\n### Troubleshooting Steps\n${checklist.map((item) => `- [${item.checked ? 'x' : ' '}] ${item.text}`).join('\n')}\n\n### Current Status\n${input.currentStatus}\n\n### Next Steps\n${input.nextSteps}`;
  }, [formData, checklist]);

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
          onConfirm={handleSaveDraft}
          onCancel={() => setShowReviewModal(false)}
          onEdit={() => setShowReviewModal(false)}
          onPostSuccess={() => {
            setShowReviewModal(false);
            navigate('/history');
          }}
        />
      )}
    </div>
  );
}
