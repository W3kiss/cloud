<script lang="ts">
  let {
    checked,
    disabled = false,
    loading = false,
    label,
    onChange,
  }: {
    checked: boolean;
    disabled?: boolean;
    loading?: boolean;
    label: string;
    onChange: (next: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  class="toggle"
  class:checked
  class:loading
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onChange(!checked)}
>
  <span class="track">
    <span class="shine"></span>
    <span class="thumb">
      {#if loading}
        <span class="spinner"></span>
      {:else if checked}
        <svg viewBox="0 0 12 12" aria-hidden="true">
          <path d="M2.5 6.3 4.8 8.5 9.5 3.6" />
        </svg>
      {/if}
    </span>
  </span>
</button>

<style>
  .toggle {
    flex: 0 0 auto;
    padding: 0;
    border-radius: 999px;
  }

  .toggle:disabled {
    opacity: 0.55;
  }

  .track {
    position: relative;
    display: block;
    width: 54px;
    height: 30px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent),
      rgba(9, 9, 12, 0.9);
    box-shadow:
      inset 0 2px 5px rgba(0, 0, 0, 0.5),
      0 0 0 0 rgba(255, 255, 255, 0);
    transition:
      background 320ms var(--ease-out),
      border-color 320ms var(--ease-out),
      box-shadow 320ms var(--ease-out);
  }

  .toggle.checked .track {
    border-color: rgba(255, 255, 255, 0.7);
    background:
      linear-gradient(100deg, #d9d9de, #ffffff 55%, #e4e4e7),
      rgba(255, 255, 255, 0.2);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.6),
      0 0 18px rgba(255, 255, 255, 0.2);
  }

  .shine {
    position: absolute;
    inset: 0;
    opacity: 0;
    background: linear-gradient(
      110deg,
      transparent 20%,
      rgba(255, 255, 255, 0.55) 48%,
      transparent 72%
    );
    transform: translateX(-100%);
    transition: opacity 220ms ease;
  }

  .toggle.checked .shine {
    opacity: 1;
    animation: shimmer 1.8s ease-in-out infinite;
  }

  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    display: grid;
    width: 22px;
    height: 22px;
    place-items: center;
    border-radius: 50%;
    background: linear-gradient(180deg, #fafafa, #cfcfd4);
    box-shadow:
      0 2px 5px rgba(0, 0, 0, 0.45),
      inset 0 1px 0 rgba(255, 255, 255, 0.9);
    transition:
      transform 340ms var(--ease-out),
      background 340ms var(--ease-out);
  }

  .toggle.checked .thumb {
    background: linear-gradient(180deg, #17171a, #09090b);
    transform: translateX(24px);
  }

  .thumb svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: #ffffff;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .spinner {
    width: 11px;
    height: 11px;
    border: 1.5px solid rgba(255, 255, 255, 0.24);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
</style>
