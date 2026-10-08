<script lang="ts">
  import '../lib/styles/themes.css';
  import { onMount } from 'svelte';
  import { logError } from '$lib/utils/log';

  let { children } = $props();

  onMount(() => {
    window.onerror = (message, source, lineno, colno, error) => {
      const stack = error?.stack || 'no stack available';
      logError('global-error', `${message} at ${source}:${lineno}:${colno}\n${stack}`);
    };

    window.addEventListener('unhandledrejection', (event) => {
      const reason = event.reason;
      const message = reason instanceof Error ? reason.message : String(reason);
      const stack = reason instanceof Error ? reason.stack : 'no stack available';
      logError('global-error', `Unhandled Promise rejection: ${message}\n${stack}`);
    });
  });
</script>

{@render children()}
