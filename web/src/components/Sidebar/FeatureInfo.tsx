import { FileQuestion, Copy, Check } from 'lucide-react';
import { useState } from 'react';
import { useMapStore } from '@/stores/mapStore';

export function FeatureInfo() {
  const { selectedFeatures } = useMapStore();
  const [copiedField, setCopiedField] = useState<string | null>(null);

  const copyToClipboard = (text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  if (selectedFeatures.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="text-gray-400 mb-4">
          <FileQuestion className="w-12 h-12 mx-auto" />
        </div>
        <p className="text-gray-600 mb-2">No features selected</p>
        <p className="text-sm text-gray-500">
          Click on a feature on the map to view its properties
        </p>
      </div>
    );
  }

  const feature = selectedFeatures[0];

  const renderGeometry = () => {
    const { type, coordinates } = feature.geometry;

    return (
      <div className="bg-gray-50 rounded-lg p-3 text-xs font-mono">
        <p className="font-semibold mb-1">Geometry Type: {type}</p>
        <pre className="overflow-x-auto text-gray-600">
          {JSON.stringify(coordinates, null, 2)}
        </pre>
      </div>
    );
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-gray-900 mb-3">Feature Properties</h3>

        {selectedFeatures.length > 1 && (
          <div className="mb-3 p-2 bg-blue-50 border border-blue-200 rounded text-sm text-blue-800">
            {selectedFeatures.length} features selected. Showing first feature.
          </div>
        )}

        <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
          <div className="p-3 bg-gray-50 border-b border-gray-200">
            <p className="text-xs font-medium text-gray-700">
              Feature ID: <span className="font-mono">{feature.id}</span>
            </p>
          </div>

          <div className="p-3 space-y-3">
            {Object.entries(feature.properties).length > 0 ? (
              <div className="space-y-2">
                {Object.entries(feature.properties).map(([key, value]) => (
                  <div
                    key={key}
                    className="flex items-start justify-between p-2 hover:bg-gray-50 rounded"
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-xs font-medium text-gray-700">{key}</p>
                      <p className="text-sm text-gray-900 mt-0.5 break-words">
                        {typeof value === 'object'
                          ? JSON.stringify(value)
                          : String(value)}
                      </p>
                    </div>
                    <button
                      onClick={() => copyToClipboard(String(value), key)}
                      className="ml-2 p-1 hover:bg-gray-200 rounded"
                      title="Copy value"
                    >
                      {copiedField === key ? (
                        <Check className="w-3 h-3 text-green-600" />
                      ) : (
                        <Copy className="w-3 h-3 text-gray-600" />
                      )}
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-gray-500 text-center py-4">
                No properties available
              </p>
            )}
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 mb-3">Geometry</h3>
        {renderGeometry()}
      </div>

      <div className="pt-3 border-t border-gray-200">
        <button className="w-full px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors">
          Edit Feature
        </button>
      </div>
    </div>
  );
}
