/**
 * 3D Data Visualization Component
 * Renders data as 3D bars, scatter plots, or surfaces using Three.js
 */

import React, { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { Scene3D } from './Scene3D';
import { DataVisualization3DConfig } from '../types';

interface DataPoint3D {
  x: number;
  y: number;
  z: number;
  value: number;
  label?: string;
  category?: string;
}

interface DataVisualization3DProps {
  data: DataPoint3D[];
  config: DataVisualization3DConfig;
  className?: string;
  style?: React.CSSProperties;
  onDataPointClick?: (dataPoint: DataPoint3D) => void;
}

export const DataVisualization3D: React.FC<DataVisualization3DProps> = ({
  data,
  config,
  className = '',
  style,
  onDataPointClick,
}) => {
  const meshesRef = useRef<THREE.Mesh[]>([]);
  const raycasterRef = useRef<THREE.Raycaster>(new THREE.Raycaster());
  const mouseRef = useRef<THREE.Vector2>(new THREE.Vector2());
  const containerRef = useRef<HTMLDivElement>(null);
  const cameraRef = useRef<THREE.Camera | null>(null);

  const {
    dataType = 'bars',
    heightScale = 1,
    colorScale: colorScheme = ['#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b'],
  } = config;

  const handleSceneReady = (scene: THREE.Scene, camera: THREE.Camera, renderer: THREE.WebGLRenderer) => {
    cameraRef.current = camera;

    // Clear previous meshes
    meshesRef.current.forEach((mesh) => scene.remove(mesh));
    meshesRef.current = [];

    // Create color scale
    const colors = colorScheme.map((c) => new THREE.Color(c));
    const maxValue = Math.max(...data.map((d) => d.value));
    const minValue = Math.min(...data.map((d) => d.value));

    const getColor = (value: number): THREE.Color => {
      const normalized = (value - minValue) / (maxValue - minValue);
      const index = Math.floor(normalized * (colors.length - 1));
      const nextIndex = Math.min(index + 1, colors.length - 1);
      const t = (normalized * (colors.length - 1)) % 1;

      const color = new THREE.Color();
      color.lerpColors(colors[index]!, colors[nextIndex]!, t);
      return color;
    };

    if (dataType === 'bars') {
      // Render 3D bars
      data.forEach((point) => {
        const height = point.value * heightScale;
        const geometry = new THREE.BoxGeometry(0.5, height, 0.5);
        const material = new THREE.MeshPhongMaterial({
          color: getColor(point.value),
          shininess: 100,
        });

        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(point.x, height / 2, point.z);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        mesh.userData = { dataPoint: point };

        scene.add(mesh);
        meshesRef.current.push(mesh);
      });
    } else if (dataType === 'scatter') {
      // Render 3D scatter plot
      data.forEach((point) => {
        const radius = 0.2 + (point.value / maxValue) * 0.3;
        const geometry = new THREE.SphereGeometry(radius, 32, 32);
        const material = new THREE.MeshPhongMaterial({
          color: getColor(point.value),
          shininess: 100,
          transparent: true,
          opacity: 0.8,
        });

        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(point.x, point.y * heightScale, point.z);
        mesh.castShadow = true;
        mesh.userData = { dataPoint: point };

        scene.add(mesh);
        meshesRef.current.push(mesh);
      });
    } else if (dataType === 'surface') {
      // Create surface from data points
      const gridSize = Math.ceil(Math.sqrt(data.length));
      const geometry = new THREE.PlaneGeometry(10, 10, gridSize - 1, gridSize - 1);

      // Update vertices based on data
      const positions = geometry.attributes['position'];
      if (positions) {
        for (let i = 0; i < data.length && i < positions.count; i++) {
          const point = data[i];
          if (point) {
            positions.setY(i, point.value * heightScale);
          }
        }
        positions.needsUpdate = true;
      }

      geometry.computeVertexNormals();

      // Create vertex colors
      const vertexColors: number[] = [];
      for (let i = 0; i < data.length && i < (positions?.count || 0); i++) {
        const point = data[i];
        if (point) {
          const color = getColor(point.value);
          vertexColors.push(color.r, color.g, color.b);
        }
      }

      geometry.setAttribute('color', new THREE.Float32BufferAttribute(vertexColors, 3));

      const material = new THREE.MeshPhongMaterial({
        vertexColors: true,
        side: THREE.DoubleSide,
        shininess: 100,
        flatShading: false,
      });

      const mesh = new THREE.Mesh(geometry, material);
      mesh.rotation.x = -Math.PI / 2;
      mesh.position.y = 0;
      mesh.receiveShadow = true;

      scene.add(mesh);
      meshesRef.current.push(mesh);

      // Add wireframe
      const wireframe = new THREE.WireframeGeometry(geometry);
      const lineMaterial = new THREE.LineBasicMaterial({ color: 0x000000, opacity: 0.2, transparent: true });
      const line = new THREE.LineSegments(wireframe, lineMaterial);
      line.rotation.x = -Math.PI / 2;
      line.position.y = 0;
      scene.add(line);
    } else if (dataType === 'network') {
      // Create network visualization
      data.forEach((point, index) => {
        const geometry = new THREE.SphereGeometry(0.3, 32, 32);
        const material = new THREE.MeshPhongMaterial({
          color: getColor(point.value),
          shininess: 100,
        });

        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(point.x, point.y * heightScale, point.z);
        mesh.castShadow = true;
        mesh.userData = { dataPoint: point };

        scene.add(mesh);
        meshesRef.current.push(mesh);

        // Draw connections to nearby points
        if (index > 0) {
          const prevPoint = data[index - 1];
          if (prevPoint && Math.random() > 0.5) {
            const points = [
              new THREE.Vector3(prevPoint.x, prevPoint.y * heightScale, prevPoint.z),
              new THREE.Vector3(point.x, point.y * heightScale, point.z),
            ];

            const lineGeometry = new THREE.BufferGeometry().setFromPoints(points);
            const lineMaterial = new THREE.LineBasicMaterial({
              color: 0x999999,
              opacity: 0.3,
              transparent: true,
            });
            const line = new THREE.Line(lineGeometry, lineMaterial);
            scene.add(line);
          }
        }
      });
    }

    // Add base plane
    const planeGeometry = new THREE.PlaneGeometry(20, 20);
    const planeMaterial = new THREE.ShadowMaterial({ opacity: 0.2 });
    const plane = new THREE.Mesh(planeGeometry, planeMaterial);
    plane.rotation.x = -Math.PI / 2;
    plane.position.y = 0;
    plane.receiveShadow = true;
    scene.add(plane);

    // Handle click events
    const handleClick = (event: MouseEvent) => {
      if (!containerRef.current || !cameraRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      mouseRef.current.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouseRef.current.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycasterRef.current.setFromCamera(mouseRef.current, cameraRef.current);
      const intersects = raycasterRef.current.intersectObjects(meshesRef.current);

      if (intersects.length > 0) {
        const firstIntersect = intersects[0];
        const dataPoint = firstIntersect?.object.userData['dataPoint'] as DataPoint3D | undefined;
        if (dataPoint && onDataPointClick) {
          onDataPointClick(dataPoint);
        }
      }
    };

    renderer.domElement.addEventListener('click', handleClick);

    // Cleanup click listener
    return () => {
      renderer.domElement.removeEventListener('click', handleClick);
    };
  };

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', ...style }}>
      <Scene3D config={config} className={className} onSceneReady={handleSceneReady}>
        {() => {
          // Scene setup is handled in handleSceneReady
        }}
      </Scene3D>
    </div>
  );
};

export default DataVisualization3D;
