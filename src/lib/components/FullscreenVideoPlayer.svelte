<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';

  let {
    filePath = null,
    fileSize = 0,
    onClose = () => {},
  }: {
    filePath: string | null;
    fileSize?: number;
    onClose?: () => void;
  } = $props();

  let videoUrl = $state('');
  let isLoading = $state(true);
  let hasError = $state(false);
  let errorMessage = $state('');
  let videoEl: HTMLVideoElement | undefined = $state(undefined);
  let overlayEl: HTMLDivElement | undefined = $state(undefined);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(1);
  let isPlaying = $state(false);

  const SEEK_STEP = 5;
  const VOLUME_STEP = 0.1;

  function formatTime(seconds: number): string {
    if (seconds <= 0) return '0:00';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) {
      return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    }
    return `${m}:${String(s).padStart(2, '0')}`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getFileName(): string {
    if (!filePath) return '';
    return filePath.split('\\').pop() || filePath.split('/').pop() || '';
  }

  function cleanup() {
    if (videoEl) {
      videoEl.pause();
      videoEl.removeAttribute('src');
      videoEl.load();
    }
    // Stop the HTTP server in the background
    invoke('stop_video_server').catch(() => {});
  }

  async function loadVideo() {
    if (!filePath) return;
    isLoading = true;
    hasError = false;

    try {
      const url = await invoke<string>('start_video_server', { path: filePath });
      videoUrl = url;
      isLoading = false;
    } catch (e) {
      isLoading = false;
      hasError = true;
      errorMessage = '视频加载失败: ' + String(e);
    }
  }

  function handleVideoError() {
    hasError = true;
    errorMessage = '解码失败，格式不受浏览器支持';
    cleanup();
  }

  async function openSystemPlayer() {
    if (!filePath) return;
    try {
      await openPath(filePath);
    } catch {
      // Fallback: do nothing
    }
  }

  function handleLoadedMetadata() {
    if (videoEl) {
      duration = videoEl.duration;
      volume = videoEl.volume;
      videoEl.play();
      isPlaying = true;
    }
  }

  function handleTimeUpdate() {
    if (videoEl) {
      currentTime = videoEl.currentTime;
      isPlaying = !videoEl.paused;
    }
  }

  function handleEnded() {
    isPlaying = false;
  }

  function togglePlay() {
    if (!videoEl) return;
    if (videoEl.paused) {
      videoEl.play();
      isPlaying = true;
    } else {
      videoEl.pause();
      isPlaying = false;
    }
  }

  function seek(offset: number) {
    if (!videoEl) return;
    videoEl.currentTime = Math.max(0, Math.min(videoEl.duration, videoEl.currentTime + offset));
  }

  function adjustVolume(delta: number) {
    if (!videoEl) return;
    const newVol = Math.max(0, Math.min(1, videoEl.volume + delta));
    videoEl.volume = newVol;
    volume = newVol;
  }

  function seekTo(ratio: number) {
    if (!videoEl) return;
    videoEl.currentTime = videoEl.duration * ratio;
  }

  function toggleFullscreen() {
    if (!overlayEl) return;
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      overlayEl.requestFullscreen();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.code === 'Space') {
      e.preventDefault();
      togglePlay();
    } else if (e.key === 'h' || e.key === 'ArrowLeft') {
      e.preventDefault();
      seek(-SEEK_STEP);
    } else if (e.key === 'l' || e.key === 'ArrowRight') {
      e.preventDefault();
      seek(SEEK_STEP);
    } else if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      adjustVolume(-VOLUME_STEP);
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      adjustVolume(VOLUME_STEP);
    } else if (e.key === '0' || e.key === 'Home') {
      e.preventDefault();
      seekTo(0);
    } else if (e.key === '$' || e.key === 'End') {
      e.preventDefault();
      seekTo(1);
    } else if (e.key === 'f' && !e.ctrlKey && !e.altKey && !e.metaKey) {
      e.preventDefault();
      toggleFullscreen();
    } else if (e.key === 'Escape' && !e.ctrlKey && !e.altKey && !e.metaKey) {
      e.preventDefault();
      close();
    }
  }

  function close() {
    cleanup();
    onClose();
  }

  // Load video on mount
  $effect(() => {
    if (filePath) {
      loadVideo();
    }
  });

  // Focus overlay on mount so keydown events are captured
  import { onMount, onDestroy } from 'svelte';
  onMount(() => {
    overlayEl?.focus();
  });
  onDestroy(() => {
    cleanup();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fullscreen-video-overlay"
  bind:this={overlayEl}
  onkeydown={handleKeydown}
  role="dialog"
  aria-label="Video Player"
  tabindex="0"
>
  <!-- Header bar -->
  <div class="video-header">
    <span class="video-title">{getFileName()}</span>
    {#if !hasError && !isLoading}
      <span class="video-meta">
        {filePath ? formatSize(fileSize) : ''} {duration > 0 ? `· ${formatTime(currentTime)} / ${formatTime(duration)}` : ''}
      </span>
    {/if}
    <span class="mode-indicator">PLAY</span>
  </div>

  <!-- Video area -->
  <div class="video-area">
    {#if isLoading}
      <div class="video-loading">加载中...</div>
    {:else if hasError}
      <div class="video-error">
        <p>{errorMessage}</p>
        <button class="video-open-system" onclick={openSystemPlayer}>用系统播放器打开</button>
      </div>
    {:else}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        bind:this={videoEl}
        src={videoUrl}
        class="video-element"
        onloadedmetadata={handleLoadedMetadata}
        ontimeupdate={handleTimeUpdate}
        onended={handleEnded}
        onerror={handleVideoError}
        onclick={togglePlay}
      ></video>
      <!-- Progress bar -->
      <div class="video-progress" onclick={(e) => {
        if (!videoEl) return;
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const ratio = (e.clientX - rect.left) / rect.width;
        seekTo(ratio);
      }}>
        <div class="video-progress-fill" style="width: {duration > 0 ? (currentTime / duration) * 100 : 0}%"></div>
      </div>
    {/if}
  </div>

  <!-- Control hints -->
  <div class="video-controls">
    <span>Space:{isPlaying ? '暂停' : '播放'}</span>
    <span>h/l:±5s</span>
    <span>j/k:音量</span>
    <span>0/$:首/尾</span>
    <span>f:全屏</span>
    <span>Esc:关闭</span>
  </div>

</div>

<style>
  .fullscreen-video-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 1000;
    background-color: rgba(0, 0, 0, 0.92);
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    outline: none;
  }

  .video-header {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background-color: #1d2021;
    border-bottom: 1px solid #3c3836;
    gap: 12px;
    flex-shrink: 0;
  }

  .video-title {
    color: #ebdbb2;
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .video-meta {
    color: #928374;
    font-size: 11px;
    flex: 1;
  }

  .mode-indicator {
    font-size: 10px;
    padding: 1px 8px;
    background-color: #fe8019;
    color: #1d2021;
    flex-shrink: 0;
  }

  .video-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    min-height: 0;
  }

  .video-element {
    width: 100%;
    height: 100%;
    object-fit: contain;
    cursor: pointer;
  }

  .video-progress {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: rgba(235, 219, 178, 0.15);
    cursor: pointer;
  }

  .video-progress-fill {
    height: 100%;
    background: #fe8019;
    transition: width 0.1s linear;
  }

  .video-loading {
    color: #928374;
    font-size: 14px;
  }

  .video-error {
    text-align: center;
    color: #fb4934;
    font-size: 14px;
  }

  .video-error-hint {
    color: #928374;
    font-size: 12px;
    margin-top: 8px;
  }

  .video-open-system {
    margin-top: 16px;
    background: none;
    border: 1px solid #b8bb26;
    color: #b8bb26;
    padding: 6px 16px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .video-open-system:hover {
    background: rgba(184, 187, 38, 0.15);
  }

  .video-controls {
    display: flex;
    gap: 16px;
    padding: 6px 16px;
    background-color: #1d2021;
    border-top: 1px solid #3c3836;
    color: #7c6f64;
    font-size: 11px;
    flex-shrink: 0;
  }

  .video-controls span {
    white-space: nowrap;
  }

  .video-confirm-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1010;
  }

  .video-confirm-dialog {
    background: #282828;
    border: 1px solid #504945;
    padding: 24px;
    text-align: center;
  }

  .video-confirm-dialog p {
    color: #ebdbb2;
    font-size: 13px;
    margin-bottom: 16px;
  }

  .video-confirm-buttons {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  .video-confirm-buttons button {
    background: none;
    border: 1px solid #504945;
    color: #ebdbb2;
    padding: 4px 16px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .video-confirm-buttons button:hover {
    background: #3c3836;
  }
</style>
