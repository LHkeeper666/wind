export interface RestoreIdentity {
  workspaceId: number;
  slotId: number;
  generation: number;
}

export function isCurrentRestore(request: RestoreIdentity, current: RestoreIdentity): boolean {
  return request.workspaceId === current.workspaceId
    && request.slotId === current.slotId
    && request.generation === current.generation;
}

export function clampEditorSelection(selection: { anchor: number; head: number }, documentLength: number) {
  const clamp = (position: number) => Math.min(Math.max(0, position), documentLength);
  return { anchor: clamp(selection.anchor), head: clamp(selection.head) };
}
