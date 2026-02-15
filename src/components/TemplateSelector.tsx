import { useTemplates } from '../hooks/useTemplates';
import type { Template } from '../types';

interface TemplateSelectorProps {
  value: number | null;
  onChange: (template: Template | null) => void;
}

export default function TemplateSelector({ value, onChange }: TemplateSelectorProps) {
  const { templates, loading, error } = useTemplates();

  if (loading) {
    return <div className="text-sm text-gray-500">Loading templates...</div>;
  }

  if (error) {
    return <div className="text-sm text-red-600">Error loading templates: {error}</div>;
  }

  // Group templates by category
  const groupedTemplates = templates.reduce((acc, template) => {
    if (!acc[template.category]) {
      acc[template.category] = [];
    }
    acc[template.category].push(template);
    return acc;
  }, {} as Record<string, Template[]>);

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedId = e.target.value ? parseInt(e.target.value) : null;
    const selectedTemplate = templates.find((t) => t.id === selectedId) || null;
    onChange(selectedTemplate);
  };

  return (
    <div>
      <label htmlFor="template" className="block text-sm font-medium text-gray-700 mb-1">
        Template
      </label>
      <select
        id="template"
        value={value || ''}
        onChange={handleChange}
        className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
      >
        <option value="">Select a template...</option>
        {Object.entries(groupedTemplates).map(([category, categoryTemplates]) => (
          <optgroup key={category} label={category}>
            {categoryTemplates.map((template) => (
              <option key={template.id} value={template.id}>
                {template.name}
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      {value && (
        <p className="mt-1 text-sm text-gray-500">
          {templates.find((t) => t.id === value)?.description}
        </p>
      )}
    </div>
  );
}
