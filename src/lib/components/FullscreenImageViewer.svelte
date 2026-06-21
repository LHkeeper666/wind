<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface ImageEntry {
    name: string;
    path: string;
  }

  let {
    imageList = [],
    currentIndex = 0,
    onClose = () => {},
    onNavigate = (_index: number) => {},
  }: {
    imageList: ImageEntry[];
    currentIndex: number;
    onClose?: () => void;
    onNavigate?: (index: number) => void;
  } = $props();

  let scale = $state(1);
  let translateX = $state(0);
  let translateY = $state(0);
  let imageSrc = $state('');
  let imageWidth = $state(0);
  let imageHeight = $state(0);
  let imageFileSize = $state(0);
  let isLoading = $state(true);
  let hasError = $state(false);
  let viewportEl: HTMLDivElement | undefined = $state(undefined);
  let overlayEl: HTMLDivElement | undefined = $state(undefined);
  let currentBlobUrl: string | null = null;

  const PAN_STEP = 100;
  const ZOOM_STEP = 0.25;
  const MIN_SCALE = 0.1;

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function revokeBlobUrl() {
    if (currentBlobUrl) {
      URL.revokeObjectURL(currentBlobUrl);
      currentBlobUrl = null;
    }
  }

  async function loadImage(path: string) {
    isLoading = true;
    hasError = false;
    revokeBlobUrl();

    try {
      const base64 = await invoke<string>('read_binary_file', { path });
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes.buffer]);
      const url = URL.createObjectURL(blob);
      currentBlobUrl = url;

      await new Promise<void>((resolve, reject) => {
        const img = new Image();
        img.onload = () => {
          imageWidth = img.naturalWidth;
          imageHeight = img.naturalHeight;
          imageFileSize = bytes.length;
          imageSrc = url;
          isLoading = false;

          // Calculate fit-to-screen scale
          if (viewportEl) {
            const vw = viewportEl.clientWidth;
            const vh = viewportEl.clientHeight;
            scale = Math.min(vw / imageWidth, vh / imageHeight, 1);
          } else {
            scale = 1;
          }
          translateX = 0;
          translateY = 0;
          resolve();
        };
        img.onerror = () => {
          hasError = true;
          isLoading = false;
          reject(new Error('Failed to load image'));
        };
        img.src = url;
      });
    } catch (error) {
      console.error('Failed to load image:', error);
      hasError = true;
      isLoading = false;
    }
  }

  // Load image when currentIndex changes
  $effect(() => {
    const entry = imageList[currentIndex];
    if (entry) {
      loadImage(entry.path);
    }
  });

  // Focus overlay on mount so keyboard events are captured
  onMount(() => {
    requestAnimationFrame(() => {
      overlayEl?.focus();
    });
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' || (event.key === 'q' && !event.ctrlKey)) {
      event.preventDefault();
      onClose();
      return;
    }

    if (event.ctrlKey) {
      switch (event.key) {
        case 'j':
          event.preventDefault();
          translateY -= PAN_STEP;
          break;
        case 'k':
          event.preventDefault();
          translateY += PAN_STEP;
          break;
        case 'h':
          event.preventDefault();
          translateX += PAN_STEP;
          break;
        case 'l':
          event.preventDefault();
          translateX -= PAN_STEP;
          break;
      }
      return;
    }

    switch (event.key) {
      case 'j':
        event.preventDefault();
        onNavigate((currentIndex + 1) % imageList.length);
        break;
      case 'k':
        event.preventDefault();
        onNavigate((currentIndex - 1 + imageList.length) % imageList.length);
        break;
      case 'h':
        event.preventDefault();
        scale = Math.max(MIN_SCALE, scale - ZOOM_STEP);
        break;
      case 'l':
        event.preventDefault();
        scale += ZOOM_STEP;
        break;
    }
  }

  const entry = $derived(imageList[currentIndex]);
  const fileName = $derived(entry?.name || '');
  const position = $derived(`${currentIndex + 1}/${imageList.length}`);
  const imageStyle = $derived(
    `transform: scale(${scale}) translate(${translateX}px, ${translateY}px); transform-origin: center center;`
  );
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="image-viewer-overlay"
  bind:this={overlayEl}
  onkeydown={handleKeydown}
  onclick={() => overlayEl?.focus()}
  role="dialog"
  aria-label="Fullscreen Image Viewer"
  tabindex="-1"
>
  <div class="image-viewport" bind:this={viewportEl}>
    {#if isLoading}
      <div class="image-viewer-status">Loading...</div>
    {:else if hasError}
      <div class="image-viewer-status error">Failed to load image</div>
    {:else if imageSrc}
      <img
        src={imageSrc}
        alt={fileName}
        style={imageStyle}
        draggable="false"
      />
    {/if}
  </div>

  <div class="image-footer">
    <span class="image-footer-name">{fileName}</span>
    {#if !isLoading && !hasError && imageWidth > 0}
      <span class="image-footer-meta">{imageWidth}×{imageHeight}</span>
      <span class="image-footer-meta">{formatSize(imageFileSize)}</span>
    {/if}
    <span class="image-footer-meta">{position}</span>
    <span class="image-footer-hints">j/k:切换 h/l:缩放 Ctrl+hjkl:平移 Esc:关闭</span>
  </div>
</div>

<style>
  .image-viewer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    background-color: #000;
    display: flex;
    flex-direction: column;
    outline: none;
  }

  .image-viewport {
    flex: 1;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-viewport img {
    max-width: none;
    max-height: none;
    user-select: none;
    -webkit-user-drag: none;
  }

  .image-viewer-status {
    color: #999;
    font-size: 16px;
  }

  .image-viewer-status.error {
    color: #e74c3c;
  }

  .image-footer {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 16px;
    background-color: rgba(0, 0, 0, 0.8);
    color: #ccc;
    font-size: 13px;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  }

  .image-footer-name {
    color: #fff;
    font-weight: 500;
  }

  .image-footer-meta {
    color: #999;
  }

  .image-footer-hints {
    margin-left: auto;
    color: #666;
    font-size: 12px;
  }
</style>
