import type { OcrGeometryBounds, OcrGeometryPoint, OverlayBox } from '../../api/types';

export interface OverlayShape {
  bounds: OcrGeometryBounds;
  isPolygon: boolean;
  points: OcrGeometryPoint[];
  svgPoints: string;
}

export function overlayShapeForBox(box: OverlayBox): OverlayShape {
  const fallbackBounds = boundsFromBox(box);
  const geometry = box.geometry;
  if (geometry?.coordinate_space !== 'page_percent' || geometry.points.length < 3) {
    return rectangularShape(fallbackBounds);
  }
  const bounds = validBounds(geometry.bounds) ? geometry.bounds : fallbackBounds;
  const svgPoints = geometry.points.map((point) => relativePoint(point, bounds)).join(' ');
  return {
    bounds,
    isPolygon: geometry.kind === 'rotated_quad' || geometry.kind === 'polygon',
    points: geometry.points,
    svgPoints,
  };
}

function rectangularShape(bounds: OcrGeometryBounds): OverlayShape {
  const points = [
    { x: bounds.left, y: bounds.top },
    { x: bounds.left + bounds.width, y: bounds.top },
    { x: bounds.left + bounds.width, y: bounds.top + bounds.height },
    { x: bounds.left, y: bounds.top + bounds.height },
  ];
  return {
    bounds,
    isPolygon: false,
    points,
    svgPoints: '0,0 100,0 100,100 0,100',
  };
}

function boundsFromBox(box: OverlayBox): OcrGeometryBounds {
  return {
    left: box.left_percent,
    top: box.top_percent,
    width: box.width_percent,
    height: box.height_percent,
  };
}

function validBounds(bounds: OcrGeometryBounds) {
  return bounds.width > 0 && bounds.height > 0;
}

function relativePoint(point: OcrGeometryPoint, bounds: OcrGeometryBounds) {
  const x = ((point.x - bounds.left) / bounds.width) * 100;
  const y = ((point.y - bounds.top) / bounds.height) * 100;
  return `${roundForSvg(x)},${roundForSvg(y)}`;
}

function roundForSvg(value: number) {
  return Number(value.toFixed(3));
}
