import { useEffect } from 'react';

interface ToastProps {
  message: string;
  type: 'success' | 'error' | 'warning' | 'info';
  onDismiss: () => void;
}

export default function Toast({ message, type, onDismiss }: ToastProps) {
  useEffect(() => {
    const duration = type === 'error' ? 5000 : 3000;
    const timeout = setTimeout(() => {
      onDismiss();
    }, duration);

    return () => clearTimeout(timeout);
  }, [type, onDismiss]);

  const styles = {
    success: 'bg-green-50 border-green-200 text-green-800',
    error: 'bg-red-50 border-red-200 text-red-800',
    warning: 'bg-yellow-50 border-yellow-200 text-yellow-800',
    info: 'bg-blue-50 border-blue-200 text-blue-800',
  };

  const buttonStyles = {
    success: 'text-green-400 hover:text-green-500',
    error: 'text-red-400 hover:text-red-500',
    warning: 'text-yellow-400 hover:text-yellow-500',
    info: 'text-blue-400 hover:text-blue-500',
  };

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-slide-up">
      <div className={`max-w-sm rounded-lg shadow-lg p-4 border ${styles[type]}`}>
        <div className="flex items-start">
          <div className="flex-1">
            <p className="text-sm font-medium">
              {message}
            </p>
          </div>
          <button
            onClick={onDismiss}
            className={`ml-4 flex-shrink-0 ${buttonStyles[type]}`}
          >
            <span className="sr-only">Close</span>
            <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
