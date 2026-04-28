import { useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import * as THREE from "three";

export interface LutCubeSceneProps {
  lutSize?: number;
  lutData?: number[]; // flattened [r,g,b, r,g,b, ...] in lattice order, 0-1 range
}

function buildLutGeometry(lutSize: number, lutData: number[]) {
  const count = lutSize * lutSize * lutSize;
  const positions = new Float32Array(count * 3);
  const colors = new Float32Array(count * 3);

  let idx = 0;
  for (let b = 0; b < lutSize; b++) {
    for (let g = 0; g < lutSize; g++) {
      for (let r = 0; r < lutSize; r++) {
        const dataIdx = ((b * lutSize + g) * lutSize + r) * 3;
        // Position in input RGB space, centered at origin
        positions[idx * 3] = r / (lutSize - 1) - 0.5;
        positions[idx * 3 + 1] = g / (lutSize - 1) - 0.5;
        positions[idx * 3 + 2] = b / (lutSize - 1) - 0.5;
        // Color from corrected output
        colors[idx * 3] = lutData[dataIdx] ?? 0;
        colors[idx * 3 + 1] = lutData[dataIdx + 1] ?? 0;
        colors[idx * 3 + 2] = lutData[dataIdx + 2] ?? 0;
        idx++;
      }
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
  return geometry;
}

export function LutCubeScene({ lutSize = 33, lutData }: LutCubeSceneProps) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame((_, delta) => {
    if (groupRef.current) {
      groupRef.current.rotation.y += delta * 0.2;
    }
  });

  const pointsGeometry = useMemo(() => {
    if (!lutData || lutData.length === 0) return null;
    return buildLutGeometry(lutSize, lutData);
  }, [lutData, lutSize]);

  return (
    <>
      <OrbitControls enablePan={false} />
      <group ref={groupRef}>
        {pointsGeometry ? (
          <points geometry={pointsGeometry}>
            <pointsMaterial size={0.015} vertexColors sizeAttenuation />
          </points>
        ) : (
          <mesh>
            <boxGeometry args={[1, 1, 1]} />
            <meshBasicMaterial color="#2563eb" wireframe />
          </mesh>
        )}
        <axesHelper args={[1.5]} />
      </group>
    </>
  );
}
