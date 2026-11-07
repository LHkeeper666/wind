<script lang="ts">
  import { onDestroy, tick, untrack } from 'svelte';
  import { PreviewRouter, type TocHeading } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import { isPdfFile, isImageFile, isVideoFile } from '$lib/utils/file-types';
  import { loadArchiveDirectory, loadArchiveFile, loadImage, loadTextOrBinary, loadVideo } from '$lib/utils/file-loaders';
  import type { LoadContext } from '$lib/utils/file-loaders';
  import { createPdfState } from '$lib/composables/pdf-state.svelte';
  import { logInfo } from '$lib/utils/log';
  import PdfPreviewPanel from './PdfPreviewPanel.svelte';
  import PdfTocSidebar from './PdfTocSidebar.svelte';
  import TocSidebar from './TocSidebar.svelte';

  let { workspaceId, path, isDirectory, archivePath = null, recycleItem = null }: {
    workspaceId: number;
    path: string;
    isDirectory: boolean | null;
    archivePath?: string | null;
    recycleItem?: { id: string; name: string; original_path: string; date_deleted: number; size: number | null } | null;
  } = $props();

  let body: HTMLDivElement;
  let generation = 0;
  let lastTargetKey = '';
  let pdfStartedAt = 0;
  let previousPdfSelectionAt = 0;
  let pdfWaitMs = 50;
  let firstPdfPaintLogged = false;
  let pdfDelay: ReturnType<typeof setTimeout> | undefined;
  let cancelPdfDelay: (() => void) | undefined;
  function clearPdfDelay() {
    if (pdfDelay !== undefined) clearTimeout(pdfDelay);
    pdfDelay = undefined;
    cancelPdfDelay?.();
    cancelPdfDelay = undefined;
  }
  let renderedPath = $state('');
  let error = $state('');
  let pdfVisible = $state(false);
  let headings = $state<TocHeading[]>([]);
  let tocOpen = $state(true);
  let router = new PreviewRouter();
  router.onHeadings = (value) => { headings = value; };
  let directoryPreviewer = new DirectoryPreviewer();
  const pdf = createPdfState({ getFilePath: () => path, onToast: () => {} });

  async function load(workspace: number, selectedPath: string, directory: boolean | null, archive: string | null, gen: number) {
    // Keep the overlay opaque while the next file loads, but do not show the
    // previous selection or expose the opened slot underneath it.
    body?.replaceChildren();
    error = '';
    pdfVisible = false;
    renderedPath = '';
    const previousRouter = router;
    const nextRouter = new PreviewRouter();
    nextRouter.onHeadings = (value) => { if (generation === gen) headings = value; };
    headings = [];
    pdf.reset();
    await tick();
    const valid = () => generation === gen && workspaceId === workspace && path === selectedPath && archivePath === archive;
    if (!valid() || !body) { nextRouter.dispose(); return; }
    const ctx: LoadContext = {
      gen, loadTabId: workspace, path: selectedPath, archivePath: archive,
      archiveInternalPath: archive ? selectedPath : null, selectedEntryIsDir: directory,
    };
    try {
      if (isPdfFile(selectedPath) && !archive) {
        await new Promise<void>((resolve) => {
          cancelPdfDelay = resolve;
          pdfDelay = setTimeout(() => {
            pdfDelay = undefined;
            cancelPdfDelay = undefined;
            resolve();
          }, pdfWaitMs);
        });
        if (!valid()) { nextRouter.dispose(); return; }
        logInfo('pdf-select', `dispatch gen=${gen} file=${selectedPath.split(/[/\\]/).pop()} targetWait=${pdfWaitMs}ms wait=${(performance.now() - pdfStartedAt).toFixed(1)}ms`);
        const requestedAt = performance.now();
        const ready = await pdf.loadPdfInfo(selectedPath, gen);
        logInfo('pdf-select', `info gen=${gen} file=${selectedPath.split(/[/\\]/).pop()} invoke=${(performance.now() - requestedAt).toFixed(1)}ms current=${valid()} ready=${ready}`);
        if (valid()) {
          if (ready) pdfVisible = true;
          else error = 'Failed to load PDF';
        }
      } else if (directory) {
        if (archive) {
          const result = await loadArchiveDirectory(ctx, valid);
          if (valid() && !('aborted' in result)) body.innerHTML = result.html;
        } else {
          const staging = document.createElement('div');
          staging.dataset.filePath = selectedPath;
          await directoryPreviewer.render('', staging);
          if (valid()) body.replaceChildren(...Array.from(staging.childNodes));
        }
      } else {
        const result = archive ? await loadArchiveFile(ctx, valid)
          : isImageFile(selectedPath) && !/\.svg$/i.test(selectedPath) ? await loadImage(ctx, valid)
          : isVideoFile(selectedPath) ? await loadVideo(ctx, valid)
          : await loadTextOrBinary(ctx, valid);
        if (!valid() || 'aborted' in result) { nextRouter.dispose(); return; }
        const content = result.binaryContent ?? result.content;
        const renderHost = document.createElement('div');
        renderHost.dataset.filePath = selectedPath;
        renderHost.style.visibility = 'hidden';
        if ('originalFileSize' in result) renderHost.dataset.originalFileSize = String(result.originalFileSize);
        renderHost.style.minHeight = '100%';
        if ('thumbnailMeta' in result && result.thumbnailMeta) {
          renderHost.dataset.thumbWidth = String(result.thumbnailMeta.width);
          renderHost.dataset.thumbHeight = String(result.thumbnailMeta.height);
          renderHost.dataset.thumbOriginalSize = String(result.thumbnailMeta.originalSize);
          renderHost.dataset.thumbIsThumbnail = String(result.thumbnailMeta.isThumbnail);
        }
        body.replaceChildren(renderHost);
        await nextRouter.preview(selectedPath, content, renderHost, valid);
        if (!valid()) { renderHost.remove(); nextRouter.dispose(); return; }
        renderHost.style.visibility = '';
      }
      if (valid()) {
        router = nextRouter;
        previousRouter.dispose();
        renderedPath = selectedPath;
      } else nextRouter.dispose();
    } catch (cause) {
      body?.replaceChildren();
      nextRouter.dispose();
      if (valid()) error = String(cause);
    }
  }

  $effect(() => {
    const workspace = workspaceId;
    const selectedPath = path;
    const directory = isDirectory;
    const archive = archivePath;
    const recycle = recycleItem;
    return untrack(() => {
      const targetKey = JSON.stringify([workspace, selectedPath, directory, archive, recycle?.id]);
      if (targetKey === lastTargetKey) return;
      lastTargetKey = targetKey;
      const gen = ++generation;
      pdfStartedAt = performance.now();
      if (isPdfFile(selectedPath) && !archive) {
        pdfWaitMs = pdfStartedAt - previousPdfSelectionAt < 500 ? 420 : 50;
        previousPdfSelectionAt = pdfStartedAt;
      } else {
        previousPdfSelectionAt = 0;
      }
      firstPdfPaintLogged = false;
      clearPdfDelay();
      router.dispose();
      if (body) body.dataset.filePath = '';
      body?.replaceChildren();
      renderedPath = '';
      error = '';
      pdfVisible = false;
      headings = [];
      pdf.reset();
      pdf.invalidatePendingLoads(gen);
      if (recycle?.id === selectedPath) {
        generation++;
        clearPdfDelay();
        renderedPath = selectedPath;
      } else {
        void load(workspace, selectedPath, directory, archive, gen);
      }
    });
  });

  onDestroy(() => { generation++; clearPdfDelay(); router.dispose(); directoryPreviewer.dispose(); pdf.reset(); pdf.invalidatePendingLoads(generation); });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="selection-preview" role="region" aria-label="Selection preview" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
  {#if recycleItem?.id === path}
    <div class="status"><strong>{recycleItem.name}</strong><p>{recycleItem.original_path}</p><p>{new Date(recycleItem.date_deleted * 1000).toLocaleString('zh-CN')}</p></div>
  {/if}
  {#if error}<div class="status">{error}</div>{/if}
  {#if pdfVisible && renderedPath === path}
    <div class="pdf-content">
      {#if pdf.getPdfOutline().length > 0}
        <button class="outline-toggle" onclick={() => pdf.togglePdfToc()} aria-label="Toggle PDF outline">☰</button>
      {/if}
      <PdfPreviewPanel bind:this={pdf.pdfPreviewPanel} pdfPath={path} pageDimensions={pdf.getPdfPageDimensions()} pageCount={pdf.getPdfPageCount()} fileSize={pdf.getPdfFileSize()} title={pdf.getPdfTitle()} onPageChange={(page) => pdf.setPage(page)} onFirstPaint={() => { if (!firstPdfPaintLogged) { firstPdfPaintLogged = true; logInfo('pdf-select', `first-paint gen=${generation} file=${path.split(/[/\\]/).pop()} total=${(performance.now() - pdfStartedAt).toFixed(1)}ms`); } }} />
      {#if pdf.getPdfOutline().length > 0 && pdf.getPdfTocOpen()}
        <PdfTocSidebar bind:this={pdf.pdfTocSidebar} outline={pdf.getPdfOutline()} currentPage={pdf.getPdfCurrentPage()} pageDimensions={pdf.getPdfPageDimensions()} expandedIds={new Set(pdf.getPdfTocExpanded())} onToggleExpanded={(id) => pdf.togglePdfTocExpanded(id)} onJump={(page, y) => pdf.jumpToPage(page, y)} onFocusChange={(focused) => pdf.setTocFocused(focused)} onExit={() => {}} />
      {/if}
    </div>
  {/if}
  {#if headings.length > 0 && !pdfVisible && renderedPath === path}
    <button class="outline-toggle" onclick={() => tocOpen = !tocOpen} aria-label="Toggle outline">☰</button>
  {/if}
  <div class="reading-content" class:hidden={pdfVisible || recycleItem?.id === path || renderedPath !== path}>
    <div class="selection-body" bind:this={body}></div>
    {#if headings.length > 0 && tocOpen}
      <TocSidebar {headings} onJump={(line) => {
        const target = Array.from(body.querySelectorAll<HTMLElement>('[data-line]'))
          .find(element => Number(element.dataset.line) >= line);
        target?.scrollIntoView({ block: 'start' });
      }} />
    {/if}
  </div>
</div>

<style>
  .selection-preview { position: absolute; inset: 0; z-index: 5; background: var(--bg-primary); display: flex; flex-direction: column; min-height: 0; }
  .selection-body { overflow: auto; flex: 1; min-height: 0; padding: 12px; }
  .reading-content { display: flex; flex: 1; min-height: 0; }
  .reading-content.hidden { display: none; }
  .pdf-content { display: flex; flex: 1; min-height: 0; }
  .outline-toggle { align-self: flex-end; }
  .status { padding: 12px; }
</style>
