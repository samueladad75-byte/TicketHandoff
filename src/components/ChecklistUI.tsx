import { useCallback } from 'react';
import type { ChecklistItem } from '../types';

interface ChecklistUIProps {
  items: ChecklistItem[];
  onChange: (items: ChecklistItem[]) => void;
}

export default function ChecklistUI({ items, onChange }: ChecklistUIProps) {
  const toggleCheck = useCallback(
    (index: number) => {
      const newItems = [...items];
      newItems[index].checked = !newItems[index].checked;
      onChange(newItems);
    },
    [items, onChange]
  );

  const updateText = useCallback(
    (index: number, text: string) => {
      const newItems = [...items];
      newItems[index].text = text;
      onChange(newItems);
    },
    [items, onChange]
  );

  const addItem = useCallback(() => {
    onChange([...items, { text: '', checked: false }]);
  }, [items, onChange]);

  const removeItem = useCallback(
    (index: number) => {
      const newItems = items.filter((_, i) => i !== index);
      onChange(newItems);
    },
    [items, onChange]
  );

  const handleKeyPress = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        addItem();
      }
    },
    [addItem]
  );

  const checkedCount = items.filter((item) => item.checked).length;

  return (
    <div>
      <div className="flex justify-between items-center mb-2">
        <label className="block text-sm font-medium text-gray-700">
          Troubleshooting Steps
        </label>
        <span className="text-sm text-gray-500">
          {checkedCount}/{items.length} completed
        </span>
      </div>
      <div className="space-y-2">
        {items.map((item, index) => (
          <div key={index} className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={item.checked}
              onChange={() => toggleCheck(index)}
              className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
            />
            <input
              type="text"
              value={item.text}
              onChange={(e) => updateText(index, e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Troubleshooting step..."
              className="flex-1 px-3 py-1.5 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            />
            <button
              type="button"
              onClick={() => removeItem(index)}
              className="text-red-600 hover:text-red-800 px-2 py-1"
              aria-label="Remove item"
            >
              ×
            </button>
          </div>
        ))}
      </div>
      <button
        type="button"
        onClick={addItem}
        className="mt-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
      >
        + Add step
      </button>
    </div>
  );
}
