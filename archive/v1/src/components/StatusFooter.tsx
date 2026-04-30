export function StatusFooter() {
  return (
    <footer className="h-8 bg-surface border-t border-gray-800 flex items-center justify-between px-4 text-xs text-gray-500 shrink-0">
      <div className="flex items-center gap-3">
        <span>v0.1.0-alpha</span>
        <span className="text-gray-700">|</span>
        <span>build 2026.04.26</span>
      </div>
      <div>Ready</div>
    </footer>
  );
}
