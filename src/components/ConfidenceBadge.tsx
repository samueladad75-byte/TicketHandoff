interface ConfidenceBadgeProps {
  level: 'High' | 'Medium' | 'Low';
  reason: string;
}

export default function ConfidenceBadge({ level, reason }: ConfidenceBadgeProps) {
  const styles = {
    High: 'bg-green-100 text-green-800 border-green-200',
    Medium: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    Low: 'bg-red-100 text-red-800 border-red-200',
  };

  return (
    <div className="inline-flex items-center gap-2">
      <span
        className={`px-2 py-1 text-xs font-medium rounded border ${styles[level]}`}
        title={reason}
      >
        Confidence: {level}
      </span>
      <span className="text-xs text-gray-500" title={reason}>
        ℹ️
      </span>
    </div>
  );
}
