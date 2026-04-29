import { useState, useEffect } from "react";
import { commands, SessionSummaryDto, SessionDetailDto, SessionFilterDto } from "../../bindings";
import { SessionTable } from "../history/SessionTable";
import { SessionDetailView } from "../history/SessionDetailView";
import { SessionCompareView } from "../history/SessionCompareView";

export function HistoryView() {
  const [mode, setMode] = useState<"list" | "detail" | "compare">("list");
  const [sessions, setSessions] = useState<SessionSummaryDto[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [perPage] = useState(10);
  const [detail, setDetail] = useState<SessionDetailDto | null>(null);
  const [compareA, setCompareA] = useState<SessionDetailDto | null>(null);
  const [compareB, setCompareB] = useState<SessionDetailDto | null>(null);
  const [filter, setFilter] = useState<SessionFilterDto>({
    target_space: null,
    state: null,
    date_from: null,
    date_to: null,
    search: null,
  });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSessions();
  }, [page, filter]);

  async function loadSessions() {
    try {
      setError(null);
      const response = await commands.listSessions(filter, page, perPage);
      setSessions(response.items);
      setTotal(response.total);
    } catch (e) {
      setError(String(e));
    }
  }

  async function viewSession(id: string) {
    try {
      setError(null);
      const d = await commands.getSessionDetail(id);
      setDetail(d);
      setMode("detail");
    } catch (e) {
      setError(String(e));
    }
  }

  async function startCompare(id: string) {
    try {
      setError(null);
      if (!compareA) {
        const d = await commands.getSessionDetail(id);
        setCompareA(d);
      } else if (!compareB && compareA.summary.id !== id) {
        const d = await commands.getSessionDetail(id);
        setCompareB(d);
        setMode("compare");
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleExport(format: string) {
    if (!detail) return;
    try {
      const path = await commands.exportSessionData(detail.summary.id, format);
      alert(`Exported to ${path}`);
    } catch (e) {
      setError(String(e));
    }
  }

  if (mode === "detail" && detail) {
    return (
      <SessionDetailView
        detail={detail}
        onBack={() => {
          setMode("list");
          setDetail(null);
        }}
        onExport={handleExport}
        onCompare={() => startCompare(detail.summary.id)}
      />
    );
  }

  if (mode === "compare" && compareA && compareB) {
    return (
      <SessionCompareView
        sessionA={compareA}
        sessionB={compareB}
        onBack={() => {
          setMode("list");
          setCompareA(null);
          setCompareB(null);
        }}
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-white">Session History</h2>
        {compareA && !compareB && (
          <div className="text-sm text-gray-400">
            Select another session to compare with{" "}
            <span className="text-white">{compareA.summary.name}</span>
          </div>
        )}
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-2">
        <input
          type="text"
          placeholder="Search..."
          className="px-3 py-1.5 bg-gray-800 border border-gray-700 rounded text-sm text-white"
          value={filter.search ?? ""}
          onChange={(e) =>
            setFilter((f) => ({ ...f, search: e.target.value || null }))
          }
        />
        <select
          className="px-3 py-1.5 bg-gray-800 border border-gray-700 rounded text-sm text-white"
          value={filter.target_space ?? ""}
          onChange={(e) =>
            setFilter((f) => ({
              ...f,
              target_space: e.target.value || null,
            }))
          }
        >
          <option value="">All Targets</option>
          <option value="BT.709">Rec.709</option>
          <option value="BT.2020">Rec.2020</option>
          <option value="DCI-P3">DCI-P3</option>
        </select>
        <select
          className="px-3 py-1.5 bg-gray-800 border border-gray-700 rounded text-sm text-white"
          value={filter.state ?? ""}
          onChange={(e) =>
            setFilter((f) => ({ ...f, state: e.target.value || null }))
          }
        >
          <option value="">All States</option>
          <option value="finished">Finished</option>
          <option value="error">Error</option>
          <option value="aborted">Aborted</option>
        </select>
        <button
          className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded"
          onClick={() => {
            setPage(0);
            loadSessions();
          }}
        >
          Apply
        </button>
        <button
          className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded"
          onClick={() => {
            setFilter({
              target_space: null,
              state: null,
              date_from: null,
              date_to: null,
              search: null,
            });
            setPage(0);
          }}
        >
          Reset
        </button>
      </div>

      {error && (
        <div className="text-sm text-red-400 bg-red-900/20 border border-red-800 rounded p-3">
          {error}
        </div>
      )}

      <SessionTable
        sessions={sessions}
        total={total}
        page={page}
        perPage={perPage}
        onPageChange={setPage}
        onView={viewSession}
        onCompare={startCompare}
      />
    </div>
  );
}
