import { ToolKind, Viewport } from "../types";

interface Props {
  status: string;
  objectCount: number;
  selectedCount: number;
  tool: ToolKind;
  viewport: Viewport;
  armedSymbolName: string | null;
}

export default function StatusBar({
  status,
  objectCount,
  selectedCount,
  tool,
  viewport,
  armedSymbolName,
}: Props) {
  const zoomLabel = `${Math.round((viewport.zoom / 8) * 100)}%`;
  const armedLabel =
    armedSymbolName && armedSymbolName.length > 18
      ? `${armedSymbolName.slice(0, 16)}…`
      : armedSymbolName;

  return (
    <div className="tracer-panel flex max-w-[min(72rem,calc(100vw-10rem))] flex-wrap items-center gap-2 rounded-full px-2.5 py-2 text-[11px] shadow-[0_18px_45px_rgba(0,0,0,0.36)]">
      <div className="min-w-0 max-w-[18rem] truncate px-1 text-white/60">{status}</div>
      <div className="h-4 w-px bg-white/8" />

      <div className="flex flex-wrap items-center gap-1.5">
        <Chip label={tool} />
        <Chip label={zoomLabel} />
        {objectCount > 0 ? <Chip label={`${objectCount} obj`} /> : null}
        {selectedCount > 0 ? <Chip label={`${selectedCount} sel`} accent /> : null}
        {armedLabel ? <Chip label={armedLabel} accent /> : null}
      </div>
    </div>
  );
}

function Chip({ label, accent = false }: { label: string; accent?: boolean }) {
  return (
    <span
      className={[
        "inline-flex max-w-[10rem] items-center overflow-hidden text-ellipsis whitespace-nowrap rounded-full border px-2.5 py-1 leading-none",
        accent
          ? "border-[#8bd5ff]/22 bg-[#8bd5ff]/12 text-[#8bd5ff]"
          : "border-white/8 bg-white/4 text-white/56",
      ].join(" ")}
    >
      {label}
    </span>
  );
}
