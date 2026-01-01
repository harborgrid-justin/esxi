/**
 * 3D Globe Visualization Component
 * Interactive globe with data points and geographic visualization
 */

import React, { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { Scene3D } from './Scene3D';
import { GlobeConfig, GeoPoint } from '../types';

interface GlobeVisualizationProps {
  data: GeoPoint[];
  config: GlobeConfig;
  className?: string;
  style?: React.CSSProperties;
  onPointClick?: (point: GeoPoint) => void;
}

export const GlobeVisualization: React.FC<GlobeVisualizationProps> = ({
  data,
  config,
  className = '',
  style,
  onPointClick,
}) => {
  const markersRef = useRef<THREE.Mesh[]>([]);
  const globeRef = useRef<THREE.Mesh | null>(null);
  const raycasterRef = useRef<THREE.Raycaster>(new THREE.Raycaster());
  const mouseRef = useRef<THREE.Vector2>(new THREE.Vector2());
  const containerRef = useRef<HTMLDivElement>(null);
  const cameraRef = useRef<THREE.Camera | null>(null);

  const {
    radius = 5,
    segments = 64,
    textureUrl,
    showAtmosphere = true,
    rotationSpeed = 0.001,
  } = config;

  const handleSceneReady = (scene: THREE.Scene, camera: THREE.Camera, renderer: THREE.WebGLRenderer) => {
    cameraRef.current = camera;

    // Clear previous objects
    markersRef.current.forEach((marker) => scene.remove(marker));
    markersRef.current = [];

    if (globeRef.current) {
      scene.remove(globeRef.current);
    }

    // Create globe
    const globeGeometry = new THREE.SphereGeometry(radius, segments, segments);

    let globeMaterial: THREE.Material;
    if (textureUrl) {
      const textureLoader = new THREE.TextureLoader();
      const texture = textureLoader.load(textureUrl);
      globeMaterial = new THREE.MeshPhongMaterial({
        map: texture,
        shininess: 20,
      });
    } else {
      // Default earth-like appearance
      globeMaterial = new THREE.MeshPhongMaterial({
        color: 0x2233ff,
        emissive: 0x112244,
        shininess: 20,
      });
    }

    const globe = new THREE.Mesh(globeGeometry, globeMaterial);
    globe.receiveShadow = true;
    globe.castShadow = true;
    globeRef.current = globe;
    scene.add(globe);

    // Add atmosphere
    if (showAtmosphere) {
      const atmosphereGeometry = new THREE.SphereGeometry(radius * 1.05, segments, segments);
      const atmosphereMaterial = new THREE.MeshPhongMaterial({
        color: 0x4488ff,
        transparent: true,
        opacity: 0.2,
        side: THREE.BackSide,
      });
      const atmosphere = new THREE.Mesh(atmosphereGeometry, atmosphereMaterial);
      scene.add(atmosphere);
    }

    // Add data points
    data.forEach((point) => {
      const marker = createMarker(point, radius);
      scene.add(marker);
      markersRef.current.push(marker);

      // Add connecting line from surface to marker
      if (point.value && point.value > 0) {
        const lineHeight = (point.value / 100) * radius * 0.5;
        const markerPos = latLonToVector3(point.lat, point.lng, radius);
        const surfacePos = latLonToVector3(point.lat, point.lng, radius);
        const topPos = latLonToVector3(point.lat, point.lng, radius + lineHeight);

        const points = [surfacePos, topPos];
        const lineGeometry = new THREE.BufferGeometry().setFromPoints(points);
        const lineMaterial = new THREE.LineBasicMaterial({
          color: 0xff6b6b,
          linewidth: 2,
        });
        const line = new THREE.Line(lineGeometry, lineMaterial);
        scene.add(line);
      }
    });

    // Add star background
    addStarField(scene, 1000);

    // Animation loop for rotation
    let animationId: number;
    const animate = () => {
      if (globeRef.current) {
        globeRef.current.rotation.y += rotationSpeed;
      }
      animationId = requestAnimationFrame(animate);
    };
    animate();

    // Handle click events
    const handleClick = (event: MouseEvent) => {
      if (!containerRef.current || !cameraRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      mouseRef.current.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouseRef.current.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycasterRef.current.setFromCamera(mouseRef.current, cameraRef.current);
      const intersects = raycasterRef.current.intersectObjects(markersRef.current);

      if (intersects.length > 0) {
        const firstIntersect = intersects[0];
        const geoPoint = firstIntersect?.object.userData['geoPoint'] as GeoPoint | undefined;
        if (geoPoint && onPointClick) {
          onPointClick(geoPoint);
        }
      }
    };

    renderer.domElement.addEventListener('click', handleClick);

    // Cleanup
    return () => {
      cancelAnimationFrame(animationId);
      renderer.domElement.removeEventListener('click', handleClick);
    };
  };

  const latLonToVector3 = (lat: number, lon: number, radius: number): THREE.Vector3 => {
    const phi = (90 - lat) * (Math.PI / 180);
    const theta = (lon + 180) * (Math.PI / 180);

    const x = -(radius * Math.sin(phi) * Math.cos(theta));
    const y = radius * Math.cos(phi);
    const z = radius * Math.sin(phi) * Math.sin(theta);

    return new THREE.Vector3(x, y, z);
  };

  const createMarker = (point: GeoPoint, globeRadius: number): THREE.Mesh => {
    const markerRadius = 0.1;
    const markerHeight = point.value ? (point.value / 100) * globeRadius * 0.5 : 0.3;

    const geometry = new THREE.SphereGeometry(markerRadius, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: point.value && point.value > 50 ? 0xff4444 : 0x44ff44,
      emissive: point.value && point.value > 50 ? 0x880000 : 0x008800,
      shininess: 100,
    });

    const marker = new THREE.Mesh(geometry, material);
    const position = latLonToVector3(point.lat, point.lng, globeRadius + markerHeight);
    marker.position.copy(position);
    marker.userData = { geoPoint: point };

    // Add pulsing animation
    const scale = 1 + (point.value || 0) / 200;
    marker.scale.set(scale, scale, scale);

    return marker;
  };

  const addStarField = (scene: THREE.Scene, count: number) => {
    const starsGeometry = new THREE.BufferGeometry();
    const starPositions: number[] = [];

    for (let i = 0; i < count; i++) {
      const x = (Math.random() - 0.5) * 2000;
      const y = (Math.random() - 0.5) * 2000;
      const z = (Math.random() - 0.5) * 2000;
      starPositions.push(x, y, z);
    }

    starsGeometry.setAttribute('position', new THREE.Float32BufferAttribute(starPositions, 3));

    const starsMaterial = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 2,
      sizeAttenuation: false,
    });

    const stars = new THREE.Points(starsGeometry, starsMaterial);
    scene.add(stars);
  };

  // Override camera config for globe
  const globeConfig: GlobeConfig = {
    ...config,
    camera: {
      type: 'perspective',
      fov: 45,
      near: 0.1,
      far: 2000,
      position: { x: 0, y: 0, z: 15 },
      lookAt: { x: 0, y: 0, z: 0 },
      ...config.camera,
    },
    lighting: {
      ambient: { color: '#404040', intensity: 0.4 },
      directional: [
        { color: '#ffffff', intensity: 1.2, position: { x: 10, y: 10, z: 10 } },
        { color: '#6688ff', intensity: 0.3, position: { x: -10, y: -10, z: -10 } },
      ],
      ...config.lighting,
    },
    controls: {
      enableRotate: true,
      enableZoom: true,
      enablePan: false,
      autoRotate: false,
      ...config.controls,
    },
  };

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', ...style }}>
      <Scene3D config={globeConfig} className={className} onSceneReady={handleSceneReady}>
        {() => {
          // Scene setup is handled in handleSceneReady
        }}
      </Scene3D>
    </div>
  );
};

export default GlobeVisualization;
