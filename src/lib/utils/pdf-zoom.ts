export const PDF_ZOOM_KEY_STEP = 0.05;

export function clampPdfZoom(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

export function wheelZoomDelta(deltaY: number, deltaMode: number): number {
  const pixels = deltaY * (deltaMode === 1 ? 16 : deltaMode === 2 ? 800 : 1);
  // A typical 100px wheel notch changes the scale by about ten percentage points.
  return -clampPdfZoom(pixels, -160, 160) * 0.001;
}

export function nextZoomScale(target: number, delta: number, min: number, max: number): number {
  return clampPdfZoom(Math.round((target + delta) * 10000) / 10000, min, max);
}

export function advanceZoomScale(current: number, target: number, elapsedMs: number): number {
  if (Math.abs(target - current) < 0.0005) return target;
  const fraction = 1 - Math.exp(-Math.min(elapsedMs, 64) / 55);
  const next = current + (target - current) * fraction;
  return Math.abs(target - next) < 0.0005 ? target : next;
}
