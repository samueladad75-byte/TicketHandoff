import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { postEscalation } from '../lib/tauri';

interface ReviewModalProps {
  markdown: string;
  ticketId: string;
  attachedFiles?: Array<{ path: string; name: string; size: number }>;
  onConfirm: () => void;
  onCancel: () => void;
  onEdit: () => void;
  onPostSuccess?: () => void;
  onSaveAndPost?: () => Promise<number | null>; // Save escalation and return ID
}

export default function ReviewModal({
  markdown,
  ticketId,
  attachedFiles = [],
  onConfirm,
  onCancel,
  onEdit,
  onPostSuccess,
  onSaveAndPost,
}: ReviewModalProps) {
  const [posting, setPosting] = useState(false);
  const [postError, setPostError] = useState<string | null>(null);
  const [postSuccess, setPostSuccess] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<string>('');

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  const handlePostToJira = async () => {
    if (!onSaveAndPost) {
      setPostError('Save callback not provided');
      return;
    }

    setPosting(true);
    setPostError(null);
    setUploadProgress('');
    try {
      // Step 1: Save escalation and get ID
      setUploadProgress('Saving escalation...');
      const escalationId = await onSaveAndPost();
      if (!escalationId) {
        throw new Error('Failed to save escalation');
      }

      // Step 2: Post to Jira (comment + attachments)
      setUploadProgress('Posting to Jira...');
      const filePaths = attachedFiles.map(f => f.path);
      await postEscalation(escalationId, filePaths);

      setPostSuccess(true);
      setUploadProgress('');
      if (onPostSuccess) {
        // Give user time to see success message
        setTimeout(() => {
          onPostSuccess();
        }, 1500);
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      setPostError(errorMsg);
      setUploadProgress('');
    } finally {
      setPosting(false);
    }
  };
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">Review Escalation</h2>
        </div>

        <div className="flex-1 overflow-y-auto px-6 py-4">
          {/* Upload progress */}
          {uploadProgress && (
            <div className="mb-4 p-3 rounded-md bg-blue-50 border border-blue-200 text-blue-800 text-sm">
              <div className="font-medium">{uploadProgress}</div>
            </div>
          )}

          {/* Success message */}
          {postSuccess && (
            <div className="mb-4 p-3 rounded-md bg-green-50 border border-green-200 text-green-800 text-sm">
              <div className="font-medium">✓ Comment posted to {ticketId} successfully!</div>
              {attachedFiles.length > 0 && (
                <div className="mt-1">✓ {attachedFiles.length} file(s) attached</div>
              )}
            </div>
          )}

          {/* Error message */}
          {postError && (
            <div className="mb-4 p-3 rounded-md bg-red-50 border border-red-200 text-red-800 text-sm">
              <div className="font-medium">Failed to post: {postError}</div>
              <button
                onClick={handlePostToJira}
                disabled={posting}
                className="mt-2 text-sm text-red-600 hover:text-red-800 underline"
              >
                Retry
              </button>
            </div>
          )}

          {/* Attached files preview */}
          {attachedFiles.length > 0 && (
            <div className="mb-4 p-3 rounded-md bg-gray-50 border border-gray-200">
              <div className="text-sm font-medium text-gray-700 mb-2">
                Attachments ({attachedFiles.length})
              </div>
              <div className="space-y-1">
                {attachedFiles.map((file, index) => (
                  <div key={index} className="text-sm text-gray-600">
                    • {file.name} ({formatFileSize(file.size)})
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="prose prose-sm max-w-none">
            <ReactMarkdown>{markdown}</ReactMarkdown>
          </div>
        </div>

        <div className="px-6 py-4 border-t border-gray-200 bg-gray-50 flex justify-between">
          <button
            onClick={onEdit}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Edit
          </button>
          <div className="flex gap-3">
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              disabled={posting}
            >
              Cancel
            </button>
            <button
              onClick={onConfirm}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              disabled={posting || postSuccess}
            >
              Save as Draft
            </button>
            <button
              onClick={handlePostToJira}
              disabled={posting || postSuccess}
              className="px-4 py-2 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              {posting ? 'Posting...' : postSuccess ? 'Posted ✓' : 'Post to Ticket'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
