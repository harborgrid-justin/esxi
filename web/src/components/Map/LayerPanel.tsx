import { useState } from 'react';
import { X, Plus, Eye, EyeOff, Settings, Trash2 } from 'lucide-react';
import { useMapStore } from '@/stores/mapStore';
import { useLayers } from '@/hooks/useLayers';
import * as Dialog from '@radix-ui/react-dialog';
import * as Slider from '@radix-ui/react-slider';

export function LayerPanel() {
  const { layerPanelOpen, setLayerPanelOpen } = useMapStore();
  const { layers, deleteLayer } = useLayers();
  const { toggleLayerVisibility, setLayerOpacity } = useMapStore();
  const [selectedLayer, setSelectedLayer] = useState<string | null>(null);

  if (!layerPanelOpen) return null;

  return (
    <div className="absolute top-20 right-4 w-80 bg-white rounded-lg shadow-xl border border-gray-200 z-10">
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <h2 className="text-lg font-semibold text-gray-900">Layers</h2>
        <div className="flex gap-2">
          <button
            className="p-1 hover:bg-gray-100 rounded"
            title="Add Layer"
          >
            <Plus className="w-5 h-5 text-gray-600" />
          </button>
          <button
            onClick={() => setLayerPanelOpen(false)}
            className="p-1 hover:bg-gray-100 rounded"
          >
            <X className="w-5 h-5 text-gray-600" />
          </button>
        </div>
      </div>

      <div className="max-h-96 overflow-y-auto">
        {layers && layers.length > 0 ? (
          <div className="divide-y divide-gray-200">
            {layers.map((layer) => (
              <div key={layer.id} className="p-3 hover:bg-gray-50">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 flex-1">
                    <button
                      onClick={() => toggleLayerVisibility(layer.id)}
                      className="p-1 hover:bg-gray-200 rounded"
                      title={layer.visible ? 'Hide layer' : 'Show layer'}
                    >
                      {layer.visible ? (
                        <Eye className="w-4 h-4 text-gray-600" />
                      ) : (
                        <EyeOff className="w-4 h-4 text-gray-400" />
                      )}
                    </button>
                    <span className="text-sm font-medium text-gray-900">{layer.name}</span>
                  </div>

                  <div className="flex gap-1">
                    <Dialog.Root>
                      <Dialog.Trigger asChild>
                        <button
                          onClick={() => setSelectedLayer(layer.id)}
                          className="p-1 hover:bg-gray-200 rounded"
                          title="Layer settings"
                        >
                          <Settings className="w-4 h-4 text-gray-600" />
                        </button>
                      </Dialog.Trigger>
                      <Dialog.Portal>
                        <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
                        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-white rounded-lg shadow-xl p-6 w-96 z-50">
                          <Dialog.Title className="text-lg font-semibold mb-4">
                            Layer Settings: {layer.name}
                          </Dialog.Title>

                          <div className="space-y-4">
                            <div>
                              <label className="block text-sm font-medium text-gray-700 mb-2">
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
                                <Slider.Thumb className="block w-4 h-4 bg-white border-2 border-blue-500 rounded-full hover:bg-gray-50" />
                              </Slider.Root>
                            </div>

                            <div>
                              <p className="text-sm text-gray-600">Type: {layer.type}</p>
                              <p className="text-sm text-gray-600">Source: {layer.source.type}</p>
                            </div>
                          </div>

                          <Dialog.Close asChild>
                            <button className="absolute top-4 right-4 p-1 hover:bg-gray-100 rounded">
                              <X className="w-5 h-5" />
                            </button>
                          </Dialog.Close>
                        </Dialog.Content>
                      </Dialog.Portal>
                    </Dialog.Root>

                    <button
                      onClick={() => deleteLayer(layer.id)}
                      className="p-1 hover:bg-red-100 rounded"
                      title="Delete layer"
                    >
                      <Trash2 className="w-4 h-4 text-red-600" />
                    </button>
                  </div>
                </div>

                <div className="mt-2 text-xs text-gray-500">
                  {layer.type} layer
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="p-8 text-center text-gray-500">
            <p>No layers available</p>
            <button className="mt-2 text-blue-600 hover:text-blue-700 text-sm">
              Add your first layer
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
