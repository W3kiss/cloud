<script lang="ts">
  import "../app.css";
  import { applySettings, loadSettings } from "$lib/settings.svelte";

  let { children } = $props();

  loadSettings();

  $effect(() => {
    applySettings();
  });
</script>

{@render children()}
