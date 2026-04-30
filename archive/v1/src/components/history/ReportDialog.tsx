import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ReportFormat, ReportRequestDto, ReportTemplate } from "../../bindings";

interface ReportDialogProps {
  sessionId: string;
  sessionName: string;
  compareSessions?: { id: string; name: string }[];
  onClose: () => void;
}

export function ReportDialog({
  sessionId,
  sessionName,
  compareSessions,
  onClose,
}: ReportDialogProps) {
  const [template, setTemplate] = useState<ReportTemplate>("QuickSummary");
  const [format, setFormat] = useState<ReportFormat>("Html");
  const [compareId, setCompareId] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async (preview: boolean) => {
    setLoading(true);
    setError(null);
    try {
      const request: ReportRequestDto = {
        session_id: sessionId,
        template,
        format,
        compare_session_id: template === "PrePostComparison" && compareId ? compareId : null,
      };

      const response = await invoke<{ path: string; format: ReportFormat }>(
        "generate_report",
        { request }
      );

      if (preview) {
        if (response.format === "Html") {
          window.open(`file://${response.path}`, "_blank");
        } else {
          setError("Preview is only available for HTML reports");
        }
      } else {
        const a = document.createElement("a");
        a.href = `file://${response.path}`;
        a.download = `${sessionName.replace(/\s+/g, "_")}_report.${response.format === "Html" ? "html" : "pdf"}`;
        a.click();
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-700 rounded-lg p-6 w-full max-w-md">
        <h2 className="text-lg font-semibold text-white mb-4">Generate Report</h2>
        <p className="text-sm text-gray-400 mb-4">{sessionName}</p>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Template</label>
            <select
              className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-white"
              value={template}
              onChange={(e) => setTemplate(e.target.value as ReportTemplate)}
            >
              <option value="QuickSummary">Quick Summary</option>
              <option value="Detailed">Detailed</option>
              <option value="PrePostComparison">Pre/Post Comparison</option>
            </select>
          </div>

          {template === "PrePostComparison" && compareSessions && compareSessions.length > 0 && (
            <div>
              <label className="block text-sm text-gray-400 mb-1">Compare With</label>
              <select
                className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-white"
                value={compareId}
                onChange={(e) => setCompareId(e.target.value)}
              >
                <option value="">Select session...</option>
                {compareSessions.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div>
            <label className="block text-sm text-gray-400 mb-1">Format</label>
            <div className="flex space-x-4">
              {(["Html", "Pdf"] as ReportFormat[]).map((f) => (
                <label key={f} className="flex items-center space-x-2 text-sm text-white">
                  <input
                    type="radio"
                    name="format"
                    value={f}
                    checked={format === f}
                    onChange={() => setFormat(f)}
                  />
                  <span>{f === "Html" ? "HTML" : "PDF"}</span>
                </label>
              ))}
            </div>
          </div>
        </div>

        {error && (
          <div className="mt-4 p-2 bg-red-900/30 border border-red-700 rounded text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="mt-6 flex justify-end space-x-3">
          <button
            className="px-4 py-2 text-sm text-gray-300 hover:text-white"
            onClick={onClose}
            disabled={loading}
          >
            Cancel
          </button>
          <button
            className="px-4 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded text-white"
            onClick={() => handleGenerate(true)}
            disabled={loading}
          >
            Preview
          </button>
          <button
            className="px-4 py-2 text-sm bg-blue-700 hover:bg-blue-600 rounded text-white"
            onClick={() => handleGenerate(false)}
            disabled={loading}
          >
            {loading ? "Generating..." : "Download"}
          </button>
        </div>
      </div>
    </div>
  );
}
