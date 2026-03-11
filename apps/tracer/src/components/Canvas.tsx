import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  DocumentDto,
  LibraryEntry,
  ObjectDto,
  Point2D,
  SymbolGraphics,
  ToolKind,
  Viewport,
} from "../types";

const GRID_MM = 1.27;

interface Props {
  doc: DocumentDto;
  tool: ToolKind;
  viewport: Viewport;
  pendingDrop: LibraryEntry | null;
  armedSymbol: LibraryEntry | null;
  onViewportChange: (viewport: Viewport) => void;
  onDropStateClear: () => void;
  onArmedSymbolClear: () => void;
  onSymbolPlaced: (entry: LibraryEntry, x: number, y: number) => Promise<void> | void;
  onWirePlaced: (x1: number, y1: number, x2: number, y2: number) => Promise<void> | void;
  onDocChange: (doc: DocumentDto) => void;
  onStatusChange: (message: string) => void;
}

export default function Canvas({
  doc,
  tool,
  viewport,
  pendingDrop,
  armedSymbol,
  onViewportChange,
  onDropStateClear,
  onArmedSymbolClear,
  onSymbolPlaced,
  onWirePlaced,
  onDocChange,
  onStatusChange,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dragDepth = useRef(0);
  const panStart = useRef<{ mx: number; my: number; px: number; py: number } | null>(null);
  const wireStart = useRef<Point2D | null>(null);
  const mouseSchematic = useRef<Point2D>({ x: 0, y: 0 });
  const [isDragOver, setIsDragOver] = useState(false);
  const [renderTick, setRenderTick] = useState(0);

  const activePlacement = armedSymbol ?? pendingDrop;

  const syncCanvasSize = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    if (width === 0 || height === 0) {
      return;
    }
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    const context = canvas.getContext("2d");
    if (context) {
      context.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
    setRenderTick((value) => value + 1);
  }, []);

  const toSchematic = useCallback(
    (clientX: number, clientY: number): Point2D => {
      const canvas = canvasRef.current;
      if (!canvas) {
        return { x: 0, y: 0 };
      }
      const rect = canvas.getBoundingClientRect();
      const cx = rect.width / 2;
      const cy = rect.height / 2;
      return {
        x: (clientX - rect.left - cx) / viewport.zoom + viewport.pan.x,
        y: (clientY - rect.top - cy) / viewport.zoom + viewport.pan.y,
      };
    },
    [viewport.pan.x, viewport.pan.y, viewport.zoom]
  );

  const snap = useCallback(
    (point: Point2D): Point2D => ({
      x: Math.round(point.x / GRID_MM) * GRID_MM,
      y: Math.round(point.y / GRID_MM) * GRID_MM,
    }),
    []
  );

  useEffect(() => {
    syncCanvasSize();
  }, [syncCanvasSize]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const observer = new ResizeObserver(() => syncCanvasSize());
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [syncCanvasSize]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    const cx = width / 2;
    const cy = height / 2;
    const { pan, zoom } = viewport;

    const toScreen = (x: number, y: number): [number, number] => [
      (x - pan.x) * zoom + cx,
      (y - pan.y) * zoom + cy,
    ];

    ctx.clearRect(0, 0, width, height);

    const background = ctx.createLinearGradient(0, 0, width, height);
    background.addColorStop(0, "#0b1016");
    background.addColorStop(1, "#07090d");
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, width, height);

    drawGrid(ctx, width, height, viewport, toScreen);

    for (const object of doc.objects) {
      const [sx, sy] = toScreen(object.x, object.y);
      if (sx < -320 || sx > width + 320 || sy < -320 || sy > height + 320) {
        continue;
      }

      if (object.kind === "Wire" && object.wire) {
        const [x1, y1] = toScreen(object.wire.x1, object.wire.y1);
        const [x2, y2] = toScreen(object.wire.x2, object.wire.y2);
        ctx.strokeStyle = object.selected ? "#fbbf24" : "#5eead4";
        ctx.lineWidth = object.selected ? 2.6 : 1.7;
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
        continue;
      }

      if (object.kind === "Symbol") {
        drawSymbol(ctx, object, toScreen, zoom);
        continue;
      }

      if (object.kind === "Label") {
        ctx.fillStyle = object.selected ? "#fbbf24" : "#8bd5ff";
        ctx.font = `${Math.max(10, zoom * 0.8)}px Inter, sans-serif`;
        ctx.fillText(object.name, sx + 4, sy - 6);
        continue;
      }

      if (object.kind === "Junction") {
        ctx.fillStyle = "#5eead4";
        ctx.beginPath();
        ctx.arc(sx, sy, Math.max(2.5, zoom * 0.22), 0, Math.PI * 2);
        ctx.fill();
      }
    }

    if (wireStart.current && tool === "wire") {
      const start = wireStart.current;
      const current = snap(mouseSchematic.current);
      const [x1, y1] = toScreen(start.x, start.y);
      const [x2, y2] = toScreen(current.x, current.y);
      ctx.strokeStyle = "rgba(94, 234, 212, 0.7)";
      ctx.lineWidth = 1.4;
      ctx.setLineDash([6, 6]);
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = "#5eead4";
      ctx.beginPath();
      ctx.arc(x1, y1, 3, 0, Math.PI * 2);
      ctx.fill();
    }

    const mouse = mouseSchematic.current;
    ctx.fillStyle = "rgba(13, 17, 23, 0.88)";
    ctx.fillRect(16, height - 42, 182, 26);
    ctx.strokeStyle = "rgba(255,255,255,0.08)";
    ctx.strokeRect(16, height - 42, 182, 26);
    ctx.fillStyle = "rgba(255,255,255,0.68)";
    ctx.font = "11px JetBrains Mono, monospace";
    ctx.fillText(`X ${mouse.x.toFixed(2)}  Y ${mouse.y.toFixed(2)} mm`, 26, height - 25);

    if (isDragOver) {
      ctx.fillStyle = "rgba(139, 213, 255, 0.09)";
      ctx.fillRect(0, 0, width, height);
      ctx.strokeStyle = "rgba(139, 213, 255, 0.55)";
      ctx.setLineDash([12, 10]);
      ctx.lineWidth = 2;
      ctx.strokeRect(18, 18, width - 36, height - 36);
      ctx.setLineDash([]);
    }
  }, [doc, isDragOver, renderTick, snap, tool, viewport]);

  const handleMouseDown = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      if (event.button === 1 || (event.button === 0 && event.altKey)) {
        panStart.current = {
          mx: event.clientX,
          my: event.clientY,
          px: viewport.pan.x,
          py: viewport.pan.y,
        };
        return;
      }

      if (event.button !== 0) {
        return;
      }

      const point = snap(toSchematic(event.clientX, event.clientY));

      if (tool === "wire") {
        if (!wireStart.current) {
          wireStart.current = point;
          onStatusChange(`Wire start ${point.x.toFixed(2)}, ${point.y.toFixed(2)}`);
        } else {
          void onWirePlaced(wireStart.current.x, wireStart.current.y, point.x, point.y);
          wireStart.current = null;
          onStatusChange("Wire committed");
        }
        setRenderTick((value) => value + 1);
        return;
      }

      if (tool === "place" && activePlacement) {
        void onSymbolPlaced(activePlacement, point.x, point.y);
        setRenderTick((value) => value + 1);
        return;
      }

      if (tool !== "select") {
        return;
      }

      const canvas = canvasRef.current;
      if (!canvas) {
        return;
      }
      const rect = canvas.getBoundingClientRect();
      const mouseX = event.clientX - rect.left;
      const mouseY = event.clientY - rect.top;
      const cx = rect.width / 2;
      const cy = rect.height / 2;

      let hitId: number | null = null;
      for (const object of [...doc.objects].reverse()) {
        const bounds = screenBoundsForObject(object, viewport, cx, cy);
        if (
          mouseX >= bounds.minX &&
          mouseX <= bounds.maxX &&
          mouseY >= bounds.minY &&
          mouseY <= bounds.maxY
        ) {
          hitId = object.id;
          break;
        }
      }

      invoke<DocumentDto>("select_objects", { ids: hitId ? [hitId] : [] })
        .then(onDocChange)
        .catch((error) => onStatusChange(`Selection failed: ${error}`));
    },
    [activePlacement, doc.objects, onArmedSymbolClear, onDocChange, onStatusChange, onSymbolPlaced, onWirePlaced, snap, toSchematic, tool, viewport]
  );

  const handleMouseMove = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      mouseSchematic.current = toSchematic(event.clientX, event.clientY);

      if (panStart.current) {
        const dx = (event.clientX - panStart.current.mx) / viewport.zoom;
        const dy = (event.clientY - panStart.current.my) / viewport.zoom;
        onViewportChange({
          ...viewport,
          pan: {
            x: panStart.current.px - dx,
            y: panStart.current.py - dy,
          },
        });
      }

      setRenderTick((value) => value + 1);
    },
    [onViewportChange, toSchematic, viewport]
  );

  const handleMouseUp = useCallback(() => {
    panStart.current = null;
  }, []);

  const handleWheel = useCallback(
    (event: React.WheelEvent<HTMLCanvasElement>) => {
      event.preventDefault();
      const canvas = canvasRef.current;
      if (!canvas) {
        return;
      }
      const rect = canvas.getBoundingClientRect();
      const cx = rect.width / 2;
      const cy = rect.height / 2;
      const mouseX = event.clientX - rect.left;
      const mouseY = event.clientY - rect.top;
      const nextZoom = Math.max(1.4, Math.min(120, viewport.zoom * (event.deltaY < 0 ? 1.1 : 0.9)));
      const ratio = 1 / viewport.zoom - 1 / nextZoom;
      onViewportChange({
        zoom: nextZoom,
        pan: {
          x: viewport.pan.x + (mouseX - cx) * ratio,
          y: viewport.pan.y + (mouseY - cy) * ratio,
        },
      });
    },
    [onViewportChange, viewport]
  );

  const parseDroppedEntry = useCallback(
    (event: React.DragEvent<HTMLCanvasElement>): LibraryEntry | null => {
      const payload = event.dataTransfer.getData("application/json") || event.dataTransfer.getData("text/plain");
      if (payload) {
        try {
          return JSON.parse(payload) as LibraryEntry;
        } catch {
          return pendingDrop;
        }
      }
      return pendingDrop;
    },
    [pendingDrop]
  );

  const handleDrop = useCallback(
    (event: React.DragEvent<HTMLCanvasElement>) => {
      event.preventDefault();
      dragDepth.current = 0;
      setIsDragOver(false);
      const entry = parseDroppedEntry(event);
      if (!entry || !entry.symbol_name) {
        onStatusChange("Pick a symbol row");
        onDropStateClear();
        return;
      }
      const point = snap(toSchematic(event.clientX, event.clientY));
      void onSymbolPlaced(entry, point.x, point.y);
      onDropStateClear();
    },
    [onDropStateClear, onStatusChange, onSymbolPlaced, parseDroppedEntry, snap, toSchematic]
  );

  const overlayText = activePlacement
    ? `Place ${activePlacement.name}`
    : `${tool.charAt(0).toUpperCase()}${tool.slice(1)}`;

  return (
    <section className="tracer-panel relative flex min-w-0 flex-1 flex-col overflow-hidden rounded-[24px] p-1.5">
      <div className="pointer-events-none absolute left-[5.5rem] top-4 z-10 max-w-[18rem]">
        <div
          className={[
            "truncate rounded-full border bg-[#0c1219]/82 px-3 py-1.5 text-[11px] backdrop-blur-xl",
            activePlacement ? "border-[#8bd5ff]/20 text-[#8bd5ff]" : "border-white/10 text-white/58",
          ].join(" ")}
        >
          {overlayText}
        </div>
      </div>

      <canvas
        ref={canvasRef}
        className="h-full w-full rounded-[20px] bg-transparent"
        style={{
          cursor: isDragOver ? "copy" : tool === "wire" ? "crosshair" : tool === "place" ? "cell" : panStart.current ? "grabbing" : "default",
          touchAction: "none",
        }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
        onDragEnter={(event) => {
          event.preventDefault();
          dragDepth.current += 1;
          setIsDragOver(true);
        }}
        onDragLeave={(event) => {
          event.preventDefault();
          dragDepth.current = Math.max(0, dragDepth.current - 1);
          if (dragDepth.current === 0) {
            setIsDragOver(false);
          }
        }}
        onDragOver={(event) => {
          event.preventDefault();
          if (!isDragOver) {
            setIsDragOver(true);
          }
        }}
        onDrop={handleDrop}
        onContextMenu={(event) => {
          event.preventDefault();
          wireStart.current = null;
          onArmedSymbolClear();
          onStatusChange("Cancelled");
          setRenderTick((value) => value + 1);
        }}
      />

      {doc.objects.length === 0 ? (
        <div className="pointer-events-none absolute inset-x-0 top-1/2 flex -translate-y-1/2 justify-center px-6">
          <div className="rounded-full border border-white/10 bg-[#0d1117]/82 px-4 py-2 text-[12px] text-white/48 shadow-[0_24px_60px_rgba(0,0,0,0.34)] backdrop-blur-xl">
            Drop or place a symbol
          </div>
        </div>
      ) : null}
    </section>
  );
}

