<script lang="ts">
  import type { ToastItem } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    toasts,
    offset = 24,
    onDismiss,
  }: {
    toasts: ToastItem[];
    offset?: number;
    onDismiss: (id: number) => void;
  } = $props();

  const icons = {
    success: "check",
    error: "alert",
    info: "info",
  } as const;
</script>

<div
  class="toast-stack"
  style={`--toast-offset:${offset}px;`}
  aria-live="polite"
  aria-atomic="false"
>
  {#each toasts as toast (toast.id)}
    <div class="toast" class:success={toast.tone === "success"} class:error={toast.tone === "error"}>
      <span class="toast-icon">
        <Icon name={icons[toast.tone]} size={16} />
      </span>
      <div class="toast-copy">
        <strong>{toast.title}</strong>
        {#if toast.message}
          <p>{toast.message}</p>
        {/if}
      </div>
      <button
        type="button"
        class="toast-close"
        aria-label="Закрыть уведомление"
        onclick={() => onDismiss(toast.id)}
      >
        <Icon name="x" size={13} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    right: 24px;
    bottom: var(--toast-offset, 24px);
    z-index: 950;
    display: flex;
    width: min(360px, calc(100vw - 40px));
    flex-direction: column;
    gap: 9px;
    pointer-events: none;
    transition: bottom 260ms var(--ease-out);
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 13px 13px 13px 12px;
    border: 1px solid rgb(var(--fg) / 0.11);
    border-radius: 15px;
    background:
      linear-gradient(150deg, rgb(var(--fg) / 0.055), transparent 44%),
      rgb(var(--panel) / 0.95);
    box-shadow:
      0 18px 46px rgba(0, 0, 0, 0.48),
      inset 0 1px 0 rgb(var(--fg) / 0.07);
    backdrop-filter: blur(20px);
    animation: toast-in 260ms var(--ease-out) both;
    pointer-events: auto;
  }

  .toast.success {
    border-color: rgb(var(--fg) / 0.2);
  }

  .toast.error {
    border-color: rgba(255, 107, 107, 0.24);
  }

  .toast-icon {
    display: grid;
    width: 32px;
    height: 32px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.1);
    border-radius: 10px;
    color: var(--text);
    background: rgb(var(--fg) / 0.045);
  }

  .toast.success .toast-icon {
    color: var(--text);
    background: rgb(var(--fg) / 0.06);
  }

  .toast.error .toast-icon {
    color: var(--danger);
    background: rgba(255, 107, 107, 0.09);
  }

  .toast-copy {
    min-width: 0;
    padding-top: 1px;
  }

  .toast-copy strong {
    display: block;
    font-size: 11.5px;
    font-weight: 620;
    line-height: 1.3;
  }

  .toast-copy p {
    display: -webkit-box;
    overflow: hidden;
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 10px;
    line-height: 1.45;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
  }

  .toast-close {
    display: grid;
    width: 24px;
    height: 24px;
    flex: 0 0 auto;
    margin-left: auto;
    place-items: center;
    border-radius: 7px;
    color: rgb(var(--fg) / 0.32);
    transition:
      color 180ms ease,
      background 180ms ease;
  }

  .toast-close:hover {
    color: var(--text);
    background: rgb(var(--fg) / 0.07);
  }
</style>
