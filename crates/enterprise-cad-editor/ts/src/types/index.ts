/**
 * Enterprise CAD/Vector Editor - Core Type Definitions
 * Comprehensive types for GPU-accelerated CAD system
 */

import { mat3, vec2 } from 'gl-matrix';

// ============================================================================
// Geometric Primitives
// ============================================================================

export interface Point {
  x: number;
  y: number;
}

export interface Vector2D extends Point {
  magnitude(): number;
  normalize(): Vector2D;
  dot(other: Vector2D): number;
  cross(other: Vector2D): number;
}

export interface BoundingBox {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
  center: Point;
}

export interface Circle {
  center: Point;
  radius: number;
}

export interface Rectangle {
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
}

// ============================================================================
// Transform System
// ============================================================================

export interface Transform {
  matrix: mat3;
  position: vec2;
  rotation: number; // radians
  scale: vec2;
  pivot: vec2;

  apply(point: Point): Point;
  inverse(): Transform;
  compose(other: Transform): Transform;
  decompose(): { position: vec2; rotation: number; scale: vec2 };
}

export interface TransformState {
  local: Transform;
  world: Transform;
  parent?: string; // Parent entity ID
}

// ============================================================================
// Path and Shape System
// ============================================================================

export enum PathCommandType {
  MoveTo = 'M',
  LineTo = 'L',
  CurveTo = 'C',
  QuadraticCurveTo = 'Q',
  ArcTo = 'A',
  ClosePath = 'Z'
}

export interface PathCommand {
  type: PathCommandType;
  points: Point[];
  // For arc commands
  radiusX?: number;
  radiusY?: number;
  rotation?: number;
  largeArc?: boolean;
  sweep?: boolean;
}

export interface Path {
  id: string;
  commands: PathCommand[];
  closed: boolean;
  fillRule: 'nonzero' | 'evenodd';

  getBounds(): BoundingBox;
  getLength(): number;
  getPointAtLength(distance: number): Point;
  getTangentAtLength(distance: number): Vector2D;
  simplify(tolerance: number): Path;
  reverse(): Path;
  offset(distance: number): Path[];
}

export interface BezierCurve {
  p0: Point; // Start point
  p1: Point; // Control point 1
  p2: Point; // Control point 2 (for cubic)
  p3: Point; // End point
  order: 2 | 3; // Quadratic or cubic

  evaluate(t: number): Point;
  derivative(t: number): Vector2D;
  split(t: number): [BezierCurve, BezierCurve];
  getBounds(): BoundingBox;
  getLength(): number;
}

// ============================================================================
// Shape System
// ============================================================================

export enum ShapeType {
  Path = 'path',
  Rectangle = 'rectangle',
  Circle = 'circle',
  Ellipse = 'ellipse',
  Polygon = 'polygon',
  Line = 'line',
  Polyline = 'polyline',
  Text = 'text',
  Group = 'group',
  Image = 'image'
}

export interface ShapeStyle {
  fill?: string | CanvasGradient | CanvasPattern;
  fillOpacity?: number;
  stroke?: string;
  strokeWidth?: number;
  strokeOpacity?: number;
  strokeLinecap?: 'butt' | 'round' | 'square';
  strokeLinejoin?: 'miter' | 'round' | 'bevel';
  strokeDasharray?: number[];
  strokeDashoffset?: number;
  opacity?: number;
  blendMode?: GlobalCompositeOperation;
}

export interface BaseShape {
  id: string;
  type: ShapeType;
  name?: string;
  layerId: string;
  transform: Transform;
  style: ShapeStyle;
  locked: boolean;
  visible: boolean;
  selected: boolean;
  metadata?: Record<string, any>;

  getBounds(): BoundingBox;
  containsPoint(point: Point): boolean;
  intersects(other: BaseShape): boolean;
  clone(): BaseShape;
}

export interface PathShape extends BaseShape {
  type: ShapeType.Path;
  path: Path;
}