function drawGrid(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  viewport: Viewport,
  toScreen: (x: number, y: number) => [number, number]
) {
  const cx = width / 2;
  const cy = height / 2;
  const gridPx = GRID_MM * viewport.zoom;
  if (gridPx < 7 || gridPx > 120) {
    return;
  }

  const leftMm = viewport.pan.x - cx / viewport.zoom;
  const rightMm = viewport.pan.x + cx / viewport.zoom;
  const topMm = viewport.pan.y - cy / viewport.zoom;
  const bottomMm = viewport.pan.y + cy / viewport.zoom;
  const startX = Math.floor(leftMm / GRID_MM) * GRID_MM;
  const startY = Math.floor(topMm / GRID_MM) * GRID_MM;

  ctx.fillStyle = "rgba(255,255,255,0.12)";
  const radius = Math.max(0.9, gridPx * 0.05);
  for (let gx = startX; gx <= rightMm + GRID_MM; gx += GRID_MM) {
    for (let gy = startY; gy <= bottomMm + GRID_MM; gy += GRID_MM) {
      const [sx, sy] = toScreen(gx, gy);
      ctx.beginPath();
      ctx.arc(sx, sy, radius, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

function drawSymbol(
  ctx: CanvasRenderingContext2D,
  object: ObjectDto,
  toScreen: (x: number, y: number) => [number, number],
  zoom: number
) {
  const graphics = object.graphics;
  const [originX, originY] = toScreen(object.x, object.y);
  const selected = object.selected;

  if (!graphics) {
    ctx.strokeStyle = selected ? "#fbbf24" : "#8bd5ff";
    ctx.lineWidth = selected ? 2 : 1.3;
    ctx.strokeRect(originX - 14, originY - 12, 28, 24);
    ctx.fillStyle = "rgba(255,255,255,0.72)";
    ctx.font = "10px Inter, sans-serif";
    ctx.fillText(object.name.slice(0, 10), originX - 10, originY + 4);
    return;
  }

  const localX = (value: number) => originX + value * zoom;
  const localY = (value: number) => originY + value * zoom;
  const stroke = selected ? "#fbbf24" : "#8bd5ff";

  for (const rect of graphics.rectangles) {
    const x = Math.min(rect.start.x, rect.end.x);
    const y = Math.min(rect.start.y, rect.end.y);
    const width = Math.abs(rect.end.x - rect.start.x) * zoom;
    const height = Math.abs(rect.end.y - rect.start.y) * zoom;
    ctx.fillStyle = "rgba(18, 26, 36, 0.85)";
    ctx.fillRect(localX(x), localY(y), width, height);
    ctx.strokeStyle = stroke;
    ctx.lineWidth = selected ? 2.2 : 1.4;
    ctx.strokeRect(localX(x), localY(y), width, height);
  }

  ctx.strokeStyle = stroke;
  ctx.lineWidth = selected ? 2.2 : 1.4;
  for (const polyline of graphics.polylines) {
    if (polyline.points.length < 2) {
      continue;
    }
    ctx.beginPath();
    ctx.moveTo(localX(polyline.points[0].x), localY(polyline.points[0].y));
    for (let index = 1; index < polyline.points.length; index += 1) {
      ctx.lineTo(localX(polyline.points[index].x), localY(polyline.points[index].y));
    }
    ctx.stroke();
  }

  for (const circle of graphics.circles) {
    ctx.beginPath();
    ctx.arc(localX(circle.center.x), localY(circle.center.y), circle.radius * zoom, 0, Math.PI * 2);
    ctx.stroke();
  }

  if (zoom >= 4) {
    for (const pin of graphics.pins) {
      const stub = pinStubEnd(pin.position, pin.direction, pin.length);
      ctx.strokeStyle = "#5eead4";
      ctx.lineWidth = 1.05;
      ctx.beginPath();
      ctx.moveTo(localX(pin.position.x), localY(pin.position.y));
      ctx.lineTo(localX(stub.x), localY(stub.y));
      ctx.stroke();
      ctx.fillStyle = "#5eead4";
      ctx.beginPath();
      ctx.arc(localX(pin.position.x), localY(pin.position.y), 2.1, 0, Math.PI * 2);
      ctx.fill();
      if (zoom >= 6 && pin.number) {
        ctx.fillStyle = "rgba(255,255,255,0.38)";
        ctx.font = `${Math.max(8, zoom * 0.6)}px JetBrains Mono, monospace`;
        ctx.fillText(pin.number, localX(stub.x) + 2, localY(stub.y) + 3);
      }
    }
  }

  if (zoom >= 5) {
    ctx.fillStyle = selected ? "#fbbf24" : "rgba(255,255,255,0.64)";
    ctx.font = `${Math.max(10, zoom * 0.7)}px Inter, sans-serif`;
    const label = graphics.reference ? `${graphics.reference}?` : object.name.slice(0, 12);
    ctx.fillText(label, originX + 4, originY - 8);
  }
}

function pinStubEnd(position: Point2D, direction: string, length: number): Point2D {
  switch (direction) {
    case "Right":
      return { x: position.x + length, y: position.y };
    case "Left":
      return { x: position.x - length, y: position.y };
    case "Up":
      return { x: position.x, y: position.y - length };
    case "Down":
      return { x: position.x, y: position.y + length };
    default:
      return position;
  }
}

function graphicsBounds(graphics: SymbolGraphics) {
  const xs: number[] = [];
  const ys: number[] = [];

  for (const rect of graphics.rectangles) {
    xs.push(rect.start.x, rect.end.x);
    ys.push(rect.start.y, rect.end.y);
  }

  for (const polyline of graphics.polylines) {
    for (const point of polyline.points) {
      xs.push(point.x);
      ys.push(point.y);
    }
  }

  for (const circle of graphics.circles) {
    xs.push(circle.center.x - circle.radius, circle.center.x + circle.radius);
    ys.push(circle.center.y - circle.radius, circle.center.y + circle.radius);
  }

  for (const pin of graphics.pins) {
    const stub = pinStubEnd(pin.position, pin.direction, pin.length);
    xs.push(pin.position.x, stub.x);
    ys.push(pin.position.y, stub.y);
  }

  if (xs.length === 0 || ys.length === 0) {
    return { minX: -4, maxX: 4, minY: -4, maxY: 4 };
  }

  return {
    minX: Math.min(...xs),
    maxX: Math.max(...xs),
    minY: Math.min(...ys),
    maxY: Math.max(...ys),
  };
}

function screenBoundsForObject(object: ObjectDto, viewport: Viewport, cx: number, cy: number) {
  const toScreen = (x: number, y: number) => ({
    x: (x - viewport.pan.x) * viewport.zoom + cx,
    y: (y - viewport.pan.y) * viewport.zoom + cy,
  });

  if (object.kind === "Wire" && object.wire) {
    const start = toScreen(object.wire.x1, object.wire.y1);
    const end = toScreen(object.wire.x2, object.wire.y2);
    return {
      minX: Math.min(start.x, end.x) - 6,
      maxX: Math.max(start.x, end.x) + 6,
      minY: Math.min(start.y, end.y) - 6,
      maxY: Math.max(start.y, end.y) + 6,
    };
  }

  if (object.kind === "Symbol" && object.graphics) {
    const bounds = graphicsBounds(object.graphics);
    const topLeft = toScreen(object.x + bounds.minX, object.y + bounds.minY);
    const bottomRight = toScreen(object.x + bounds.maxX, object.y + bounds.maxY);
    return {
      minX: Math.min(topLeft.x, bottomRight.x) - 10,
      maxX: Math.max(topLeft.x, bottomRight.x) + 10,
      minY: Math.min(topLeft.y, bottomRight.y) - 10,
      maxY: Math.max(topLeft.y, bottomRight.y) + 10,
    };
  }

  const center = toScreen(object.x, object.y);
  return {
    minX: center.x - 18,
    maxX: center.x + 18,
    minY: center.y - 18,
    maxY: center.y + 18,
  };
}
