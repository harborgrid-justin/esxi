/**
 * Three.js Scene Wrapper Component
 * Provides a base scene with camera, lighting, and controls
 */

import React, { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { Scene3DConfig } from '../types';

interface Scene3DProps {
  config: Scene3DConfig;
  className?: string;
  style?: React.CSSProperties;
  onSceneReady?: (scene: THREE.Scene, camera: THREE.Camera, renderer: THREE.WebGLRenderer) => void;
  children?: (scene: THREE.Scene) => void;
}

export const Scene3D: React.FC<Scene3DProps> = ({
  config,
  className = '',
  style,
  onSceneReady,
  children,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.Camera | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const {
      camera: cameraConfig = {},
      lighting: lightingConfig = {},
      controls: controlsConfig = {},
      renderer: rendererConfig = {},
    } = config;

    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Create scene
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf0f0f0);
    sceneRef.current = scene;

    // Create camera
    let camera: THREE.Camera;
    if (cameraConfig.type === 'orthographic') {
      const aspect = width / height;
      const frustumSize = 10;
      camera = new THREE.OrthographicCamera(
        (frustumSize * aspect) / -2,
        (frustumSize * aspect) / 2,
        frustumSize / 2,
        frustumSize / -2,
        cameraConfig.near || 0.1,
        cameraConfig.far || 1000
      );
    } else {
      camera = new THREE.PerspectiveCamera(
        cameraConfig.fov || 75,
        width / height,
        cameraConfig.near || 0.1,
        cameraConfig.far || 1000
      );
    }

    const cameraPosition = cameraConfig.position || { x: 0, y: 5, z: 10 };
    camera.position.set(cameraPosition.x, cameraPosition.y, cameraPosition.z);

    const lookAt = cameraConfig.lookAt || { x: 0, y: 0, z: 0 };
    camera.lookAt(lookAt.x, lookAt.y, lookAt.z);

    cameraRef.current = camera;

    // Create renderer
    const renderer = new THREE.WebGLRenderer({
      antialias: rendererConfig.antialias !== false,
      alpha: rendererConfig.alpha || false,
    });
    renderer.setSize(width, height);
    renderer.setPixelRatio(rendererConfig.pixelRatio || window.devicePixelRatio);

    if (rendererConfig.shadowMap !== false) {
      renderer.shadowMap.enabled = true;
      renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    }

    container.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // Lighting
    const ambientConfig = lightingConfig.ambient || { color: '#ffffff', intensity: 0.5 };
    const ambientLight = new THREE.AmbientLight(
      typeof ambientConfig.color === 'string'
        ? ambientConfig.color
        : `rgb(${ambientConfig.color.r},${ambientConfig.color.g},${ambientConfig.color.b})`,
      ambientConfig.intensity
    );
    scene.add(ambientLight);

    // Directional lights
    const directionalLights = lightingConfig.directional || [
      { color: '#ffffff', intensity: 1, position: { x: 5, y: 10, z: 5 } },
    ];
    directionalLights.forEach((lightConfig) => {
      const light = new THREE.DirectionalLight(
        typeof lightConfig.color === 'string'
          ? lightConfig.color
          : `rgb(${lightConfig.color.r},${lightConfig.color.g},${lightConfig.color.b})`,
        lightConfig.intensity
      );
      light.position.set(lightConfig.position.x, lightConfig.position.y, lightConfig.position.z);
      light.castShadow = rendererConfig.shadowMap !== false;
      light.shadow.camera.near = 0.5;
      light.shadow.camera.far = 500;
      light.shadow.mapSize.width = 2048;
      light.shadow.mapSize.height = 2048;
      scene.add(light);
    });

    // Point lights
    if (lightingConfig.point) {
      lightingConfig.point.forEach((lightConfig) => {
        const light = new THREE.PointLight(
          typeof lightConfig.color === 'string'
            ? lightConfig.color
            : `rgb(${lightConfig.color.r},${lightConfig.color.g},${lightConfig.color.b})`,
          lightConfig.intensity,
          lightConfig.distance || 0
        );
        light.position.set(lightConfig.position.x, lightConfig.position.y, lightConfig.position.z);
        scene.add(light);
      });
    }

    // Controls
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableRotate = controlsConfig.enableRotate !== false;
    controls.enableZoom = controlsConfig.enableZoom !== false;
    controls.enablePan = controlsConfig.enablePan !== false;
    controls.autoRotate = controlsConfig.autoRotate || false;
    controls.autoRotateSpeed = controlsConfig.autoRotateSpeed || 2.0;
    controls.dampingFactor = 0.05;
    controls.enableDamping = true;
    controlsRef.current = controls;

    // Add grid helper
    const gridHelper = new THREE.GridHelper(20, 20, 0xcccccc, 0xeeeeee);
    scene.add(gridHelper);

    // Add axes helper
    const axesHelper = new THREE.AxesHelper(5);
    scene.add(axesHelper);

    // Call children function to add custom objects
    if (children) {
      children(scene);
    }

    // Notify parent component
    if (onSceneReady) {
      onSceneReady(scene, camera, renderer);
    }

    // Animation loop
    const animate = () => {
      animationFrameRef.current = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    // Handle resize
    const handleResize = () => {
      if (!containerRef.current || !cameraRef.current || !rendererRef.current) return;

      const newWidth = containerRef.current.clientWidth;
      const newHeight = containerRef.current.clientHeight;

      if (cameraRef.current instanceof THREE.PerspectiveCamera) {
        cameraRef.current.aspect = newWidth / newHeight;
        cameraRef.current.updateProjectionMatrix();
      } else if (cameraRef.current instanceof THREE.OrthographicCamera) {
        const aspect = newWidth / newHeight;
        const frustumSize = 10;
        cameraRef.current.left = (frustumSize * aspect) / -2;
        cameraRef.current.right = (frustumSize * aspect) / 2;
        cameraRef.current.top = frustumSize / 2;
        cameraRef.current.bottom = frustumSize / -2;
        cameraRef.current.updateProjectionMatrix();
      }

      rendererRef.current.setSize(newWidth, newHeight);
    };

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);

      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      if (controlsRef.current) {
        controlsRef.current.dispose();
      }

      if (rendererRef.current) {
        rendererRef.current.dispose();
        container.removeChild(rendererRef.current.domElement);
      }

      if (sceneRef.current) {
        sceneRef.current.traverse((object) => {
          if (object instanceof THREE.Mesh) {
            object.geometry.dispose();
            if (Array.isArray(object.material)) {
              object.material.forEach((material) => material.dispose());
            } else {
              object.material.dispose();
            }
          }
        });
      }
    };
  }, [config, onSceneReady, children]);

  return (
    <div
      ref={containerRef}
      className={`scene-3d ${className}`}
      style={{
        width: '100%',
        height: '100%',
        minHeight: '400px',
        ...style,
      }}
    />
  );
};

export default Scene3D;
