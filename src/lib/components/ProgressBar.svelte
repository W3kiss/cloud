<script lang="ts">
  let {
    percent = null,
    phase = "Установка",
  }: { percent?: number | null; phase?: string } = $props();

  let rounded = $derived(
    percent === null ? null : Math.max(0, Math.min(100, Math.round(percent))),
  );
  let indeterminate = $derived(percent === null);
</script>

<div class="progress" aria-live="polite">
  <div class="progress-head">
    <span class="phase">
      <span class="pulse"></span>
      {phase}
    </span>
    <span class="value">{rounded === null ? "···" : `${rounded}%`}</span>
  </div>
  <div class="track" class:indeterminate>
    {#if indeterminate}
      <span class="fill indeterminate-fill"></span>
    {:else}
      <span class="fill" style={`width:${rounded}%`}></span>
    {/if}
  </div>
</div>

<style>
  .progress {
    margin-top: 14px;
    padding: 12px 14px;
    border: 1px solid rgb(var(--fg) / 0.08);
    border-radius: 14px;
    background: rgb(var(--panel) / 0.55);
  }

  .progress-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 9px;
    font-size: 11px;
    letter-spacing: 0.03em;
  }

  .phase {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    overflow: hidden;
    color: var(--muted-strong);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pulse {
    width: 6px;
    height: 6px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 10px rgb(var(--fg) / 0.72);
    animation: pulse-dot 1.3s ease-in-out infinite;
  }

  .value {
    flex: 0 0 auto;
    color: var(--text);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 10px;
    letter-spacing: 0.08em;
  }

  .track {
    position: relative;
    height: 6px;
    overflow: hidden;
    border-radius: 999px;
    background: rgb(var(--fg) / 0.075);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.55);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: inherit;
    background: linear-gradient(
      100deg,
      #7c7c84 0%,
      #d9d9de 20%,
      #ffffff 45%,
      #d9d9de 70%,
      #7c7c84 100%
    );
    background-size: 260% 100%;
    animation: progress-shift 1.7s linear infinite;
    box-shadow:
      0 0 12px rgb(var(--fg) / 0.32),
      0 0 26px rgb(var(--fg) / 0.12);
    transition: width 220ms linear;
  }

  .indeterminate .fill {
    inset: 0 auto 0 0;
    width: 46%;
    animation:
      progress-shift 1.2s linear infinite,
      progress-indeterminate 1.45s cubic-bezier(0.45, 0, 0.55, 1) infinite;
  }

  .indeterminate-fill {
    width: 46%;
  }
</style>
