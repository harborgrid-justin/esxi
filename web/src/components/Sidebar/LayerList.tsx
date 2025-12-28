import { Plus, Eye, EyeOff, Settings, Trash2, ChevronDown, ChevronRight } from 'lucide-react';
import { useState } from 'react';
import { useMapStore } from '@/stores/mapStore';
import { useLayers } from '@/hooks/useLayers';
import * as Switch from '@radix-ui/react-switch';
import * as Slider from '@radix-ui/react-slider';
import * as Popover from '@radix-ui/react-popover';

export function LayerList() {
  const { layers, deleteLayer } = useLayers();
  const { toggleLayerVisibility, setLayerOpacity } = useMapStore();
  const [expandedLayers, setExpandedLayers] = useState<Set<string>>(new Set());

  const toggleExpand = (layerId: string) => {
    setExpandedLayers((prev) => {
      const next = new Set(prev);
      if (next.has(layerId)) {
        next.delete(layerId);
      } else {
        next.add(layerId);
      }
      return next;
    });
  };

  if (!layers || layers.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="text-gray-400 mb-4">
          <Layers className="w-12 h-12 mx-auto" />
        </div>
        <p className="text-gray-600 mb-4">No layers yet</p>
        <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2 mx-auto">
          <Plus className="w-4 h-4" />
          Add Layer
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-gray-900">Map Layers</h3>
        <button
          className="p-1.5 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          title="Add Layer"
        >
          <Plus className="w-4 h-4" />
        </button>
      </div>

      <div className="space-y-1">
        {layers.map((layer) => {
          const isExpanded = expandedLayers.has(layer.id);

          return (
            <div
              key={layer.id}
              className="border border-gray-200 rounded-lg overflow-hidden bg-white"
            >
              <div className="p-3 hover:bg-gray-50 transition-colors">
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => toggleExpand(layer.id)}
                    className="p-0.5 hover:bg-gray-200 rounded"
                  >
                    {isExpanded ? (
                      <ChevronDown className="w-4 h-4 text-gray-600" />
                    ) : (
                      <ChevronRight className="w-4 h-4 text-gray-600" />
                    )}
                  </button>

                  <Switch.Root
                    checked={layer.visible}
                    onCheckedChange={() => toggleLayerVisibility(layer.id)}
                    className="w-9 h-5 bg-gray-300 rounded-full relative data-[state=checked]:bg-blue-600 transition-colors"
                  >
                    <Switch.Thumb className="block w-4 h-4 bg-white rounded-full transition-transform translate-x-0.5 data-[state=checked]:translate-x-4" />
                  </Switch.Root>

                  <span className="flex-1 text-sm font-medium text-gray-900">
                    {layer.name}
                  </span>

                  <div className="flex gap-1">
                    <Popover.Root>
                      <Popover.Trigger asChild>
                        <button
                          className="p-1 hover:bg-gray-200 rounded"
                          title="Layer Settings"
                        >
                          <Settings className="w-4 h-4 text-gray-600" />
                        </button>
                      </Popover.Trigger>
                      <Popover.Portal>
                        <Popover.Content
                          className="bg-white rounded-lg shadow-xl border border-gray-200 p-4 w-64 z-50"
                          sideOffset={5}
                        >
                          <div className="space-y-3">
                            <div>
                              <h4 className="text-sm font-semibold mb-2">Layer Settings</h4>
                            </div>

                            <div>
                              <label className="block text-xs font-medium text-gray-700 mb-2">
                                Opacity: {Math.round(layer.opacity * 100)}%
                              </label>
                              <Slider.Root
                                className="relative flex items-center select-none touch-none w-full h-5"
                                value={[layer.opacity]}
                                onValueChange={(value) => setLayerOpacity(layer.id, value[0])}
                                max={1}
                                step={0.01}
                              >
                                <Slider.Track className="bg-gray-200 relative grow rounded-full h-1">
                                  <Slider.Range className="absolute bg-blue-500 rounded-full h-full" />
                                </Slider.Track>
                                <Slider.Thumb className="block w-3 h-3 bg-white border-2 border-blue-500 rounded-full hover:bg-gray-50" />
                              </Slider.Root>
                            </div>

                            <div className="pt-2 border-t border-gray-200">
                              <p className="text-xs text-gray-600">Type: {layer.type}</p>
                              <p className="text-xs text-gray-600">Source: {layer.source.type}</p>
                            </div>
                          </div>
                          <Popover.Arrow className="fill-white" />
                        </Popover.Content>
                      </Popover.Portal>
                    </Popover.Root>

                    <button
                      onClick={() => deleteLayer(layer.id)}
                      className="p-1 hover:bg-red-100 rounded"
                      title="Delete Layer"
                    >
                      <Trash2 className="w-4 h-4 text-red-600" />
                    </button>
                  </div>
                </div>
              </div>

              {isExpanded && (
                <div className="px-3 pb-3 border-t border-gray-100 bg-gray-50">
                  <div className="mt-2 space-y-1 text-xs text-gray-600">
                    <p>ID: {layer.id}</p>
                    <p>Type: {layer.type}</p>
                    <p>Source: {layer.source.type}</p>
                    {layer.source.url && <p>URL: {layer.source.url}</p>}
                    {layer.metadata && (
                      <div className="mt-2">
                        <p className="font-medium">Metadata:</p>
                        <pre className="mt-1 text-xs overflow-x-auto">
                          {JSON.stringify(layer.metadata, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
