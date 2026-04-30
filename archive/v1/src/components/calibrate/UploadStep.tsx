import { useEffect, useState } from "react";

export function UploadStep({
  onComplete,
}: {
  onComplete: () => void;
}) {
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("Uploading LUT...");

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress((p) => {
        if (p >= 100) {
          clearInterval(interval);
          setStatus("Corrections uploaded successfully");
          setTimeout(onComplete, 2000);
          return 100;
        }
        if (p === 50) setStatus("Applying white balance gains...");
        return p + 10;
      });
    }, 300);
    return () => clearInterval(interval);
  }, [onComplete]);

  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-4">
      <div className="text-lg font-medium text-white">{status}</div>
      <div className="w-64 h-2 bg-gray-800 rounded-full overflow-hidden">
        <div
          className="h-full bg-primary rounded-full transition-all duration-300"
          style={{ width: `${progress}%` }}
        />
      </div>
      <div className="text-sm text-gray-400">{progress}%</div>
    </div>
  );
}