export interface RectangleShape extends BaseShape {
  type: ShapeType.Rectangle;
  rect: Rectangle;
  cornerRadius?: number;
}

export interface CircleShape extends BaseShape {
  type: ShapeType.Circle;
  circle: Circle;
}

export interface EllipseShape extends BaseShape {
  type: ShapeType.Ellipse;
  center: Point;
  radiusX: number;
  radiusY: number;
}

export interface PolygonShape extends BaseShape {
  type: ShapeType.Polygon;
  points: Point[];
}

export interface LineShape extends BaseShape {
  type: ShapeType.Line;
  start: Point;
  end: Point;
}

export interface TextShape extends BaseShape {
  type: ShapeType.Text;
  text: string;
  position: Point;
  fontSize: number;
  fontFamily: string;
  fontWeight?: string;
  fontStyle?: string;
  textAlign?: 'left' | 'center' | 'right';
  textBaseline?: 'top' | 'middle' | 'bottom';
}

export interface GroupShape extends BaseShape {
  type: ShapeType.Group;
  children: string[]; // Child shape IDs
}

export type Shape =
  | PathShape
  | RectangleShape
  | CircleShape
  | EllipseShape
  | PolygonShape
  | LineShape
  | TextShape
  | GroupShape;

// ============================================================================
// Layer System
// ============================================================================

export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  blendMode: GlobalCompositeOperation;
  shapes: string[]; // Shape IDs
  parent?: string; // Parent layer ID for nested layers
  children?: string[]; // Child layer IDs
  order: number;
  color?: string; // Layer color for organization
}

// ============================================================================
// Viewport and Camera
// ============================================================================

export interface Viewport {
  x: number;
  y: number;
  width: number;
  height: number;
  zoom: number;
  rotation: number;
  center: Point;

  screenToWorld(point: Point): Point;
  worldToScreen(point: Point): Point;
  getViewBounds(): BoundingBox;
  fitBounds(bounds: BoundingBox, padding?: number): void;
}

export interface Camera {
  position: vec2;
  zoom: number;
  rotation: number;
  viewMatrix: mat3;
  projectionMatrix: mat3;

  update(): void;
  pan(delta: vec2): void;
  zoomAt(point: Point, factor: number): void;
  reset(): void;
}

// ============================================================================
// CAD Document
// ============================================================================

export interface CADDocument {
  id: string;
  name: string;
  width: number;
  height: number;
  units: 'px' | 'mm' | 'cm' | 'in' | 'pt';
  layers: Map<string, Layer>;
  shapes: Map<string, Shape>;
  viewport: Viewport;
  activeLayerId?: string;
  selectedShapeIds: Set<string>;

  addLayer(layer: Layer): void;
  removeLayer(layerId: string): void;
  addShape(shape: Shape, layerId?: string): void;
  removeShape(shapeId: string): void;
  getShape(shapeId: string): Shape | undefined;
  getLayer(layerId: string): Layer | undefined;
  getShapesInLayer(layerId: string): Shape[];
  export(format: 'svg' | 'json' | 'dxf'): string;
}

// ============================================================================
// Snap System
// ============================================================================

export enum SnapType {
  Grid = 'grid',
  Point = 'point',
  Midpoint = 'midpoint',
  Center = 'center',
  Intersection = 'intersection',
  Perpendicular = 'perpendicular',
  Tangent = 'tangent',
  Quadrant = 'quadrant',
  Endpoint = 'endpoint',
  Extension = 'extension',
  Parallel = 'parallel'
}

export interface SnapPoint {
  point: Point;
  type: SnapType;
  targetId?: string; // Shape/entity ID
  distance: number; // Distance from cursor
  angle?: number;
  metadata?: Record<string, any>;
}

export interface SnapSettings {
  enabled: boolean;
  threshold: number; // Screen space pixels
  types: Set<SnapType>;
  gridSize?: number;
  angleSnap?: number; // Snap to angles (e.g., 15, 30, 45 degrees)
}

// ============================================================================
// Constraint System
// ============================================================================

