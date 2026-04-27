import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import * as THREE from "three";

export function LutCubeScene() {
  const cubeRef = useRef<THREE.Mesh>(null);

  useFrame((_, delta) => {
    if (cubeRef.current) {
      cubeRef.current.rotation.y += delta * 0.2;
    }
  });

  return (
    <>
      <OrbitControls enablePan={false} />
      <mesh ref={cubeRef}>
        <boxGeometry args={[1, 1, 1]} />
        <meshBasicMaterial color="#2563eb" wireframe />
      </mesh>
      <axesHelper args={[1.5]} />
    </>
  );
}
