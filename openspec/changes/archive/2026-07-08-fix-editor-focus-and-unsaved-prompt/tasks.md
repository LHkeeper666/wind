## 1. Editor focus forwarding (PreviewEditor)

- [x] 1.1 Add `handlePanelFocus()` method that forwards focus to overlay (editor-normal) or editorView (editor-insert)
- [x] 1.2 Bind `onfocus={handlePanelFocus}` on the panelElement div in template

## 2. Expose isModified state

- [x] 2.1 Add `getIsModified()` export function on PreviewEditor that returns `isModified`

## 3. Unsaved changes dialog (PanelLayout)

- [x] 3.1 Add `pendingActivatePath` state and `showUnsavedConfirm` state to PanelLayout
- [x] 3.2 Modify `handleActivate` to check `previewEditor?.getIsModified()` before switching files
- [x] 3.3 Add unsaved-confirm branch to ConfirmModal template with Save/Discard/Cancel buttons
- [x] 3.4 Implement `handleUnsavedSave()` — save file then switch to pendingActivatePath
- [x] 3.5 Implement `handleUnsavedDiscard()` — revert content then switch to pendingActivatePath
- [x] 3.6 Implement `handleUnsavedCancel()` — close dialog, keep editing current file

## 4. Verification

- [ ] 4.1 Test mouse click outside editor → click back → keyboard works in editor-normal mode
- [ ] 4.2 Test mouse click outside editor → click back → keyboard works in editor-insert mode
- [ ] 4.3 Test unsaved prompt appears when switching files with unsaved changes
- [ ] 4.4 Test Save button saves and switches correctly
- [ ] 4.5 Test Discard button discards and switches correctly
- [ ] 4.6 Test Cancel button keeps current state
- [ ] 4.7 Test no prompt when file is unmodified
