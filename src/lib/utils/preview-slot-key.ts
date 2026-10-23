/** Numeric identity shared by editor sessions, preview DOM and PDF sessions. */
export function previewSlotKey(workspaceId: number, slotId: number): number {
  return workspaceId * 1_000_000 + slotId;
}
