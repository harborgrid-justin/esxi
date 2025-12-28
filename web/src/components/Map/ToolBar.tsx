import {
  MousePointer2,
  Hand,
  PointerSelect as Select,
  MapPin,
  Minus,
  Pentagon,
  Ruler,
  Maximize2,
  Info,
} from 'lucide-react';
import { useMapStore } from '@/stores/mapStore';
import type { MapTool } from '@/types';

const tools: Array<{ id: MapTool; icon: React.ComponentType<any>; label: string }> = [
  { id: 'pan', icon: Hand, label: 'Pan' },
  { id: 'select', icon: Select, label: 'Select' },
  { id: 'identify', icon: Info, label: 'Identify' },
  { id: 'draw-point', icon: MapPin, label: 'Draw Point' },
  { id: 'draw-line', icon: Minus, label: 'Draw Line' },
  { id: 'draw-polygon', icon: Pentagon, label: 'Draw Polygon' },
  { id: 'measure-distance', icon: Ruler, label: 'Measure Distance' },
  { id: 'measure-area', icon: Maximize2, label: 'Measure Area' },
];

export function ToolBar() {
  const { activeTool, setActiveTool } = useMapStore();

  const handleToolClick = (toolId: MapTool) => {
    if (activeTool === toolId) {
      setActiveTool(null);
    } else {
      setActiveTool(toolId);
    }
  };

  return (
    <div className="absolute top-4 left-1/2 transform -translate-x-1/2 z-10">
      <div className="bg-white rounded-lg shadow-lg border border-gray-200 p-1 flex gap-1">
        {tools.map((tool) => {
          const Icon = tool.icon;
          const isActive = activeTool === tool.id;

          return (
            <button
              key={tool.id}
              onClick={() => handleToolClick(tool.id)}
              className={`
                p-2 rounded hover:bg-gray-100 transition-colors
                ${isActive ? 'bg-blue-500 text-white hover:bg-blue-600' : 'text-gray-700'}
              `}
              title={tool.label}
              aria-label={tool.label}
            >
              <Icon className="w-5 h-5" />
            </button>
          );
        })}
      </div>
    </div>
  );
}
