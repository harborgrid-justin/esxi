import { useState } from 'react';
import { Play, Download, Trash2, Clock, CheckCircle, XCircle, Loader2 } from 'lucide-react';
import * as Select from '@radix-ui/react-select';
import * as Tabs from '@radix-ui/react-tabs';
import type { AnalysisParams, AnalysisResult } from '@/types';

const analysisTypes = [
  { value: 'buffer', label: 'Buffer' },
  { value: 'intersect', label: 'Intersect' },
  { value: 'union', label: 'Union' },
  { value: 'difference', label: 'Difference' },
  { value: 'clip', label: 'Clip' },
];

const unitOptions = [
  { value: 'meters', label: 'Meters' },
  { value: 'kilometers', label: 'Kilometers' },
  { value: 'miles', label: 'Miles' },
  { value: 'feet', label: 'Feet' },
];

export function AnalysisPanel() {
  const [activeTab, setActiveTab] = useState<'new' | 'history'>('new');
  const [analysisParams, setAnalysisParams] = useState<Partial<AnalysisParams>>({
    type: 'buffer',
    units: 'meters',
    inputLayers: [],
  });
  const [analysisHistory, setAnalysisHistory] = useState<AnalysisResult[]>([]);

  const handleRunAnalysis = () => {
    console.log('Running analysis:', analysisParams);

    // Mock analysis result
    const result: AnalysisResult = {
      id: `analysis-${Date.now()}`,
      type: analysisParams.type || 'buffer',
      status: 'running',
      createdAt: new Date().toISOString(),
    };

    setAnalysisHistory((prev) => [result, ...prev]);
    setActiveTab('history');

    // Simulate completion
    setTimeout(() => {
      setAnalysisHistory((prev) =>
        prev.map((item) =>
          item.id === result.id
            ? { ...item, status: 'completed', completedAt: new Date().toISOString() }
            : item
        )
      );
    }, 2000);
  };

  const renderAnalysisForm = () => {
    return (
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Analysis Type
          </label>
          <Select.Root
            value={analysisParams.type}
            onValueChange={(value) =>
              setAnalysisParams((prev) => ({ ...prev, type: value as AnalysisParams['type'] }))
            }
          >
            <Select.Trigger className="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500">
              <Select.Value />
            </Select.Trigger>
            <Select.Portal>
              <Select.Content className="bg-white rounded-lg shadow-xl border border-gray-200 overflow-hidden z-50">
                <Select.Viewport>
                  {analysisTypes.map((type) => (
                    <Select.Item
                      key={type.value}
                      value={type.value}
                      className="px-3 py-2 text-sm cursor-pointer hover:bg-blue-50 focus:bg-blue-50 outline-none"
                    >
                      <Select.ItemText>{type.label}</Select.ItemText>
                    </Select.Item>
                  ))}
                </Select.Viewport>
              </Select.Content>
            </Select.Portal>
          </Select.Root>
        </div>

        {analysisParams.type === 'buffer' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Distance
              </label>
              <input
                type="number"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter distance"
                value={analysisParams.distance || ''}
                onChange={(e) =>
                  setAnalysisParams((prev) => ({
                    ...prev,
                    distance: parseFloat(e.target.value),
                  }))
                }
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Units
              </label>
              <Select.Root
                value={analysisParams.units}
                onValueChange={(value) =>
                  setAnalysisParams((prev) => ({
                    ...prev,
                    units: value as AnalysisParams['units'],
                  }))
                }
              >
                <Select.Trigger className="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500">
                  <Select.Value />
                </Select.Trigger>
                <Select.Portal>
                  <Select.Content className="bg-white rounded-lg shadow-xl border border-gray-200 overflow-hidden z-50">
                    <Select.Viewport>
                      {unitOptions.map((unit) => (
                        <Select.Item
                          key={unit.value}
                          value={unit.value}
                          className="px-3 py-2 text-sm cursor-pointer hover:bg-blue-50 focus:bg-blue-50 outline-none"
                        >
                          <Select.ItemText>{unit.label}</Select.ItemText>
                        </Select.Item>
                      ))}
                    </Select.Viewport>
                  </Select.Content>
                </Select.Portal>
              </Select.Root>
            </div>
          </>
        )}

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Output Layer Name
          </label>
          <input
            type="text"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Enter output name"
            value={analysisParams.outputName || ''}
            onChange={(e) =>
              setAnalysisParams((prev) => ({ ...prev, outputName: e.target.value }))
            }
          />
        </div>

        <button
          onClick={handleRunAnalysis}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center justify-center gap-2"
        >
          <Play className="w-4 h-4" />
          Run Analysis
        </button>
      </div>
    );
  };

  const renderAnalysisHistory = () => {
    if (analysisHistory.length === 0) {
      return (
        <div className="text-center py-12">
          <Clock className="w-12 h-12 mx-auto text-gray-400 mb-4" />
          <p className="text-gray-600">No analysis history</p>
        </div>
      );
    }

    return (
      <div className="space-y-2">
        {analysisHistory.map((result) => (
          <div
            key={result.id}
            className="bg-white border border-gray-200 rounded-lg p-3 hover:border-blue-300 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  {result.status === 'completed' && (
                    <CheckCircle className="w-4 h-4 text-green-600" />
                  )}
                  {result.status === 'running' && (
                    <Loader2 className="w-4 h-4 text-blue-600 animate-spin" />
                  )}
                  {result.status === 'failed' && (
                    <XCircle className="w-4 h-4 text-red-600" />
                  )}
                  <h4 className="text-sm font-medium text-gray-900">
                    {result.type.charAt(0).toUpperCase() + result.type.slice(1)} Analysis
                  </h4>
                </div>
                <p className="text-xs text-gray-500">
                  Started: {new Date(result.createdAt).toLocaleString()}
                </p>
                {result.completedAt && (
                  <p className="text-xs text-gray-500">
                    Completed: {new Date(result.completedAt).toLocaleString()}
                  </p>
                )}
                {result.error && (
                  <p className="text-xs text-red-600 mt-1">{result.error}</p>
                )}
              </div>

              <div className="flex gap-1">
                {result.status === 'completed' && (
                  <button
                    className="p-1 hover:bg-blue-100 rounded"
                    title="Download results"
                  >
                    <Download className="w-4 h-4 text-blue-600" />
                  </button>
                )}
                <button
                  onClick={() =>
                    setAnalysisHistory((prev) => prev.filter((item) => item.id !== result.id))
                  }
                  className="p-1 hover:bg-red-100 rounded"
                  title="Delete"
                >
                  <Trash2 className="w-4 h-4 text-red-600" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-gray-900">Spatial Analysis</h3>

      <Tabs.Root value={activeTab} onValueChange={(value) => setActiveTab(value as 'new' | 'history')}>
        <Tabs.List className="flex bg-gray-100 rounded-lg p-1">
          <Tabs.Trigger
            value="new"
            className="flex-1 px-3 py-1.5 text-sm font-medium rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm transition-colors"
          >
            New Analysis
          </Tabs.Trigger>
          <Tabs.Trigger
            value="history"
            className="flex-1 px-3 py-1.5 text-sm font-medium rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm transition-colors"
          >
            History
          </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="new" className="mt-4">
          {renderAnalysisForm()}
        </Tabs.Content>

        <Tabs.Content value="history" className="mt-4">
          {renderAnalysisHistory()}
        </Tabs.Content>
      </Tabs.Root>
    </div>
  );
}
