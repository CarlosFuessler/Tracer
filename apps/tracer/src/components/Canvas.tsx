import { useRef, useEffect, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  DocumentDto, LibraryEntry, ObjectDto,
  ToolKind, Viewport, Point2D,
} from "../types";

const GRID_MM = 1.27;

interface Props {
  doc: DocumentDto;
  tool: ToolKind;
  viewport: Viewport;
  onViewportChange: (v: Viewport) => void;
  onPendingDropClear: () => void;
  onSymbolDropped: (entry: LibraryEntry, x: number, y: number) => void;
  onWirePlaced: (x1: number, y1: number, x2: number, y2: number) => void;
  onDocChange: (d: DocumentDto) => void;
  onStatusChange: (msg: string) => void;
}

export default function Canvas({
  doc, tool, viewport, onViewportChange,
  onPendingDropClear,
  onSymbolDropped, onWirePlaced, onDocChange, onStatusChange,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const panStart = useRef<{ mx: number; my: number; px: number; py: number } | null>(null);
  const wireStart = useRef<Point2D | null>(null);
  const mouseSchematic = useRef<Point2D>({ x: 0, y: 0 });
  const [_tick, setTick] = useState(0); // force re-render for wire preview

  const toSchematic = useCallback(
    (mx: number, my: number): Point2D => {
      const canvas = canvasRef.current;
      if (!canvas) return { x: 0, y: 0 };
      const rect = canvas.getBoundingClientRect();
      const cx = rect.width / 2;
      const cy = rect.height / 2;
      return {
        x: (mx - rect.left - cx) / viewport.zoom + viewport.pan.x,
        y: (my - rect.top - cy) / viewport.zoom + viewport.pan.y,
      };
    },
    [viewport]
  );

  const snap = useCallback(
    (pt: Point2D): Point2D => ({
      x: Math.round(pt.x / GRID_MM) * GRID_MM,
      y: Math.round(pt.y / GRID_MM) * GRID_MM,
    }),
    []
  );

  // ── Draw ──────────────────────────────────────────────
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const W = canvas.width;
    const H = canvas.height;
    const { pan, zoom } = viewport;
    const cx = W / 2;
    const cy = H / 2;

    const toScreen = (x: number, y: number): [number, number] => [
      (x - pan.x) * zoom + cx,
      (y - pan.y) * zoom + cy,
    ];

    ctx.clearRect(0, 0, W, H);
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, W, H);

    // Grid dots
    const gridPx = GRID_MM * zoom;
    if (gridPx >= 6 && gridPx <= 200) {
      const leftMm = pan.x - cx / zoom;
      const rightMm = pan.x + cx / zoom;
      const topMm = pan.y - cy / zoom;
      const botMm = pan.y + cy / zoom;
      const startX = Math.floor(leftMm / GRID_MM) * GRID_MM;
      const startY = Math.floor(topMm / GRID_MM) * GRID_MM;

      ctx.fillStyle = "#2d3748";
      const dotR = Math.max(0.8, gridPx * 0.04);
      for (let gx = startX; gx <= rightMm + GRID_MM; gx += GRID_MM) {
        for (let gy = startY; gy <= botMm + GRID_MM; gy += GRID_MM) {
          const [sx, sy] = toScreen(gx, gy);
          ctx.beginPath();
          ctx.arc(sx, sy, dotR, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    // Objects
    for (const obj of doc.objects) {
      const [sx, sy] = toScreen(obj.x, obj.y);
      // Skip if far outside viewport
      if (sx < -300 || sx > W + 300 || sy < -300 || sy > H + 300) continue;

      if (obj.kind === "Wire" && obj.wire) {
        const [x1, y1] = toScreen(obj.wire.x1, obj.wire.y1);
        const [x2, y2] = toScreen(obj.wire.x2, obj.wire.y2);
        ctx.strokeStyle = obj.selected ? "#f0a500" : "#3fb950";
        ctx.lineWidth = obj.selected ? 2.5 : 1.5;
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
      } else if (obj.kind === "Symbol") {
        drawSymbol(ctx, obj, toScreen, zoom);
      } else if (obj.kind === "Label") {
        ctx.fillStyle = obj.selected ? "#f0a500" : "#58a6ff";
        ctx.font = `${Math.max(10, zoom * 0.8)}px monospace`;
        ctx.fillText(obj.name, sx + 4, sy - 4);
      } else if (obj.kind === "Junction") {
        ctx.fillStyle = "#3fb950";
        ctx.beginPath();
        ctx.arc(sx, sy, Math.max(2, zoom * 0.2), 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // Wire preview
    if (wireStart.current && tool === "wire") {
      const ms = mouseSchematic.current;
      const snapped = snap(ms);
      const [x1, y1] = toScreen(wireStart.current.x, wireStart.current.y);
      const [x2, y2] = toScreen(snapped.x, snapped.y);
      ctx.strokeStyle = "#3fb95088";
      ctx.lineWidth = 1.5;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
      ctx.setLineDash([]);
      // start dot
      ctx.fillStyle = "#3fb950";
      ctx.beginPath();
      ctx.arc(x1, y1, 3, 0, Math.PI * 2);
      ctx.fill();
    }

    // Coordinate overlay
    const ms = mouseSchematic.current;
    ctx.fillStyle = "#ffffff22";
    ctx.fillRect(8, H - 28, 160, 20);
    ctx.fillStyle = "#9ca3af";
    ctx.font = "10px monospace";
    ctx.fillText(
      `X: ${ms.x.toFixed(2)} mm  Y: ${ms.y.toFixed(2)} mm`,
      14,
      H - 13
    );
  });

  // ── Mouse handlers ────────────────────────────────────
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      const canvas = canvasRef.current;
      if (!canvas) return;

      if (e.button === 1 || (e.button === 0 && tool === "select" && e.altKey)) {
        // Middle-click or alt+click = pan
        panStart.current = {
          mx: e.clientX,
          my: e.clientY,
          px: viewport.pan.x,
          py: viewport.pan.y,
        };
        return;
      }

      if (e.button === 0) {
        const pt = snap(toSchematic(e.clientX, e.clientY));

        if (tool === "wire") {
          if (!wireStart.current) {
            wireStart.current = pt;
            onStatusChange(`Wire start: (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
          } else {
            onWirePlaced(wireStart.current.x, wireStart.current.y, pt.x, pt.y);
            wireStart.current = null;
          }
          setTick((t) => t + 1);
          return;
        }

        if (tool === "select") {
          // Find clicked object
          const { pan, zoom } = viewport;
          const canvas2 = canvasRef.current;
          if (!canvas2) return;
          const rect = canvas2.getBoundingClientRect();
          const mx = e.clientX - rect.left;
          const my = e.clientY - rect.top;
          const cx = rect.width / 2;
          const cy = rect.height / 2;

          let hit: number | null = null;
          for (const obj of [...doc.objects].reverse()) {
            const [sx, sy] = [
              (obj.x - pan.x) * zoom + cx,
              (obj.y - pan.y) * zoom + cy,
            ];
            if (Math.hypot(mx - sx, my - sy) < 20) {
              hit = obj.id;
              break;
            }
          }

          invoke<DocumentDto>("select_objects", { ids: hit ? [hit] : [] })
            .then(onDocChange)
            .catch(console.error);
          return;
        }
      }
    },
    [tool, viewport, toSchematic, snap, onWirePlaced, onDocChange, onStatusChange, doc.objects]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      mouseSchematic.current = toSchematic(e.clientX, e.clientY);

      if (panStart.current) {
        const dx = (e.clientX - panStart.current.mx) / viewport.zoom;
        const dy = (e.clientY - panStart.current.my) / viewport.zoom;
        onViewportChange({
          ...viewport,
          pan: { x: panStart.current.px - dx, y: panStart.current.py - dy },
        });
      }

      if (wireStart.current || true) {
        // Redraw for wire preview + coordinate display
        setTick((t) => t + 1);
      }
    },
    [viewport, onViewportChange, toSchematic]
  );

  const handleMouseUp = useCallback(() => {
    panStart.current = null;
  }, []);

  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault();
      const factor = e.deltaY < 0 ? 1.1 : 0.9;
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const cx = rect.width / 2;
      const cy = rect.height / 2;
      const newZoom = Math.max(1, Math.min(200, viewport.zoom * factor));
      const ratio = 1 / viewport.zoom - 1 / newZoom;
      onViewportChange({
        zoom: newZoom,
        pan: {
          x: viewport.pan.x + (mx - cx) * ratio,
          y: viewport.pan.y + (my - cy) * ratio,
        },
      });
    },
    [viewport, onViewportChange]
  );

  // Canvas resize observer
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(() => {
      canvas.width = canvas.offsetWidth;
      canvas.height = canvas.offsetHeight;
      setTick((t) => t + 1);
    });
    observer.observe(canvas);
    return () => observer.disconnect();
  }, []);

  // Drop handler
  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const data = e.dataTransfer.getData("application/json");
      if (!data) return;
      try {
        const entry: LibraryEntry = JSON.parse(data);
        if (!entry.symbol_name) {
          onStatusChange("Drop a specific symbol, not a library folder");
          return;
        }
        const pt = snap(toSchematic(e.clientX, e.clientY));
        onSymbolDropped(entry, pt.x, pt.y);
      } catch {
        onStatusChange("Invalid drop data");
      }
      onPendingDropClear();
    },
    [toSchematic, snap, onSymbolDropped, onPendingDropClear, onStatusChange]
  );

  const cursor =
    tool === "wire" ? "crosshair"
    : tool === "place" ? "cell"
    : panStart.current ? "grabbing"
    : "default";

  return (
    <canvas
      ref={canvasRef}
      className="flex-1 block"
      style={{ cursor, touchAction: "none" }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      onWheel={handleWheel}
      onDrop={handleDrop}
      onDragOver={(e) => e.preventDefault()}
      onContextMenu={(e) => {
        e.preventDefault();
        wireStart.current = null;
        setTick((t) => t + 1);
      }}
    />
  );
}

// ── Symbol rendering ──────────────────────────────────────────

function drawSymbol(
  ctx: CanvasRenderingContext2D,
  obj: ObjectDto,
  toScreen: (x: number, y: number) => [number, number],
  zoom: number
) {
  const g = obj.graphics;
  const [ox, oy] = toScreen(obj.x, obj.y);
  const sel = obj.selected;

  if (!g) {
    // Placeholder box
    ctx.strokeStyle = sel ? "#f0a500" : "#58a6ff";
    ctx.lineWidth = 1;
    ctx.strokeRect(ox - 12, oy - 12, 24, 24);
    ctx.fillStyle = sel ? "#f0a500" : "#9ca3af";
    ctx.font = "9px monospace";
    ctx.fillText(obj.name.slice(0, 8), ox - 10, oy + 3);
    return;
  }

  const scale = zoom; // 1 mm = zoom px

  const sx = (lx: number) => ox + lx * scale;
  const sy = (ly: number) => oy + ly * scale;

  // Rectangles
  for (const r of g.rectangles) {
    const x = Math.min(r.start.x, r.end.x);
    const y = Math.min(r.start.y, r.end.y);
    const w = Math.abs(r.end.x - r.start.x);
    const h = Math.abs(r.end.y - r.start.y);
    ctx.fillStyle = "#1a2332";
    ctx.fillRect(sx(x), sy(y), w * scale, h * scale);
    ctx.strokeStyle = sel ? "#f0a500" : "#58a6ff";
    ctx.lineWidth = sel ? 2 : 1.5;
    ctx.strokeRect(sx(x), sy(y), w * scale, h * scale);
  }

  // Polylines
  ctx.strokeStyle = sel ? "#f0a500" : "#58a6ff";
  ctx.lineWidth = sel ? 2 : 1.5;
  for (const pl of g.polylines) {
    if (pl.points.length < 2) continue;
    ctx.beginPath();
    ctx.moveTo(sx(pl.points[0].x), sy(pl.points[0].y));
    for (let i = 1; i < pl.points.length; i++) {
      ctx.lineTo(sx(pl.points[i].x), sy(pl.points[i].y));
    }
    ctx.stroke();
  }

  // Circles
  for (const c of g.circles) {
    ctx.strokeStyle = sel ? "#f0a500" : "#58a6ff";
    ctx.lineWidth = sel ? 2 : 1.5;
    ctx.beginPath();
    ctx.arc(sx(c.center.x), sy(c.center.y), c.radius * scale, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Pins
  if (zoom >= 4) {
    for (const pin of g.pins) {
      const stub = pinStubEnd(pin);
      ctx.strokeStyle = "#3fb950";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(sx(pin.position.x), sy(pin.position.y));
      ctx.lineTo(sx(stub.x), sy(stub.y));
      ctx.stroke();
      // Connection dot
      ctx.fillStyle = "#3fb950";
      ctx.beginPath();
      ctx.arc(sx(pin.position.x), sy(pin.position.y), 2, 0, Math.PI * 2);
      ctx.fill();
      // Pin number label
      if (zoom >= 6 && pin.number) {
        ctx.fillStyle = "#6b7280";
        ctx.font = `${Math.max(8, zoom * 0.6)}px monospace`;
        ctx.fillText(pin.number, sx(stub.x) + 2, sy(stub.y) + 3);
      }
    }
  }

  // Reference + value text
  if (zoom >= 5) {
    ctx.fillStyle = sel ? "#f0a500" : "#9ca3af";
    ctx.font = `${Math.max(9, zoom * 0.7)}px monospace`;
    const label = g.reference ? `${g.reference}?` : obj.name.slice(0, 12);
    ctx.fillText(label, ox + 4, oy - 6);
  }
}

function pinStubEnd(pin: { position: Point2D; direction: string; length: number }): Point2D {
  const l = pin.length;
  switch (pin.direction) {
    case "Right": return { x: pin.position.x - l, y: pin.position.y };
    case "Left":  return { x: pin.position.x + l, y: pin.position.y };
    case "Up":    return { x: pin.position.x, y: pin.position.y + l };
    case "Down":  return { x: pin.position.x, y: pin.position.y - l };
    default:      return pin.position;
  }
}
