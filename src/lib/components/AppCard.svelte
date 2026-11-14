<script lang="ts">
  import type { CatalogApp, JobState } from "$lib/api";
  import { logoFor } from "$lib/logos";
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let {
    app,
    index = 0,
    job = undefined,
    selected = false,
    disabled = false,
    favorite = false,
    onToggleFavorite = () => {},
    onToggleSelect,
    onDetails,
  }: {
    app: CatalogApp;
    index?: number;
    job?: JobState;
    selected?: boolean;
    disabled?: boolean;
    favorite?: boolean;
    onToggleFavorite?: (app: CatalogApp) => void;
    onToggleSelect: (app: CatalogApp) => void;
    onDetails: (app: CatalogApp) => void;
  } = $props();

  let logoFailed = $state(false);
  let logo = $derived(logoFor(app.id));

  let initials = $derived(
    app.name
      .split(/\s+/)
      .filter(Boolean)
      .map((part) => part[0])
      .join("")
      .slice(0, 2)
      .toUpperCase(),
  );

  let kindLabel = $derived(
    app.kind === "winget" ? "WINGET" : app.kind === "download" ? "DIRECT" : "CUSTOM",
  );

  let meta = $derived(
    app.wingetId ??
      (app.downloadUrl
        ? app.downloadUrl
            .replace(/^https:\/\//, "")
            .split("/")[0]
        : "—"),
  );

  let running = $derived(job?.status === "running");
  let locked = $derived(disabled || running);

  let configCount = $derived(Object.keys(app.customConfig).length);
  let configLabel = $derived(formatConfigCount(configCount));

  function formatConfigCount(count: number): string {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return `${count} параметр`;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
      return `${count} параметра`;
    }
    return `${count} параметров`;
  }
</script>

<article
  class="app-card"
  class:selected
  class:running
  style={`--i:${index};`}
  role="button"
  tabindex="0"
  aria-pressed={selected}
  onclick={() => {
    if (!locked) onToggleSelect(app);
  }}
  onkeydown={(event) => {
    if (event.target === event.currentTarget && (event.key === "Enter" || event.key === " ")) {
      event.preventDefault();
      if (!locked) onToggleSelect(app);
    }
  }}
