import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DocumentDto, LibraryEntry, ToolKind, Viewport } from "./types";
import Canvas from "./components/Canvas";
import LibraryBrowser from "./components/LibraryBrowser";
import ToolRail from "./components/ToolRail";
import StatusBar from "./components/StatusBar";

const INITIAL_VIEWPORT: Viewport = { pan: { x: 0, y: 0 }, zoom: 8 };

export default function App() {
  const [doc, setDoc] = useState<DocumentDto>({ objects: [], can_undo: false, can_redo: false });
  const [tool, setTool] = useState<ToolKind>("select");
  const [viewport, setViewport] = useState<Viewport>(INITIAL_VIEWPORT);
  const [status, setStatus] = useState("Ready — search components and drag onto canvas");
  const [_pendingDrop, setPendingDrop] = useState<LibraryEntry | null>(null);

  const handleSymbolDropped = useCallback(
    async (entry: LibraryEntry, x: number, y: number) => {
      try {
        const d = await invoke<DocumentDto>("place_symbol", {
          libPath: entry.lib_path,
          symbolName: entry.symbol_name,
          x,
          y,
        });
        setDoc(d);
        setStatus(`Placed ${entry.name}`);
      } catch (e) {
        setStatus(`Error: ${e}`);
      }
    },
    []
  );

  const handleWirePlaced = useCallback(
    async (x1: number, y1: number, x2: number, y2: number) => {
      try {
        const d = await invoke<DocumentDto>("place_wire", { x1, y1, x2, y2 });
        setDoc(d);
        setStatus("Wire placed");
      } catch (e) {
        setStatus(`Error: ${e}`);
      }
    },
    []
  );

  const handleUndo = useCallback(async () => {
    if (!doc.can_undo) return;
    const d = await invoke<DocumentDto>("undo");
    setDoc(d);
    setStatus("Undo");
  }, [doc.can_undo]);

  const handleRedo = useCallback(async () => {
    if (!doc.can_redo) return;
    const d = await invoke<DocumentDto>("redo");
    setDoc(d);
    setStatus("Redo");
  }, [doc.can_redo]);

  const handleDelete = useCallback(async () => {
    const selected = doc.objects.filter((o) => o.selected).map((o) => o.id);
    if (selected.length === 0) return;
    const d = await invoke<DocumentDto>("delete_objects", { ids: selected });
    setDoc(d);
    setStatus(`Deleted ${selected.length} object(s)`);
  }, [doc.objects]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        if (e.key === "z" && !e.shiftKey) { handleUndo(); return; }
        if (e.key === "z" && e.shiftKey) { handleRedo(); return; }
        if (e.key === "y") { handleRedo(); return; }
      }
      if (e.key === "Backspace" || e.key === "Delete") { handleDelete(); return; }
      if (e.key === "Escape") { setTool("select"); return; }
      switch (e.key.toLowerCase()) {
        case "v": setTool("select"); break;
        case "w": setTool("wire"); break;
        case "l": setTool("label"); break;
        case "p": setTool("place"); break;
        case "m": setTool("move"); break;
      }
    },
    [handleUndo, handleRedo, handleDelete]
  );

  return (
    <div
      className="flex flex-col w-full h-full bg-canvas text-white"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      style={{ outline: "none" }}
    >
      {/* Top bar */}
      <header className="flex items-center justify-between px-4 h-10 bg-surface border-b border-border flex-shrink-0">
        <span className="text-accent font-bold text-sm">⬡ Tracer</span>
        <div className="flex items-center gap-2">
          <button
            className="px-3 py-1 rounded text-xs text-gray-400 hover:bg-elevated hover:text-white transition-colors"
            onClick={() => invoke("get_document").then((d) => setDoc(d as DocumentDto))}
          >
            🔄 Refresh
          </button>
          <button
            disabled={!doc.can_undo}
            className="px-3 py-1 rounded text-xs text-gray-400 hover:bg-elevated hover:text-white transition-colors disabled:opacity-30"
            onClick={handleUndo}
          >
            ↩ Undo
          </button>
          <button
            disabled={!doc.can_redo}
            className="px-3 py-1 rounded text-xs text-gray-400 hover:bg-elevated hover:text-white transition-colors disabled:opacity-30"
            onClick={handleRedo}
          >
            ↪ Redo
          </button>
        </div>
      </header>

      {/* Main area */}
      <div className="flex flex-1 overflow-hidden">
        <ToolRail tool={tool} onToolChange={setTool} />
        <Canvas
          doc={doc}
          tool={tool}
          viewport={viewport}
          onViewportChange={setViewport}
          onPendingDropClear={() => setPendingDrop(null)}
          onSymbolDropped={handleSymbolDropped}
          onWirePlaced={handleWirePlaced}
          onDocChange={setDoc}
          onStatusChange={setStatus}
        />
        <LibraryBrowser
          onDragStart={setPendingDrop}
          onStatusChange={setStatus}
        />
      </div>

      <StatusBar
        status={status}
        objectCount={doc.objects.length}
        selectedCount={doc.objects.filter((o) => o.selected).length}
        tool={tool}
        viewport={viewport}
      />
    </div>
  );
}
