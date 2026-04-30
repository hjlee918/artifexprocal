import { useState, useRef, useEffect } from "react";

interface ExportMenuProps {
  onExport: (format: string) => void;
  onGenerateReport?: () => void;
}

export function ExportMenu({ onExport, onGenerateReport }: ExportMenuProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div className="relative" ref={ref}>
      <button
        className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded"
        onClick={() => setOpen((o) => !o)}
      >
        Export ▼
      </button>
      {open && (
        <div className="absolute right-0 mt-1 w-32 bg-gray-800 border border-gray-700 rounded shadow-lg z-10">
          {["CSV", "JSON"].map((fmt) => (
            <button
              key={fmt}
              className="block w-full text-left px-3 py-2 text-sm hover:bg-gray-700"
              onClick={() => {
                onExport(fmt.toLowerCase());
                setOpen(false);
              }}
            >
              {fmt}
            </button>
          ))}
          {onGenerateReport && (
            <button
              className="block w-full text-left px-3 py-2 text-sm hover:bg-gray-700 border-t border-gray-700"
              onClick={() => {
                onGenerateReport();
                setOpen(false);
              }}
            >
              Generate Report
            </button>
          )}
        </div>
      )}
    </div>
  );
}
