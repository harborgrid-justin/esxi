import { Layers, Info, BarChart3, X, ChevronLeft, ChevronRight } from 'lucide-react';
import { useMapStore } from '@/stores/mapStore';
import * as Tabs from '@radix-ui/react-tabs';
import { LayerList } from './LayerList';
import { FeatureInfo } from './FeatureInfo';
import { AnalysisPanel } from '../Analysis/AnalysisPanel';

export function Sidebar() {
  const { sidebarOpen, setSidebarOpen, sidebarTab, setSidebarTab } = useMapStore();

  if (!sidebarOpen) {
    return (
      <button
        onClick={() => setSidebarOpen(true)}
        className="absolute left-0 top-1/2 transform -translate-y-1/2 bg-white rounded-r-lg shadow-lg p-2 z-10 hover:bg-gray-50"
        title="Open sidebar"
      >
        <ChevronRight className="w-5 h-5 text-gray-600" />
      </button>
    );
  }

  return (
    <div className="absolute left-0 top-0 bottom-0 w-80 bg-white shadow-xl border-r border-gray-200 z-10 flex flex-col">
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <h1 className="text-xl font-bold text-gray-900">Meridian GIS</h1>
        <button
          onClick={() => setSidebarOpen(false)}
          className="p-1 hover:bg-gray-100 rounded"
          title="Close sidebar"
        >
          <ChevronLeft className="w-5 h-5 text-gray-600" />
        </button>
      </div>

      <Tabs.Root
        value={sidebarTab}
        onValueChange={(value) => setSidebarTab(value as 'layers' | 'properties' | 'analysis')}
        className="flex-1 flex flex-col overflow-hidden"
      >
        <Tabs.List className="flex border-b border-gray-200 bg-gray-50">
          <Tabs.Trigger
            value="layers"
            className="flex-1 px-4 py-3 text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100 data-[state=active]:bg-white data-[state=active]:text-blue-600 data-[state=active]:border-b-2 data-[state=active]:border-blue-600 transition-colors flex items-center justify-center gap-2"
          >
            <Layers className="w-4 h-4" />
            Layers
          </Tabs.Trigger>
          <Tabs.Trigger
            value="properties"
            className="flex-1 px-4 py-3 text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100 data-[state=active]:bg-white data-[state=active]:text-blue-600 data-[state=active]:border-b-2 data-[state=active]:border-blue-600 transition-colors flex items-center justify-center gap-2"
          >
            <Info className="w-4 h-4" />
            Properties
          </Tabs.Trigger>
          <Tabs.Trigger
            value="analysis"
            className="flex-1 px-4 py-3 text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100 data-[state=active]:bg-white data-[state=active]:text-blue-600 data-[state=active]:border-b-2 data-[state=active]:border-blue-600 transition-colors flex items-center justify-center gap-2"
          >
            <BarChart3 className="w-4 h-4" />
            Analysis
          </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="layers" className="flex-1 overflow-y-auto p-4">
          <LayerList />
        </Tabs.Content>

        <Tabs.Content value="properties" className="flex-1 overflow-y-auto p-4">
          <FeatureInfo />
        </Tabs.Content>

        <Tabs.Content value="analysis" className="flex-1 overflow-y-auto p-4">
          <AnalysisPanel />
        </Tabs.Content>
      </Tabs.Root>
    </div>
  );
}
