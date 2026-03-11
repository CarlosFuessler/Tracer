// TypeScript DTOs mirroring the Rust backend structs

export type ToolKind = "select" | "wire" | "label" | "place" | "move";

export interface Point2D {
  x: number;
  y: number;
}

export interface WireDto {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface SymbolPin {
  name: string;
  number: string;
  position: Point2D;
  direction: "Right" | "Left" | "Up" | "Down";
  length: number;
}

export interface SymbolRect {
  start: Point2D;
  end: Point2D;
}

export interface SymbolPolyline {
  points: Point2D[];
}

export interface SymbolCircle {
  center: Point2D;
  radius: number;
}

export interface SymbolGraphics {
  pins: SymbolPin[];
  rectangles: SymbolRect[];
  polylines: SymbolPolyline[];
  circles: SymbolCircle[];
  reference: string;
  value: string;
}

export interface ObjectDto {
  id: number;
  kind: "Symbol" | "Wire" | "Label" | "Junction";
  name: string;
  x: number;
  y: number;
  wire: WireDto | null;
  graphics: SymbolGraphics | null;
  selected: boolean;
}

export interface DocumentDto {
  objects: ObjectDto[];
  can_undo: boolean;
  can_redo: boolean;
}

export interface LibraryEntry {
  name: string;
  lib_path: string;
  symbol_name: string;
  kind: "symbol" | "footprint";
}

export interface Viewport {
  pan: Point2D;
  zoom: number; // px per mm
}
