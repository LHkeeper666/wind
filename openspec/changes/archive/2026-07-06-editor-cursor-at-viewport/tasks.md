## 1. PreviewEditor — getVisibleLine() + modify e/E keys

- [x] 1.1 Add `getVisibleLine()` method: read previewContainer.scrollTop, query first code/pre element for computed lineHeight, return `Math.floor(scrollTop / lineHeight)`
- [x] 1.2 Modify `initEditor()` to accept optional `targetLine` parameter; after EditorView creation, dispatch a transaction to set cursor and scroll it into view
- [x] 1.3 In `handleKeydown`, call `getVisibleLine()` before `mode = 'editor-normal'` (e key) and pass it; before `onFullscreen()` (E key), pass line number out

## 2. PanelLayout — pass initialLine to FullscreenEditor

- [x] 2.1 Add `initialLine` state variable
- [x] 2.2 In `handleFullscreenEditor`, capture visible line from `previewEditor?.getVisibleLine()` before opening fullscreen
- [x] 2.3 Pass `initialLine` prop to FullscreenEditor component

## 3. FullscreenEditor — accept and use initialLine

- [x] 3.1 Add `initialLine` prop (default 0)
- [x] 3.2 In `initEditor()`, after EditorView creation, dispatch transaction to set cursor to initialLine and scroll into view

## 4. Verification

- [x] 4.1 Run `npx svelte-check` to verify no type errors
- [x] 4.2 Manual test: open a code file, scroll down in preview with j, press e, verify cursor is near viewport position
- [x] 4.3 Manual test: open a code file, scroll down, press E, verify fullscreen editor cursor is near viewport position
- [x] 4.4 Manual test: open a file without scrolling, press e/E, verify cursor is at line 1 (no regression)
