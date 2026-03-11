import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LibraryEntry } from "../types";

interface Props {
  onDragStart: (entry: LibraryEntry) => void;
  onStatusChange: (msg: string) => void;
}

export default function LibraryBrowser({ onDragStart, onStatusChange }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<LibraryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  const search = useCallback(async (q: string) => {
    setLoading(true);
    try {
      const entries = await invoke<LibraryEntry[]>("search_symbols", { query: q });
      setResults(entries);
    } catch (e) {
      onStatusChange(`Search error: ${e}`);
    } finally {
      setLoading(false);
    }
  }, [onStatusChange]);

  // Initial load
  useEffect(() => { search(""); }, [search]);

  // Debounce search as user types
  useEffect(() => {
    const t = setTimeout(() => search(query), 200);
    return () => clearTimeout(t);
  }, [query, search]);

  const handleDragStart = useCallback(
    (e: React.DragEvent, entry: LibraryEntry) => {
      e.dataTransfer.setData("application/json", JSON.stringify(entry));
      e.dataTransfer.effectAllowed = "copy";
      onDragStart(entry);
      onStatusChange(`Dragging ${entry.name}…`);
    },
    [onDragStart, onStatusChange]
  );

  return (
    <aside className="w-72 flex flex-col bg-surface border-l border-border flex-shrink-0">
      {/* Header */}
      <div className="flex items-center justify-between px-3 pt-3 pb-2 flex-shrink-0">
        <span className="text-sm font-medium text-white">Components</span>
        <span className="text-xs text-gray-500">{results.length} results</span>
      </div>

      {/* Search input — native HTML input, always works */}
      <div className="px-2 pb-2 flex-shrink-0">
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-canvas border border-border focus-within:border-accent transition-colors">
          <span className="text-gray-500 text-xs">🔍</span>
          <input
            ref={searchRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.stopPropagation()} // prevent tool shortcuts while typing
            placeholder="Search components…"
            className="flex-1 bg-transparent text-white text-xs outline-none placeholder-gray-600 min-w-0"
          />
          {loading && <span className="text-gray-500 text-xs animate-spin">⟳</span>}
          {query && !loading && (
            <button
              onClick={() => setQuery("")}
              className="text-gray-500 hover:text-white text-xs transition-colors"
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {/* Results — scrollable */}
      <div className="flex-1 overflow-y-auto px-2 pb-2">
        {results.length === 0 && !loading && (
          <div className="text-xs text-gray-600 text-center py-8">
            {query ? "No matching symbols" : "Loading libraries…"}
          </div>
        )}

        <div className="flex flex-col gap-0.5">
          {results.map((entry, i) => (
            <LibraryRow
              key={`${entry.lib_path}::${entry.symbol_name}::${i}`}
              entry={entry}
              onDragStart={handleDragStart}
            />
          ))}
        </div>
      </div>
    </aside>
  );
}

function LibraryRow({
  entry,
  onDragStart,
}: {
  entry: LibraryEntry;
  onDragStart: (e: React.DragEvent, entry: LibraryEntry) => void;
}) {
  const isLibrary = !entry.symbol_name;

  return (
    <div
      draggable={!isLibrary}
      onDragStart={isLibrary ? undefined : (e) => onDragStart(e, entry)}
      className={[
        "flex items-center gap-2 px-2 py-2 rounded-lg transition-colors group",
        isLibrary
          ? "cursor-default opacity-60"
          : "cursor-grab hover:bg-elevated active:cursor-grabbing",
      ].join(" ")}
    >
      {/* Icon */}
      <div className="w-7 h-7 rounded-md bg-canvas flex items-center justify-center flex-shrink-0">
        <span className="text-accent text-xs">{isLibrary ? "📁" : "◫"}</span>
      </div>

      {/* Text */}
      <div className="flex-1 min-w-0">
        <div className="text-xs text-white truncate">{entry.name}</div>
        <div className="text-[10px] text-gray-600 truncate">
          {entry.symbol_name || entry.lib_path.split("/").pop()}
        </div>
      </div>

      {/* Drag handle indicator */}
      {!isLibrary && (
        <span className="text-gray-700 text-xs group-hover:text-gray-400 transition-colors flex-shrink-0">
          ⠿
        </span>
      )}
    </div>
  );
}