export enum ConstraintType {
  Distance = 'distance',
  Angle = 'angle',
  Parallel = 'parallel',
  Perpendicular = 'perpendicular',
  Horizontal = 'horizontal',
  Vertical = 'vertical',
  Coincident = 'coincident',
  Concentric = 'concentric',
  Tangent = 'tangent',
  Equal = 'equal',
  Symmetric = 'symmetric',
  Fix = 'fix'
}

export interface Constraint {
  id: string;
  type: ConstraintType;
  entities: string[]; // Entity IDs involved
  value?: number; // For distance/angle constraints
  reference?: string; // Reference entity ID
  satisfied: boolean;

  evaluate(): number; // Error/violation amount
  resolve(): void; // Apply constraint
}

export interface ConstraintSolver {
  constraints: Map<string, Constraint>;

  addConstraint(constraint: Constraint): void;
  removeConstraint(constraintId: string): void;
  solve(iterations?: number, tolerance?: number): boolean;
  getDependencies(entityId: string): Constraint[];
}

// ============================================================================
// GPU Rendering System
// ============================================================================

export interface GPUBuffer {
  id: string;
  buffer: WebGLBuffer;
  target: number; // GL_ARRAY_BUFFER or GL_ELEMENT_ARRAY_BUFFER
  usage: number; // GL_STATIC_DRAW, GL_DYNAMIC_DRAW, etc.
  size: number;
  data?: Float32Array | Uint16Array | Uint32Array;

  bind(gl: WebGL2RenderingContext): void;
  update(gl: WebGL2RenderingContext, data: ArrayBuffer, offset?: number): void;
  destroy(gl: WebGL2RenderingContext): void;
}

export interface Shader {
  id: string;
  program: WebGLProgram;
  vertexShader: WebGLShader;
  fragmentShader: WebGLShader;
  uniforms: Map<string, WebGLUniformLocation>;
  attributes: Map<string, number>;

  use(gl: WebGL2RenderingContext): void;
  setUniform(gl: WebGL2RenderingContext, name: string, value: any): void;
  destroy(gl: WebGL2RenderingContext): void;
}

export interface Texture {
  id: string;
  texture: WebGLTexture;
  width: number;
  height: number;
  format: number;
  type: number;

  bind(gl: WebGL2RenderingContext, unit?: number): void;
  update(gl: WebGL2RenderingContext, data: TexImageSource): void;
  destroy(gl: WebGL2RenderingContext): void;
}

export interface RenderBatch {
  shader: string;
  texture?: string;
  vertexBuffer: string;
  indexBuffer?: string;
  vertexCount: number;
  indexCount?: number;
  instanceCount?: number;
  drawMode: number; // GL_TRIANGLES, GL_LINES, etc.
  uniforms: Map<string, any>;
}

export interface RenderPipeline {
  passes: RenderPass[];
  framebuffers: Map<string, WebGLFramebuffer>;

  execute(gl: WebGL2RenderingContext, scene: CADDocument): void;
  addPass(pass: RenderPass): void;
  removePass(passId: string): void;
}

export interface RenderPass {
  id: string;
  enabled: boolean;
  framebuffer?: WebGLFramebuffer;
  clearColor?: [number, number, number, number];
  clearDepth?: number;

  render(gl: WebGL2RenderingContext, scene: CADDocument): void;
}

// ============================================================================
// Selection System
// ============================================================================

export interface SelectionState {
  selectedIds: Set<string>;
  hoveredId?: string;
  focusedId?: string;
  mode: 'normal' | 'additive' | 'subtractive';
  bounds?: BoundingBox;

  select(ids: string | string[]): void;
  deselect(ids: string | string[]): void;
  clear(): void;
  toggle(id: string): void;
  contains(id: string): boolean;
}

export interface SelectionBox {
  start: Point;
  end: Point;
  mode: 'intersect' | 'contain';

  getBounds(): BoundingBox;
  intersects(shape: Shape): boolean;
  contains(shape: Shape): boolean;
}

