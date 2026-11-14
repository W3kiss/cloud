<script lang="ts">
  import type { CatalogApp } from "$lib/api";
  import { logoFor } from "$lib/logos";
  import Icon from "./Icon.svelte";

  let {
    app,
    onClose,
  }: {
    app: CatalogApp;
    onClose: () => void;
  } = $props();

  let copied = $state(false);
  let logoFailed = $state(false);
  let logo = $derived(logoFor(app.id));

  let configEntries = $derived(Object.entries(app.customConfig));
  let domain = $derived(
    app.downloadUrl
      ? app.downloadUrl.replace(/^https:\/\//, "").split("/")[0]
      : "—",
  );

  async function copyUrl() {
    if (!app.downloadUrl) return;
    try {
      await navigator.clipboard.writeText(app.downloadUrl);
      copied = true;
      setTimeout(() => (copied = false), 1600);
    } catch {
      copied = false;
    }
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="overlay">
  <button
    type="button"
    class="overlay-backdrop"
    tabindex="-1"
    aria-label="Закрыть окно"
    onclick={onClose}
  ></button>
  <div
    class="details"
    role="dialog"
    aria-modal="true"
    aria-labelledby="details-title"
    tabindex="-1"
  >
    <div class="details-glow"></div>

    <header class="details-head">
      <div class="details-icon" class:has-logo={logo !== null && !logoFailed}>
        {#if logo !== null && !logoFailed}
          <img
            src={logo}
            alt={`Логотип ${app.name}`}
            draggable="false"
            onerror={() => (logoFailed = true)}
          />
        {:else}
          <Icon name="sliders" size={22} />
        {/if}
      </div>
      <div class="details-title">
        <span class="eyebrow">Параметры установки</span>
        <h2 id="details-title">{app.name}</h2>
        <p>{app.summary}</p>
      </div>
      <button type="button" class="close" aria-label="Закрыть" onclick={onClose}>
        <Icon name="x" size={17} />
      </button>
    </header>

    <div class="grid">
      <section class="panel">
        <div class="panel-head">
          <span>Источник</span>
          <span class="badge">{app.kind.toUpperCase()}</span>
        </div>
        <div class="domain">{domain}</div>
        {#if app.downloadUrl}
          <div class="url-row">
            <code>{app.downloadUrl}</code>
            <button type="button" class="copy" onclick={copyUrl}>
              <Icon name={copied ? "check" : "copy"} size={14} />
              {copied ? "Скопировано" : "Копировать"}
            </button>
          </div>
        {/if}
      </section>

      <section class="panel">
        <div class="panel-head">
          <span>Аргументы установщика</span>
          <span class="badge">{app.installerArgs.length}</span>
        </div>
        {#if app.installerArgs.length > 0}
          <div class="chips">
            {#each app.installerArgs as argument (argument)}
              <code>{argument}</code>
            {/each}
          </div>
        {:else}
          <p class="empty">Без дополнительных аргументов.</p>
        {/if}
        <p class="hint">
          Для MSI backend автоматически добавляет <code>/qn /norestart</code>.
        </p>
      </section>
    </div>

    <section class="panel config-panel">
      <div class="panel-head">
        <span>Параметры конфигурации</span>
        <span class="badge">{configEntries.length}</span>
      </div>
      <div class="config-grid">
        {#each configEntries as [key, value] (key)}
          <div class="config-item">
            <span>{key}</span>
            <code>{value}</code>
          </div>
        {/each}
      </div>
      <p class="hint">
        Параметры передаются установщику через переменную окружения
        <code>SETUP_HUB_APP_CONFIG</code> в формате JSON.
      </p>
    </section>

    <footer class="details-foot">
      <span class="foot-note">
        <Icon name="lock" size={13} />
        Значения каталога замаскированы в релизном бинарнике.
      </span>
      <button type="button" class="done" onclick={onClose}>Понятно</button>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 900;
    display: grid;
    padding: 26px;
    overflow-y: auto;
    place-items: center;
    background: rgb(var(--panel) / 0.64);
    backdrop-filter: blur(16px) saturate(120%);
    -webkit-backdrop-filter: blur(16px) saturate(120%);
    animation: fade-in 180ms ease both;
  }

  .overlay-backdrop {
    position: absolute;
    inset: 0;
    background: transparent;
    cursor: default;
  }

  .details {
    position: relative;
    width: min(720px, 100%);
    max-height: calc(100vh - 52px);
    padding: 26px;
    overflow-y: auto;
    border: 1px solid rgb(var(--fg) / 0.11);
    border-radius: 26px;
    background:
      linear-gradient(150deg, rgb(var(--fg) / 0.06), transparent 42%),
      linear-gradient(180deg, rgb(var(--panel) / 0.97), rgb(var(--panel) / 0.99));
    box-shadow:
      0 36px 100px rgba(0, 0, 0, 0.64),
      inset 0 1px 0 rgb(var(--fg) / 0.09);
    animation: scale-in 300ms var(--ease-out) both;
  }

  .details-glow {
    position: absolute;
    top: -80px;
    right: -40px;
    width: 240px;
    height: 240px;
    border-radius: 50%;
    background: rgb(var(--fg) / 0.065);
    filter: blur(70px);
    pointer-events: none;
  }

  .details-head {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 14px;
  }

  .details-icon {
    display: grid;
    width: 52px;
    height: 52px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.16);
    border-radius: 16px;
    color: var(--text);
    background:
      radial-gradient(circle at 30% 16%, rgb(var(--fg) / 0.12), transparent 42%),
      rgb(var(--fg) / 0.045);
  }

  .details-icon img {
    width: 28px;
    height: 28px;
    object-fit: contain;
    pointer-events: none;
    user-select: none;
  }

  .details-icon.has-logo img {
    filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.32));
  }

  .details-title {
    min-width: 0;
    padding-top: 2px;
  }

  .eyebrow {
    color: rgb(var(--fg) / 0.6);
    font-size: 9px;
    font-weight: 650;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .details-title h2 {
    margin: 5px 0 0;
    font-size: 22px;
    font-weight: 620;
    letter-spacing: -0.035em;
  }

  .details-title p {
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.5;
  }

  .close {
    display: grid;
    width: 34px;
    height: 34px;
    margin-left: auto;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.08);
    border-radius: 10px;
    color: var(--muted);
    background: rgb(var(--fg) / 0.035);
    transition:
      color 180ms ease,
      background 180ms ease;
  }

  .close:hover {
    color: var(--text);
    background: rgb(var(--fg) / 0.08);
  }

  .template-note {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin-top: 20px;
    padding: 12px 14px;
    border: 1px solid rgb(var(--fg) / 0.12);
    border-radius: 13px;
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.025);
    font-size: 10.5px;
    line-height: 1.55;
  }

  .template-note :global(svg) {
    flex: 0 0 auto;
    margin-top: 1px;
    color: rgb(var(--fg) / 0.66);
  }

  code {
    padding: 1px 4px;
    border-radius: 5px;
    color: var(--text);
    background: rgb(var(--fg) / 0.055);
    font-size: 9.5px;
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr);
    gap: 12px;
    margin-top: 14px;
  }

  .panel {
    padding: 15px;
    border: 1px solid rgb(var(--fg) / 0.07);
    border-radius: 16px;
    background: rgb(var(--fg) / 0.022);
  }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 11px;
    color: var(--muted-strong);
    font-size: 10.5px;
    font-weight: 600;
  }

  .badge {
    padding: 3px 7px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 999px;
    color: rgb(var(--fg) / 0.42);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8px;
    letter-spacing: 0.08em;
  }

  .domain {
    margin-bottom: 7px;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .url-row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .url-row code {
    min-width: 0;
    flex: 1;
    overflow-wrap: anywhere;
    padding: 7px 9px;
    line-height: 1.45;
  }

  .copy {
    display: inline-flex;
    height: 30px;
    flex: 0 0 auto;
    align-items: center;
    gap: 5px;
    padding: 0 9px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 8px;
    color: var(--muted);
    background: rgb(var(--fg) / 0.035);
    font-size: 9.5px;
  }

  .copy:hover {
    color: var(--text);
    background: rgb(var(--fg) / 0.07);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chips code {
    padding: 5px 8px;
  }

  .empty {
    margin: 0;
    color: rgb(var(--fg) / 0.3);
    font-size: 10px;
  }

  .hint {
    margin: 10px 0 0;
    color: rgb(var(--fg) / 0.3);
    font-size: 9px;
    line-height: 1.5;
  }

  .config-panel {
    margin-top: 12px;
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 7px;
  }

  .config-item {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 4px;
    padding: 9px 10px;
    border: 1px solid rgb(var(--fg) / 0.055);
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.18);
  }

  .config-item span {
    color: rgb(var(--fg) / 0.32);
    font-size: 8.5px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .config-item code {
    overflow: hidden;
    padding: 0;
    background: transparent;
    font-size: 9.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .details-foot {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid rgb(var(--fg) / 0.06);
  }

  .foot-note {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: rgb(var(--fg) / 0.28);
    font-size: 9px;
  }

  .done {
    height: 38px;
    margin-left: auto;
    padding: 0 20px;
    border: 1px solid rgb(var(--fg) / 0.78);
    border-radius: 11px;
    color: #09090b;
    background: linear-gradient(145deg, #ffffff, #dcdce0);
    font-size: 11px;
    font-weight: 650;
    box-shadow: 0 0 20px rgb(var(--fg) / 0.1);
  }

  .done:hover {
    box-shadow: 0 0 28px rgb(var(--fg) / 0.2);
  }

  @media (max-width: 640px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .details-foot {
      flex-wrap: wrap;
    }
  }
</style>
