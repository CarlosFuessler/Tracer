import { LibraryEntry, ToolKind } from "../types";

const TOOLS: { kind: ToolKind; icon: string; label: string; key: string }[] = [
  { kind: "select", icon: "↖", label: "Select", key: "V" },
  { kind: "wire", icon: "╱", label: "Wire", key: "W" },
  { kind: "label", icon: "A", label: "Label", key: "L" },
  { kind: "place", icon: "◫", label: "Place", key: "P" },
  { kind: "move", icon: "✥", label: "Move", key: "M" },
];

interface Props {
  tool: ToolKind;
  onToolChange: (tool: ToolKind) => void;
  armedSymbol: LibraryEntry | null;
}

export default function ToolRail({ tool, onToolChange, armedSymbol }: Props) {
  return (
    <aside className="tracer-panel flex h-full min-h-0 flex-col items-center gap-2 rounded-[24px] px-2 py-2.5">
      <div className="flex flex-col gap-1.5">
        {TOOLS.map((item) => {
          const active = tool === item.kind;
          return (
            <button
              key={item.kind}
              type="button"
              aria-label={`${item.label} (${item.key})`}
              title={`${item.label} (${item.key})`}
              onClick={() => onToolChange(item.kind)}
              className={[
                "group relative flex h-[54px] w-[54px] items-center justify-center rounded-[18px] border transition-all duration-150",
                active
                  ? "border-[#8bd5ff]/70 bg-[#162432] text-[#8bd5ff] shadow-[0_0_0_1px_rgba(139,213,255,0.15)_inset]"
                  : "border-white/8 bg-white/4 text-white/55 hover:border-white/16 hover:bg-white/7 hover:text-white/90",
              ].join(" ")}
            >
              <span className="text-[18px] leading-none">{item.icon}</span>
              <span className="pointer-events-none absolute bottom-1.5 right-1.5 text-[8px] uppercase tracking-wide text-white/28 transition group-hover:text-white/45">
                {item.key}
              </span>
            </button>
          );
        })}
      </div>

      {armedSymbol ? (
        <div
          className="mt-auto flex w-full items-center justify-center rounded-[16px] border border-[#8bd5ff]/18 bg-[#8bd5ff]/8 px-2 py-2 text-[11px] text-[#8bd5ff]"
          title={armedSymbol.name}
        >
          <span className="truncate">{armedSymbol.name}</span>
        </div>
      ) : (
        <div className="mt-auto h-8 w-full" />
      )}
    </aside>
  );
}
