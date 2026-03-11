import { ToolKind, Viewport } from "../types";

interface Props {
  status: string;
  objectCount: number;
  selectedCount: number;
  tool: ToolKind;
  viewport: Viewport;
}

export default function StatusBar({ status, objectCount, selectedCount, tool, viewport }: Props) {
  return (
    <footer className="flex items-center justify-between px-4 h-6 bg-surface border-t border-border flex-shrink-0 text-xs text-gray-500">
      <span className="truncate max-w-md">{status}</span>
      <div className="flex items-center gap-3 flex-shrink-0">
        <span className="bg-elevated px-2 py-0.5 rounded">Tool: {tool}</span>
        <span className="bg-elevated px-2 py-0.5 rounded">
          {Math.round((viewport.zoom / 8) * 100)}%
        </span>
        <span className="bg-elevated px-2 py-0.5 rounded">Objects: {objectCount}</span>
        {selectedCount > 0 && (
          <span className="bg-accent-muted text-accent px-2 py-0.5 rounded">
            Selected: {selectedCount}
          </span>
        )}
      </div>
    </footer>
  );
}