// ============================================================================
// Tool System
// ============================================================================

export enum ToolType {
  Select = 'select',
  Pen = 'pen',
  Line = 'line',
  Rectangle = 'rectangle',
  Circle = 'circle',
  Ellipse = 'ellipse',
  Polygon = 'polygon',
  Pan = 'pan',
  Zoom = 'zoom',
  Measure = 'measure',
  Dimension = 'dimension',
  Boolean = 'boolean'
}

export interface ToolState {
  active: ToolType;
  cursor: string;
  temporary: Shape[];
  preview?: Shape;

  onMouseDown(point: Point, event: MouseEvent): void;
  onMouseMove(point: Point, event: MouseEvent): void;
  onMouseUp(point: Point, event: MouseEvent): void;
  onKeyDown(event: KeyboardEvent): void;
  reset(): void;
}

// ============================================================================
// Measurement and Dimensioning
// ============================================================================

export interface Measurement {
  id: string;
  type: 'distance' | 'angle' | 'area' | 'perimeter';
  value: number;
  units: string;
  start?: Point;
  end?: Point;
  points?: Point[];
  label?: string;
}

export interface Dimension {
  id: string;
  type: 'linear' | 'angular' | 'radial' | 'diameter';
  value: number;
  points: Point[];
  offset: number;
  style: DimensionStyle;
  text?: string;
}

export interface DimensionStyle {
  arrowSize: number;
  arrowType: 'arrow' | 'dot' | 'slash' | 'none';
  lineColor: string;
  lineWidth: number;
  textSize: number;
  textColor: string;
  textFont: string;
  precision: number;
  extensionLineOffset: number;
  extensionLineExtension: number;
}

// ============================================================================
// Boolean Operations
// ============================================================================

export enum BooleanOperation {
  Union = 'union',
  Subtract = 'subtract',
  Intersect = 'intersect',
  Exclude = 'exclude'
}

export interface BooleanResult {
  success: boolean;
  shapes: Shape[];
  error?: string;
}

// ============================================================================
// History and Undo System
// ============================================================================

export interface Command {
  id: string;
  name: string;
  timestamp: number;

  execute(): void;
  undo(): void;
  redo(): void;
  canMerge?(other: Command): boolean;
  merge?(other: Command): void;
}

export interface History {
  commands: Command[];
  currentIndex: number;
  maxSize: number;

  execute(command: Command): void;
  undo(): void;
  redo(): void;
  canUndo(): boolean;
  canRedo(): boolean;
  clear(): void;
}

// ============================================================================
// Export and Import
// ============================================================================

export interface ExportOptions {
  format: 'svg' | 'pdf' | 'dxf' | 'json' | 'png' | 'jpg';
  quality?: number;
  scale?: number;
  bounds?: BoundingBox;
  layers?: string[];
  includeHidden?: boolean;
}

export interface ImportResult {
  success: boolean;
  document?: CADDocument;
  shapes?: Shape[];
  layers?: Layer[];
  error?: string;
  warnings?: string[];
}

// ============================================================================
// Events
// ============================================================================

export enum CADEventType {
  ShapeAdded = 'shape:added',
  ShapeRemoved = 'shape:removed',
  ShapeModified = 'shape:modified',
  ShapeSelected = 'shape:selected',
  LayerAdded = 'layer:added',
  LayerRemoved = 'layer:removed',
  LayerModified = 'layer:modified',
  ViewportChanged = 'viewport:changed',
  ToolChanged = 'tool:changed',
  DocumentSaved = 'document:saved',
  DocumentLoaded = 'document:loaded'
}

export interface CADEvent {
  type: CADEventType;
  timestamp: number;
  data: any;
  source?: string;
}

export type CADEventListener = (event: CADEvent) => void;

export interface EventEmitter {
  on(type: CADEventType, listener: CADEventListener): void;
  off(type: CADEventType, listener: CADEventListener): void;
  emit(event: CADEvent): void;
}
