import { ToolKind } from "../types";

const TOOLS: { kind: ToolKind; icon: string; label: string; key: string }[] = [
  { kind: "select", icon: "↖", label: "Select", key: "V" },
  { kind: "wire",   icon: "─", label: "Wire",   key: "W" },
  { kind: "label",  icon: "A", label: "Label",  key: "L" },
  { kind: "place",  icon: "◫", label: "Place",  key: "P" },
  { kind: "move",   icon: "✥", label: "Move",   key: "M" },
];

interface Props {
  tool: ToolKind;
  onToolChange: (t: ToolKind) => void;
}

export default function ToolRail({ tool, onToolChange }: Props) {
  return (
    <aside className="w-14 flex flex-col items-center py-2 gap-1 bg-surface border-r border-border flex-shrink-0">
      {TOOLS.map((t) => (
        <button
          key={t.kind}
          title={`${t.label} (${t.key})`}
          onClick={() => onToolChange(t.kind)}
          className={[
            "w-11 h-11 rounded-xl flex flex-col items-center justify-center gap-0.5 transition-colors text-base",
            tool === t.kind
              ? "bg-accent-muted text-accent border border-accent"
              : "text-gray-400 hover:bg-elevated hover:text-white border border-transparent",
          ].join(" ")}
        >
          <span>{t.icon}</span>
          <span className="text-[9px] text-gray-500">{t.label}</span>
        </button>
      ))}
    </aside>
  );
}