>
  <div class="card-aura" aria-hidden="true"></div>

  <header class="card-head">
    <div class="icon-tile" class:has-logo={logo !== null && !logoFailed}>
      {#if logo !== null && !logoFailed}
        <img
          src={logo}
          alt={`Логотип ${app.name}`}
          loading="lazy"
          draggable="false"
          onerror={() => (logoFailed = true)}
        />
      {:else}
        <span>{initials}</span>
      {/if}
    </div>
    <div class="badge-row">
      <button
        type="button"
        class="fav-btn"
        class:on={favorite}
        title={favorite ? t("hub.fav.remove") : t("hub.fav.add")}
        aria-label={favorite ? t("hub.fav.remove") : t("hub.fav.add")}
        aria-pressed={favorite}
        onclick={(event) => {
          event.stopPropagation();
          onToggleFavorite(app);
        }}
        onkeydown={(event) => event.stopPropagation()}
      >
        <svg viewBox="0 0 24 24" width="15" height="15" fill={favorite ? "currentColor" : "none"} stroke="currentColor" stroke-width="1.8" stroke-linejoin="round">
          <path d="M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9L12 3Z" />
        </svg>
      </button>
      <span class="select-mark" class:on={selected} aria-hidden="true">
        {#if selected}
          <Icon name="check" size={10} stroke={2.6} />
        {/if}
      </span>
      <span class="badge kind">{kindLabel}</span>
    </div>
  </header>

  <div class="card-copy">
    <h3>{app.name}</h3>
    <p>{app.summary}</p>
  </div>

  <div class="meta-row">
    <Icon name={app.kind === "winget" ? "terminal" : "external"} size={12} />
    <code>{meta}</code>
  </div>

  {#if running}
    <ProgressBar percent={job?.percent ?? null} phase={job?.phase ?? "Установка"} />
  {:else}
    <footer class="card-foot">
      {#if true}
        <button
          type="button"
          class="action select"
          class:selected
          disabled={locked}
          aria-pressed={selected}
          onclick={(event) => {
            event.stopPropagation();
            onToggleSelect(app);
          }}
        >
          {#if selected}
            <Icon name="check" size={15} stroke={2.2} />
            Выбрано
          {:else}
            <span class="select-ring"></span>
            Выбрать
          {/if}
        </button>
      {/if}

      {#if job?.status === "done"}
        <span class="job-done">
          <Icon name="check" size={11} />
          Установлено
        </span>
      {:else if configCount > 0}
        <span class="config-count">{configLabel}</span>
      {/if}
    </footer>

    {#if job?.status === "error"}
      <p class="job-error" title={job.message ?? ""}>
        <Icon name="alert" size={12} />
        {job.message ?? "Не удалось выполнить установку"}
      </p>
    {/if}
  {/if}
</article>

<style>
  .app-card {
    position: relative;
    display: flex;
    min-height: 258px;
    flex-direction: column;
    padding: 18px;
    overflow: hidden;
    border: 1px solid rgb(var(--fg) / 0.075);
    border-radius: 22px;
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.042), transparent 38%),
      linear-gradient(180deg, rgb(var(--panel) / 0.9), rgb(var(--panel) / 0.95));
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.05),
      0 14px 36px rgba(0, 0, 0, 0.24);
    transform: translateZ(0);
    transform-origin: center center;
    transition:
      transform 280ms var(--ease-out),
      border-color 280ms ease,
      background 280ms ease,
      box-shadow 280ms ease;
    will-change: transform;
    animation: fade-up 500ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 45ms);
  }

  .app-card.template {
    border-style: dashed;
    border-color: rgb(var(--fg) / 0.13);
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.026), transparent 42%),
      linear-gradient(180deg, rgb(var(--panel) / 0.78), rgb(var(--panel) / 0.88));
  }

  .app-card::before {
    position: absolute;
    inset: -1px;
    z-index: 2;
    border-radius: inherit;
    padding: 1px;
    background: linear-gradient(
      135deg,
      rgb(var(--fg) / 0.9),
      transparent 34%,
      transparent 66%,
      rgb(var(--fg) / 0.34)
    );
    content: "";
    opacity: 0;
    transition: opacity 300ms ease;
    -webkit-mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }

  .app-card:hover {
    z-index: 3;
    border-color: rgb(var(--fg) / 0.24);
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.09),
      0 26px 54px rgba(0, 0, 0, 0.44),
      0 0 34px rgb(var(--fg) / 0.055);
    transform: translateY(-4px) scale(1.02);
  }

  .app-card:hover::before {
    opacity: 0.55;
  }

  .app-card.selected {
    border-color: rgb(var(--fg) / 0.55);
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.075), transparent 42%),
      linear-gradient(180deg, rgba(26, 26, 30, 0.94), rgb(var(--panel) / 0.97));
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.12),
      0 20px 46px rgba(0, 0, 0, 0.38),
      0 0 32px rgb(var(--fg) / 0.09);
  }

  .app-card.selected::before {
    opacity: 1;
  }

  .card-aura {
    position: absolute;
    top: -70px;
    right: -60px;
    width: 170px;
    height: 170px;
    border-radius: 50%;
    background: rgb(var(--fg) / 0.07);
    filter: blur(52px);
    opacity: 0.32;
    transition:
      opacity 320ms ease,
      transform 320ms var(--ease-out);
    pointer-events: none;
  }

  .app-card:hover .card-aura,
  .app-card.selected .card-aura {
    opacity: 0.8;
    transform: scale(1.15);
  }

  .card-head {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .icon-tile {
    position: relative;
    display: grid;
    width: 50px;
    height: 50px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 15px;
    color: rgb(var(--fg) / 0.88);
    background:
      radial-gradient(circle at 30% 16%, rgb(var(--fg) / 0.13), transparent 42%),
      linear-gradient(145deg, #252529, #111114 70%);
    box-shadow:
      0 10px 22px rgba(0, 0, 0, 0.28),
      inset 0 1px 0 rgb(var(--fg) / 0.11);
    transition:
      border-color 280ms ease,
      transform 280ms var(--ease-out);
  }

  .app-card:hover .icon-tile {
    border-color: rgb(var(--fg) / 0.3);
    transform: translateY(-1px);
  }

  .icon-tile span {
    font-size: 15px;
    font-weight: 720;
    letter-spacing: 0.04em;
  }

  .icon-tile img {
    width: 30px;
    height: 30px;
    object-fit: contain;
    pointer-events: none;
    user-select: none;
  }

  .icon-tile.has-logo img {
    filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.32));
  }

  .badge-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 5px;
  }

  .select-mark {
    display: grid;
    width: 18px;
    height: 18px;
    margin-right: 2px;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.16);
    border-radius: 50%;
    color: #09090b;
    background: transparent;
    transition:
      background 240ms ease,
      border-color 240ms ease,
      transform 240ms var(--ease-out);
  }

  .select-mark.on {
    border-color: var(--accent);
    background: var(--accent);
    transform: scale(1.04);
  }

  .badge {
    padding: 4px 7px;
    border: 1px solid rgb(var(--fg) / 0.08);
    border-radius: 999px;
    color: rgb(var(--fg) / 0.45);
    background: rgb(var(--fg) / 0.025);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.1em;
  }

  .badge.kind {
    border-color: rgb(var(--fg) / 0.1);
    color: rgb(var(--fg) / 0.56);
    background: rgb(var(--fg) / 0.035);
  }

  .template-badge {
    border-style: dashed;
    border-color: rgb(var(--fg) / 0.18);
    color: rgb(var(--fg) / 0.5);
    background: transparent;
  }

  .card-copy {
    position: relative;
    z-index: 1;
    margin-top: 16px;
  }

  .card-copy h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 620;
    letter-spacing: -0.025em;
    line-height: 1.2;
  }

  .card-copy p {
    display: -webkit-box;
    overflow: hidden;
    margin: 7px 0 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .meta-row {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 13px;
    color: rgb(var(--fg) / 0.3);
  }

  .meta-row code {
    overflow: hidden;
    font-size: 9.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-foot {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: auto;
    padding-top: 16px;
  }

  .action {
    display: inline-flex;
    height: 38px;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 0 14px;
    border-radius: 11px;
    font-size: 11px;
    font-weight: 650;
    letter-spacing: 0.01em;
    transition:
      transform 200ms var(--ease-out),
      box-shadow 200ms ease,
      background 200ms ease,
      border-color 200ms ease,
      color 200ms ease;
  }

  .action:active {
    transform: scale(0.97);
  }

  .action.select {
    border: 1px solid rgb(var(--fg) / 0.13);
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.035);
  }

  .action.select:hover:not(:disabled) {
    border-color: rgb(var(--fg) / 0.3);
    color: var(--text);
    background: rgb(var(--fg) / 0.07);
    transform: translateY(-1px);
  }

  .action.select.selected {
    border-color: rgb(var(--fg) / 0.85);
    color: #0a0a0c;
    background: linear-gradient(145deg, #ffffff, #d8d8dc);
    box-shadow:
      0 0 20px rgb(var(--fg) / 0.14),
      inset 0 1px 0 rgb(var(--fg) / 0.6);
  }

  .action.select.selected:hover:not(:disabled) {
    color: #0a0a0c;
    background: linear-gradient(145deg, #ffffff, #e7e7ea);
    box-shadow:
      0 0 28px rgb(var(--fg) / 0.22),
      inset 0 1px 0 rgb(var(--fg) / 0.7);
  }

  .action:disabled {
    opacity: 0.5;
    transform: none;
  }

  .select-ring {
    width: 12px;
    height: 12px;
    border: 1.5px solid rgb(var(--fg) / 0.42);
    border-radius: 50%;
    transition: border-color 200ms ease;
  }

  .action.select:hover:not(:disabled) .select-ring {
    border-color: rgb(var(--fg) / 0.8);
  }

  .action.ghost {
    border: 1px solid rgb(var(--fg) / 0.1);
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.035);
  }

  .action.ghost:hover {
    border-color: rgb(var(--fg) / 0.26);
    color: var(--text);
    background: rgb(var(--fg) / 0.065);
  }

  .action :global(.arrow) {
    opacity: 0.5;
    transition: transform 200ms var(--ease-out);
  }

  .action.ghost:hover :global(.arrow) {
    transform: translateX(2px);
  }

  .config-count {
    margin-left: auto;
    color: rgb(var(--fg) / 0.3);
    font-size: 9px;
    letter-spacing: 0.04em;
    white-space: nowrap;
  }

  .job-done {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-left: auto;
    color: rgb(var(--fg) / 0.58);
    font-size: 9px;
    white-space: nowrap;
  }

  .job-error {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 10px 0 0;
    overflow: hidden;
    color: #ff9d9d;
    font-size: 9.5px;
    line-height: 1.4;
    text-overflow: ellipsis;
  }

  .job-error :global(svg) {
    flex: 0 0 auto;
    margin-top: 1px;
  }

  .app-card {
    cursor: pointer;
    user-select: none;
  }

  .app-card.running {
    cursor: default;
  }

  .fav-btn {
    display: grid;
    width: 24px;
    height: 24px;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: 8px;
    color: rgb(var(--fg) / 0.4);
    background: transparent;
    cursor: pointer;
    transition: color 160ms, background 160ms, transform 160ms;
  }

  .fav-btn:hover {
    color: var(--text);
    background: rgb(var(--fg) / 0.08);
    transform: scale(1.1);
  }

  .fav-btn.on {
    color: #fbbf24;
  }
</style>
