<script lang="ts">
  import "../app.css";
  import { applySettings, loadSettings } from "$lib/settings.svelte";

  import { onMount } from "svelte";
  import { api } from "$lib/api";

  let { children } = $props();

  onMount(() => {
    const report = (text: string) => void api.logFrontendError(text).catch(() => {});
    const onError = (event: ErrorEvent) =>
      report(`${event.message} @ ${event.filename}:${event.lineno}`);
    const onRejection = (event: PromiseRejectionEvent) =>
      report(`unhandled rejection: ${String(event.reason)}`);
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);
    return () => {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    };
  });

  loadSettings();

  $effect(() => {
    applySettings();
  });
</script>

{@render children()}
