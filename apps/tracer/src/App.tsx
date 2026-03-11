import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DocumentDto, LibraryEntry, ToolKind, Viewport } from "./types";
import Canvas from "./components/Canvas";
import LibraryBrowser from "./components/LibraryBrowser";
import ToolRail from "./components/ToolRail";
import StatusBar from "./components/StatusBar";

const INITIAL_VIEWPORT: Viewport = { pan: { x: 0, y: 0 }, zoom: 8 };

export default function App() {
  const rootRef = useRef<HTMLDivElement>(null);
  const [doc, setDoc] = useState<DocumentDto>({ objects: [], can_undo: false, can_redo: false });
  const [tool, setTool] = useState<ToolKind>("select");
  const [viewport, setViewport] = useState<Viewport>(INITIAL_VIEWPORT);
  const [status, setStatus] = useState("Ready");
  const [dragSymbol, setDragSymbol] = useState<LibraryEntry | null>(null);
  const [armedSymbol, setArmedSymbol] = useState<LibraryEntry | null>(null);
  const [browserOpen, setBrowserOpen] = useState(true);
  const statusDockClass = browserOpen
    ? "absolute bottom-5 left-[6rem] right-[21rem] flex justify-center"
    : "absolute bottom-5 left-[6rem] right-4 flex justify-center";

  useEffect(() => {
    invoke<DocumentDto>("get_document")
      .then(setDoc)
      .catch((error) => setStatus(`Failed to load document: ${error}`));
    rootRef.current?.focus();
  }, []);

  const selectedCount = useMemo(
    () => doc.objects.filter((object) => object.selected).length,
    [doc.objects]
  );

  const handleSymbolPlaced = useCallback(
    async (entry: LibraryEntry, x: number, y: number) => {
      try {
        const updated = await invoke<DocumentDto>("place_symbol", {
          libPath: entry.lib_path,
          symbolName: entry.symbol_name,
          x,
          y,
        });
        setDoc(updated);
        setStatus(`Placed ${entry.name}`);
        setDragSymbol(null);
      } catch (error) {
        setStatus(`Place failed: ${error}`);
      }
    },
    []
  );

  const handleWirePlaced = useCallback(
    async (x1: number, y1: number, x2: number, y2: number) => {
      try {
        const updated = await invoke<DocumentDto>("place_wire", { x1, y1, x2, y2 });
        setDoc(updated);
        setStatus("Wire placed");
      } catch (error) {
        setStatus(`Wire failed: ${error}`);
      }
    },
    []
  );

  const handleUndo = useCallback(async () => {
    if (!doc.can_undo) return;
    const updated = await invoke<DocumentDto>("undo");
    setDoc(updated);
    setStatus("Undo");
  }, [doc.can_undo]);

  const handleRedo = useCallback(async () => {
    if (!doc.can_redo) return;
    const updated = await invoke<DocumentDto>("redo");
    setDoc(updated);
    setStatus("Redo");
  }, [doc.can_redo]);

  const handleDelete = useCallback(async () => {
    const selectedIds = doc.objects.filter((object) => object.selected).map((object) => object.id);
    if (selectedIds.length === 0) {
      return;
    }
    const updated = await invoke<DocumentDto>("delete_objects", { ids: selectedIds });
    setDoc(updated);
    setStatus(`Deleted ${selectedIds.length} object(s)`);
  }, [doc.objects]);

  const handleArmPlacement = useCallback((entry: LibraryEntry) => {
    setArmedSymbol(entry);
    setTool("place");
    setStatus(`Place ${entry.name}`);
  }, []);

  const handleResetView = useCallback(() => {
    setViewport(INITIAL_VIEWPORT);
    setStatus("View reset");
  }, []);

  const toggleBrowser = useCallback(() => {
    setBrowserOpen((open) => !open);
  }, []);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLDivElement>) => {
      const target = event.target as HTMLElement | null;
      const tagName = target?.tagName;
      if (tagName === "INPUT" || tagName === "TEXTAREA" || target?.isContentEditable) {
        return;
      }

      if (event.metaKey || event.ctrlKey) {
        if (event.key.toLowerCase() === "z" && !event.shiftKey) {
          event.preventDefault();
          void handleUndo();
          return;
        }
        if (
          (event.key.toLowerCase() === "z" && event.shiftKey) ||
          event.key.toLowerCase() === "y"
        ) {
          event.preventDefault();
          void handleRedo();
          return;
        }
      }

      if (event.key === "Backspace" || event.key === "Delete") {
        event.preventDefault();
        void handleDelete();
        return;
      }

      if (event.key === "Escape") {
        setArmedSymbol(null);
        setDragSymbol(null);
        setTool("select");
        setStatus("Cancelled");
        return;
      }

      if (event.key.toLowerCase() === "b") {
        event.preventDefault();
        toggleBrowser();
        return;
      }

      switch (event.key.toLowerCase()) {
        case "v":
          setTool("select");
          break;
        case "w":
          setTool("wire");
          break;
        case "l":
          setTool("label");
          break;
        case "p":
          setTool("place");
          break;
        case "m":
          setTool("move");
          break;
        default:
          break;
      }
    },
    [handleDelete, handleRedo, handleUndo, toggleBrowser]
  );

  return (
    <div
      ref={rootRef}
      className="flex h-full w-full flex-col overflow-hidden bg-[#090c11] text-white"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      style={{ outline: "none" }}
    >
      <header className="px-3 pb-2 pt-3">
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 items-center gap-2.5">
            <button
              type="button"
              className="tracer-icon-button"
              aria-label={browserOpen ? "Hide parts drawer" : "Show parts drawer"}
              title={browserOpen ? "Hide parts drawer" : "Show parts drawer"}
              onClick={toggleBrowser}
            >
              ☰
            </button>
            <div className="tracer-panel flex h-10 w-10 shrink-0 items-center justify-center rounded-[16px] border border-white/10 bg-white/5 shadow-[0_12px_28px_rgba(0,0,0,0.32)]">
              <span className="text-[15px] text-[#7dd3fc]">⬡</span>
            </div>
            <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-white/78">
              Tracer
            </div>
          </div>

          <div className="tracer-panel flex items-center gap-1 rounded-full p-1">
            <button className="tracer-button" onClick={handleResetView}>
              Reset
            </button>
            <button className="tracer-button" disabled={!doc.can_undo} onClick={() => void handleUndo()}>
              Undo
            </button>
            <button className="tracer-button" disabled={!doc.can_redo} onClick={() => void handleRedo()}>
              Redo
            </button>
          </div>
        </div>
      </header>

      <main className="relative min-h-0 flex-1 px-3 pb-3 pt-1">
        <Canvas
          doc={doc}
          tool={tool}
          viewport={viewport}
          pendingDrop={dragSymbol}
          armedSymbol={armedSymbol}
          onViewportChange={setViewport}
          onDropStateClear={() => setDragSymbol(null)}
          onArmedSymbolClear={() => setArmedSymbol(null)}
          onSymbolPlaced={handleSymbolPlaced}
          onWirePlaced={handleWirePlaced}
          onDocChange={setDoc}
          onStatusChange={setStatus}
        />

        <div className="pointer-events-none absolute inset-0 z-20">
          <div className="pointer-events-auto absolute bottom-5 left-4 top-4 w-[72px]">
            <ToolRail tool={tool} onToolChange={setTool} armedSymbol={armedSymbol} />
          </div>

          <div className={["pointer-events-none", statusDockClass].join(" ")}>
            <StatusBar
              status={status}
              objectCount={doc.objects.length}
              selectedCount={selectedCount}
              tool={tool}
              viewport={viewport}
              armedSymbolName={armedSymbol?.name ?? null}
            />
          </div>

          <div
            className={[
              "absolute bottom-4 right-4 top-4 w-[320px] transition-all duration-200 ease-out",
              browserOpen
                ? "pointer-events-auto translate-x-0 opacity-100"
                : "pointer-events-none translate-x-8 opacity-0",
            ].join(" ")}
          >
            <LibraryBrowser
              activeSymbol={armedSymbol}
              onDragStart={setDragSymbol}
              onDragEnd={() => setDragSymbol(null)}
              onPlaceRequest={handleArmPlacement}
              onStatusChange={setStatus}
            />
          </div>
        </div>
      </main>
    </div>
  );
}
