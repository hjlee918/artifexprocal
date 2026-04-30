import { Canvas } from "@react-three/fiber";
import type { ReactNode } from "react";

export interface ThreeCanvasProps {
  children: ReactNode;
  className?: string;
}

export function ThreeCanvas({ children, className = "" }: ThreeCanvasProps) {
  return (
    <div
      className={`rounded-lg border border-gray-700 overflow-hidden ${className}`}
      style={{ background: "#1a1a1a" }}
    >
      <Canvas camera={{ position: [3, 3, 3], fov: 50 }} style={{ height: "100%", width: "100%" }}>
        <ambientLight intensity={0.5} />
        <pointLight position={[10, 10, 10]} />
        {children}
      </Canvas>
    </div>
  );
}
