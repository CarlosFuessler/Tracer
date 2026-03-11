import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LibraryEntry, LibraryGroup } from "../types";

interface Props {
  activeSymbol: LibraryEntry | null;
  onDragStart: (entry: LibraryEntry) => void;
  onDragEnd: () => void;
  onPlaceRequest: (entry: LibraryEntry) => void;
  onStatusChange: (message: string) => void;
}

function libraryLabelFromPath(path: string) {
  return path.split(/[\\/]/).pop()?.replace(/\.kicad_sym$/i, "") ?? path;
}

function matchesLibrary(library: LibraryGroup, query: string) {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return true;
  }

  return (
    library.name.toLowerCase().includes(normalizedQuery) ||
    library.lib_path.toLowerCase().includes(normalizedQuery)
  );
}

export default function LibraryBrowser({
  activeSymbol,
  onDragStart,
  onDragEnd,
  onPlaceRequest,
  onStatusChange,
}: Props) {
  const [libraryQuery, setLibraryQuery] = useState("");
  const [symbolQuery, setSymbolQuery] = useState("");
  const [libraries, setLibraries] = useState<LibraryGroup[]>([]);
  const [selectedLibraryPath, setSelectedLibraryPath] = useState("");
  const [librarySymbols, setLibrarySymbols] = useState<LibraryEntry[]>([]);
  const [searchResults, setSearchResults] = useState<LibraryEntry[]>([]);
  const [loadingLibraries, setLoadingLibraries] = useState(false);
  const [loadingSymbols, setLoadingSymbols] = useState(false);
  const [loadingSearch, setLoadingSearch] = useState(false);

  const loadLibraries = useCallback(async () => {
    setLoadingLibraries(true);
    try {
      const groups = await invoke<LibraryGroup[]>("list_symbol_libraries");
      setLibraries(groups);
      setSelectedLibraryPath((currentPath) => {
        if (groups.some((library) => library.lib_path === currentPath)) {
          return currentPath;
        }
        return groups[0]?.lib_path ?? "";
      });
      if (groups.length === 0) {
        setLibrarySymbols([]);
      }
    } catch (error) {
      setLibraries([]);
      setLibrarySymbols([]);
      onStatusChange(`Libraries unavailable: ${error}`);
    } finally {
      setLoadingLibraries(false);
    }
  }, [onStatusChange]);

  const loadSymbolsForLibrary = useCallback(
    async (libPath: string) => {
      if (!libPath) {
        setLibrarySymbols([]);
        return;
      }

      setLoadingSymbols(true);
      try {
        const symbols = await invoke<LibraryEntry[]>("list_symbols_in_library", { libPath });
        setLibrarySymbols(symbols);
      } catch (error) {
        setLibrarySymbols([]);
        onStatusChange(`Symbols unavailable: ${error}`);
      } finally {
        setLoadingSymbols(false);
      }
    },
    [onStatusChange]
  );

  useEffect(() => {
    void loadLibraries();
  }, [loadLibraries]);

  const filteredLibraries = useMemo(
    () => libraries.filter((library) => matchesLibrary(library, libraryQuery)),
    [libraries, libraryQuery]
  );

  useEffect(() => {
    setSelectedLibraryPath((currentPath) => {
      if (filteredLibraries.length === 0) {
        return currentPath ? "" : currentPath;
      }
      if (filteredLibraries.some((library) => library.lib_path === currentPath)) {
        return currentPath;
      }
      return filteredLibraries[0].lib_path;
    });
  }, [filteredLibraries]);

  useEffect(() => {
    if (symbolQuery.trim()) {
      return;
    }
    void loadSymbolsForLibrary(selectedLibraryPath);
  }, [loadSymbolsForLibrary, selectedLibraryPath, symbolQuery]);

  useEffect(() => {
    const trimmed = symbolQuery.trim();
    if (!trimmed) {
      setSearchResults([]);
      return;
    }

    setLoadingSearch(true);
    const timer = window.setTimeout(() => {
      invoke<LibraryEntry[]>("search_symbols", { query: trimmed })
        .then(setSearchResults)
        .catch((error) => {
          setSearchResults([]);
          onStatusChange(`Search failed: ${error}`);
        })
        .finally(() => setLoadingSearch(false));
    }, 140);

    return () => {
      window.clearTimeout(timer);
      setLoadingSearch(false);
    };
  }, [onStatusChange, symbolQuery]);

  const filteredLibraryPaths = useMemo(
    () => filteredLibraries.map((library) => library.lib_path),
    [filteredLibraries]
  );
  const selectedLibrary = useMemo(
    () => libraries.find((library) => library.lib_path === selectedLibraryPath) ?? null,
    [libraries, selectedLibraryPath]
  );
  const isSearchingSymbols = symbolQuery.trim().length > 0;
  const isFilteringLibraries = libraryQuery.trim().length > 0;
  const visibleSymbols = useMemo(() => {
    if (!isSearchingSymbols) {
      return librarySymbols;
    }

    if (!isFilteringLibraries) {
      return searchResults;
    }

    const allowedLibraryPaths = new Set(filteredLibraryPaths);
    return searchResults.filter((entry) => allowedLibraryPaths.has(entry.lib_path));
  }, [filteredLibraryPaths, isFilteringLibraries, isSearchingSymbols, librarySymbols, searchResults]);

  const handleDrag = useCallback(
    (event: React.DragEvent, entry: LibraryEntry) => {
      const payload = JSON.stringify(entry);
      event.dataTransfer.setData("application/json", payload);
      event.dataTransfer.setData("text/plain", payload);
      event.dataTransfer.effectAllowed = "copyMove";
      onDragStart(entry);
      onStatusChange(`Drag ${entry.name}`);
    },
    [onDragStart, onStatusChange]
  );

  return (
    <aside className="tracer-panel flex h-full min-h-0 flex-col overflow-hidden rounded-[28px] p-2.5 shadow-[0_28px_80px_rgba(0,0,0,0.36)]">
      <div className="flex items-center gap-2 pb-2">
        <div className="min-w-0 flex-1">
          <div className="text-[11px] font-semibold uppercase tracking-[0.26em] text-white/72">
            Parts
          </div>
          <div className="truncate text-[10px] text-white/28">
            {isSearchingSymbols
              ? `${visibleSymbols.length} matches`
              : `${filteredLibraries.length} libraries`}
          </div>
        </div>
        <button
          type="button"
          className="tracer-icon-button ml-auto h-9 w-9"
          aria-label="Reload libraries"
          title="Reload libraries"
          onClick={() => void loadLibraries()}
        >
          ↻
        </button>
      </div>

      <div className="grid gap-2 pb-2">
        <label className="tracer-search flex items-center gap-2 rounded-[18px] px-3 py-2.5">
          <span className="text-[10px] uppercase tracking-[0.22em] text-white/28">Lib</span>
          <input
            value={libraryQuery}
            onChange={(event) => setLibraryQuery(event.target.value)}
            onKeyDown={(event) => event.stopPropagation()}
            placeholder="Filter libraries"
            className="min-w-0 flex-1 bg-transparent text-[13px] text-white outline-none placeholder:text-white/24"
          />
          {libraryQuery ? (
            <button
              type="button"
              className="text-[11px] text-white/38 transition hover:text-white/70"
              onClick={() => setLibraryQuery("")}
            >
              ✕
            </button>
          ) : null}
        </label>

        <label className="tracer-search flex items-center gap-2 rounded-[18px] px-3 py-2.5">
          <span className="text-[10px] uppercase tracking-[0.22em] text-white/28">Sym</span>
          <input
            value={symbolQuery}
            onChange={(event) => setSymbolQuery(event.target.value)}
            onKeyDown={(event) => event.stopPropagation()}
            placeholder={isFilteringLibraries ? `Search ${filteredLibraries.length} libs` : "Search all symbols"}
            className="min-w-0 flex-1 bg-transparent text-[13px] text-white outline-none placeholder:text-white/24"
          />
          {symbolQuery ? (
            <button
              type="button"
              className="text-[11px] text-white/38 transition hover:text-white/70"
              onClick={() => setSymbolQuery("")}
            >
              ✕
            </button>
          ) : null}
        </label>
      </div>

      <div className="grid min-h-0 flex-1 grid-rows-[minmax(128px,0.48fr)_minmax(0,1fr)] gap-2 overflow-hidden">
        <section className="flex min-h-0 flex-col overflow-hidden rounded-[22px] border border-white/7 bg-black/10 p-2.5">
          <div className="mb-2 flex items-center justify-between gap-2 px-1">
            <span className="text-[10px] uppercase tracking-[0.22em] text-white/34">Libraries</span>
            <span className="text-[11px] text-white/36">{loadingLibraries ? "…" : filteredLibraries.length}</span>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto pr-0.5">
            {filteredLibraries.length === 0 ? (
              <EmptyState message="No libraries" />
            ) : (
              <div className="flex flex-col gap-1">
                {filteredLibraries.map((library) => {
                  const selected = library.lib_path === selectedLibraryPath;
                  return (
                    <button
                      key={library.lib_path}
                      type="button"
                      onClick={() => setSelectedLibraryPath(library.lib_path)}
                      className={[
                        "flex min-w-0 items-center gap-2 overflow-hidden rounded-[16px] border px-2.5 py-2 text-left transition",
                        selected
                          ? "border-[#8bd5ff]/30 bg-[#8bd5ff]/10 text-white"
                          : "border-transparent bg-white/[0.02] text-white/72 hover:border-white/8 hover:bg-white/[0.05]",
                      ].join(" ")}
                    >
                      <span
                        className={[
                          "h-2 w-2 flex-shrink-0 rounded-full",
                          selected ? "bg-[#8bd5ff]" : "bg-white/14",
                        ].join(" ")}
                      />
                      <span className="truncate text-[13px]">{library.name}</span>
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </section>

        <section className="flex min-h-0 flex-col overflow-hidden rounded-[22px] border border-white/7 bg-black/10 p-2.5">
          <div className="mb-2 flex items-center justify-between gap-2 px-1">
            <span className="text-[10px] uppercase tracking-[0.22em] text-white/34">
              {isSearchingSymbols ? "Results" : "Symbols"}
            </span>
            <span className="max-w-[11rem] truncate text-[11px] text-white/36">
              {loadingSearch || loadingSymbols
                ? "…"
                : isSearchingSymbols
                  ? isFilteringLibraries
                    ? `${filteredLibraries.length} libs`
                    : `${visibleSymbols.length}`
                  : selectedLibrary?.name ?? ""}
            </span>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto pr-0.5">
            {visibleSymbols.length === 0 ? (
              <EmptyState
                message={
                  isSearchingSymbols
                    ? isFilteringLibraries
                      ? "No matches in filtered libs"
                      : "No matches"
                    : selectedLibraryPath
                      ? "No symbols"
                      : "Select a library"
                }
              />
            ) : (
              <div className="flex flex-col gap-1">
                {visibleSymbols.map((entry) => (
                  <SymbolRow
                    key={`${entry.lib_path}:${entry.symbol_name}`}
                    entry={entry}
                    active={activeSymbol?.lib_path === entry.lib_path && activeSymbol?.symbol_name === entry.symbol_name}
                    showLibraryName={isSearchingSymbols}
                    onDragStart={handleDrag}
                    onDragEnd={onDragEnd}
                    onPlaceRequest={onPlaceRequest}
                  />
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </aside>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex h-full min-h-[112px] items-center justify-center rounded-[16px] border border-dashed border-white/8 bg-white/[0.02] px-4 text-center text-[12px] text-white/38">
      {message}
    </div>
  );
}

function SymbolRow({
  entry,
  active,
  showLibraryName,
  onDragStart,
  onDragEnd,
  onPlaceRequest,
}: {
  entry: LibraryEntry;
  active: boolean;
  showLibraryName: boolean;
  onDragStart: (event: React.DragEvent, entry: LibraryEntry) => void;
  onDragEnd: () => void;
  onPlaceRequest: (entry: LibraryEntry) => void;
}) {
  const secondaryLabel = showLibraryName ? libraryLabelFromPath(entry.lib_path) : null;

  return (
    <div
      role="button"
      tabIndex={0}
      draggable
      onClick={() => onPlaceRequest(entry)}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onPlaceRequest(entry);
        }
      }}
      onDragStart={(event) => onDragStart(event, entry)}
      onDragEnd={onDragEnd}
      className={[
        "group flex min-w-0 cursor-grab items-center gap-2 overflow-hidden rounded-[16px] border px-2.5 py-2.5 transition focus:outline-none focus:ring-1 focus:ring-[#8bd5ff]/25 active:cursor-grabbing",
        active
          ? "border-[#8bd5ff]/30 bg-[#8bd5ff]/10"
          : "border-transparent bg-white/[0.02] hover:border-white/8 hover:bg-white/[0.055]",
      ].join(" ")}
    >
      <div className="min-w-0 flex-1 text-left">
        <div className="truncate text-[13px] font-medium text-white/88">{entry.name}</div>
        {secondaryLabel ? (
          <div className="truncate text-[11px] text-white/32">{secondaryLabel}</div>
        ) : null}
      </div>

      {active ? (
        <span className="rounded-full border border-[#8bd5ff]/22 bg-[#8bd5ff]/10 px-2 py-0.5 text-[10px] text-[#8bd5ff]">
          Armed
        </span>
      ) : (
        <div className="flex flex-shrink-0 items-center pl-1 text-[12px] text-white/18 transition group-hover:text-white/40">
          ⠿
        </div>
      )}
    </div>
  );
}
