import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MapContainer } from './components/Map/MapContainer';
import { ToolBar } from './components/Map/ToolBar';
import { LayerPanel } from './components/Map/LayerPanel';
import { DrawingTools } from './components/Map/DrawingTools';
import { Sidebar } from './components/Sidebar/Sidebar';
import { Layers, Menu } from 'lucide-react';
import { useMapStore } from './stores/mapStore';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 5 * 60 * 1000, // 5 minutes
    },
  },
});

function App() {
  const { setLayerPanelOpen, sidebarOpen, setSidebarOpen } = useMapStore();

  return (
    <QueryClientProvider client={queryClient}>
      <div className="w-screen h-screen flex overflow-hidden bg-gray-900">
        <Sidebar />

        <main className={`flex-1 relative transition-all duration-300 ${sidebarOpen ? 'ml-0' : ''}`}>
          {/* Top Bar */}
          <div className="absolute top-0 left-0 right-0 z-20 bg-white/90 backdrop-blur-sm border-b border-gray-200 px-4 py-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                {!sidebarOpen && (
                  <button
                    onClick={() => setSidebarOpen(true)}
                    className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                    title="Open sidebar"
                  >
                    <Menu className="w-5 h-5 text-gray-700" />
                  </button>
                )}
                <h1 className="text-lg font-bold text-gray-900">Meridian GIS Platform</h1>
              </div>

              <button
                onClick={() => setLayerPanelOpen(true)}
                className="px-3 py-1.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2 text-sm font-medium"
              >
                <Layers className="w-4 h-4" />
                Layers
              </button>
            </div>
          </div>

          {/* Map Container */}
          <MapContainer />

          {/* Map Tools */}
          <ToolBar />

          {/* Layer Panel */}
          <LayerPanel />

          {/* Drawing Tools (headless) */}
          <DrawingTools />

          {/* Map Coordinates Display */}
          <div className="absolute bottom-4 right-4 bg-white/90 backdrop-blur-sm rounded-lg shadow-lg px-3 py-2 text-xs font-mono text-gray-700 border border-gray-200 z-10">
            <MapCoordinates />
          </div>
        </main>
      </div>
    </QueryClientProvider>
  );
}

function MapCoordinates() {
  const { mapState } = useMapStore();

  return (
    <div className="flex flex-col gap-0.5">
      <div>
        Lat: {mapState.center[1].toFixed(6)} Lon: {mapState.center[0].toFixed(6)}
      </div>
      <div>
        Zoom: {mapState.zoom.toFixed(2)} Bearing: {mapState.bearing.toFixed(1)}°
      </div>
    </div>
  );
}

export default App;
