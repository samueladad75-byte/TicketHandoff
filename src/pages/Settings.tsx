import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { getApiConfig, saveApiConfig, testJiraConnection } from '../lib/tauri';
import type { ApiConfig } from '../types';

interface SettingsForm {
  jiraBaseUrl: string;
  jiraEmail: string;
  jiraApiToken: string;
  ollamaEndpoint: string;
  ollamaModel: string;
}

export default function Settings() {
  const { register, handleSubmit, setValue, watch } = useForm<SettingsForm>({
    defaultValues: {
      jiraBaseUrl: '',
      jiraEmail: '',
      jiraApiToken: '',
      ollamaEndpoint: 'http://localhost:11434',
      ollamaModel: 'llama3',
    },
  });

  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const config = await getApiConfig();
      if (config) {
        setValue('jiraBaseUrl', config.jiraBaseUrl);
        setValue('jiraEmail', config.jiraEmail);
        // Don't set jiraApiToken - it's masked on server
        setValue('ollamaEndpoint', config.ollamaEndpoint);
        setValue('ollamaModel', config.ollamaModel);
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const onSubmit = async (data: SettingsForm) => {
    setLoading(true);
    setMessage(null);
    try {
      const config: ApiConfig = {
        jiraBaseUrl: data.jiraBaseUrl,
        jiraEmail: data.jiraEmail,
        jiraApiToken: data.jiraApiToken,
        ollamaEndpoint: data.ollamaEndpoint,
        ollamaModel: data.ollamaModel,
      };
      await saveApiConfig(config);
      setMessage({ type: 'success', text: 'Settings saved successfully' });
    } catch (error) {
      setMessage({ type: 'error', text: String(error) });
    } finally {
      setLoading(false);
    }
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testJiraConnection();
      setTestResult(result);
      setMessage({ type: 'success', text: 'Connection successful!' });
    } catch (error) {
      setTestResult(null);
      setMessage({ type: 'error', text: `Connection failed: ${error}` });
    } finally {
      setTesting(false);
    }
  };

  const formData = watch();
  const hasJiraConfig = formData.jiraBaseUrl && formData.jiraEmail;

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Settings</h1>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Jira Configuration */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Jira Configuration</h2>

          <div className="space-y-4">
            <div>
              <label htmlFor="jiraBaseUrl" className="block text-sm font-medium text-gray-700 mb-1">
                Jira Base URL *
              </label>
              <input
                {...register('jiraBaseUrl', { required: true })}
                type="url"
                id="jiraBaseUrl"
                placeholder="https://your-company.atlassian.net"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Your Jira Cloud instance URL (without trailing slash)
              </p>
            </div>

            <div>
              <label htmlFor="jiraEmail" className="block text-sm font-medium text-gray-700 mb-1">
                Email *
              </label>
              <input
                {...register('jiraEmail', { required: true })}
                type="email"
                id="jiraEmail"
                placeholder="your-email@company.com"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Your Jira account email
              </p>
            </div>

            <div>
              <label htmlFor="jiraApiToken" className="block text-sm font-medium text-gray-700 mb-1">
                API Token *
              </label>
              <input
                {...register('jiraApiToken', { required: true })}
                type="password"
                id="jiraApiToken"
                placeholder="Enter your Jira API token"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Generate at{' '}
                <a
                  href="https://id.atlassian.com/manage-profile/security/api-tokens"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:text-blue-800"
                >
                  https://id.atlassian.com/manage-profile/security/api-tokens
                </a>
              </p>
            </div>

            {hasJiraConfig && (
              <div>
                <button
                  type="button"
                  onClick={handleTestConnection}
                  disabled={testing}
                  className="px-4 py-2 text-sm font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-md hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {testing ? 'Testing...' : 'Test Connection'}
                </button>
                {testResult && (
                  <p className="mt-2 text-sm text-green-700">{testResult}</p>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Ollama Configuration */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Ollama Configuration</h2>

          <div className="space-y-4">
            <div>
              <label htmlFor="ollamaEndpoint" className="block text-sm font-medium text-gray-700 mb-1">
                Ollama Endpoint
              </label>
              <input
                {...register('ollamaEndpoint')}
                type="url"
                id="ollamaEndpoint"
                placeholder="http://localhost:11434"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Local Ollama server endpoint (Phase 3 feature)
              </p>
            </div>

            <div>
              <label htmlFor="ollamaModel" className="block text-sm font-medium text-gray-700 mb-1">
                Model Name
              </label>
              <input
                {...register('ollamaModel')}
                type="text"
                id="ollamaModel"
                placeholder="llama3"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Model to use for LLM summarization (must be already pulled in Ollama)
              </p>
            </div>
          </div>
        </div>

        {/* Status Message */}
        {message && (
          <div
            className={`p-4 rounded-md ${
              message.type === 'success'
                ? 'bg-green-50 border border-green-200 text-green-800'
                : 'bg-red-50 border border-red-200 text-red-800'
            }`}
          >
            {message.text}
          </div>
        )}

        {/* Save Button */}
        <div className="flex justify-end">
          <button
            type="submit"
            disabled={loading}
            className="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            {loading ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </form>
    </div>
  );
}
