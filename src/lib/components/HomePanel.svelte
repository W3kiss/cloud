<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t } from "$lib/i18n.svelte";
  import { favorites } from "$lib/favorites.svelte";
  import { APP_VERSION } from "$lib/version";
  import type { UpdateInfo } from "$lib/updates";
  import Icon from "./Icon.svelte";

  let {
    tabs,
    selectedCount,
    catalogCount,
    admin,
    update,
    checking,
    onNavigate,
    onCheckUpdate,
    onOpenPalette,
  }: {
    tabs: { id: string; label: string; hint: string; icon: string }[];
    selectedCount: number;
    catalogCount: number;
    admin: boolean;
    update: UpdateInfo | null;
    checking: boolean;
    onNavigate: (id: string) => void;
    onCheckUpdate: () => void;
    onOpenPalette: () => void;
  } = $props();

  let stats = $derived([
    { value: String(catalogCount), label: t("home.programs") },
    { value: String(selectedCount), label: t("home.selected") },
    { value: String(favorites.ids.length), label: t("home.favs") },
    { value: admin ? "Admin" : "User", label: t("home.mode") },
  ]);

  let quick = $derived(tabs.filter((tab) => tab.id !== "home" && tab.id !== "authors"));
  let tips = $derived(t("home.tips").split(" · "));
  let tip = $state(Math.floor(Math.random() * 4));

  async function openRelease() {
    if (!update) return;
    try {
      await openUrl(update.url);
    } catch {
      // Browser could not be opened.
    }
  }
</script>

<div class="home">
  <header class="hero">
    <p class="eyebrow">Setup Hub · v{APP_VERSION}</p>
    <h1>{t("home.title")}</h1>
    <p class="lead">{t("home.sub")}</p>

    <div class="hero-actions">
      <button type="button" class="pill" onclick={onOpenPalette}>
        <Icon name="command" size={15} />
        {t("home.palette")}
        <kbd>Ctrl+Shift+P</kbd>
      </button>
      <button type="button" class="pill" disabled={checking} onclick={onCheckUpdate}>
        <Icon name="refresh" size={15} />
        {t("home.update.check")}
      </button>
    </div>

    {#if update}
      <div class="update" class:new={update.newer}>
        <Icon name={update.newer ? "download" : "check"} size={16} />
        <span>
          {update.newer ? `${t("home.update.new")}: v${update.latest}` : `${t("home.update.ok")} (v${update.latest})`}
        </span>
        {#if update.newer}
          <button type="button" onclick={openRelease}>{t("home.update.open")}</button>
        {/if}
      </div>
    {/if}
  </header>

  <section class="stats" aria-label="stats">
    {#each stats as stat (stat.label)}
      <div class="stat">
        <strong>{stat.value}</strong>
        <small>{stat.label}</small>
      </div>
    {/each}
  </section>

  <section>
    <h2>{t("home.quick")}</h2>
    <div class="tiles">
      {#each quick as tab, i (tab.id)}
        <button type="button" class="tile" style={`--i:${i};`} onclick={() => onNavigate(tab.id)}>
          <span class="tile-icon"><Icon name={tab.icon} size={20} /></span>
          <strong>{tab.label}</strong>
          <small>{tab.hint}</small>
        </button>
      {/each}
    </div>
  </section>

  <section class="tip">
    <span class="tip-label">{t("home.tip")}</span>
    <p>{tips[tip % tips.length]}</p>
  </section>
</div>

<style>
  .home {
    display: grid;
    gap: 26px;
    animation: fade-up 500ms var(--ease-out) both;
  }

  .hero {
    padding: 34px 32px;
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    background:
      radial-gradient(circle at 85% 0%, rgb(var(--fg) / 0.08), transparent 45%),
      rgb(var(--panel) / 0.6);
  }

  .eyebrow {
    margin: 0 0 10px;
    color: var(--muted);
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0;
    font-size: clamp(30px, 4vw, 44px);
    letter-spacing: -0.03em;
  }

  h2 {
    margin: 0 0 12px;
    color: var(--muted-strong);
    font-size: 12px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .lead {
    margin: 10px 0 20px;
    color: var(--muted-strong);
    max-width: 52ch;
  }

  .hero-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 14px;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    color: var(--text);
    background: var(--surface);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  .pill:hover:not(:disabled) { background: var(--surface-strong); }
  .pill:disabled { opacity: 0.6; cursor: default; }

  kbd {
    padding: 1px 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    font-size: 11px;
  }

  .update {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 16px;
    padding: 10px 14px;
    border: 1px solid var(--border);
    border-radius: 12px;
    color: var(--muted-strong);
    font-size: 13px;
  }

  .update.new { border-color: var(--accent); color: var(--text); }

  .update button {
    margin-left: auto;
    padding: 5px 12px;
    border: 0;
    border-radius: 8px;
    color: #09090b;
    background: var(--accent);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .stat {
    display: grid;
    gap: 2px;
    padding: 16px 18px;
    border: 1px solid var(--border);
    border-radius: 16px;
    background: var(--surface);
  }

  .stat strong { font-size: 26px; letter-spacing: -0.02em; }
  .stat small { color: var(--muted); }

  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 12px;
  }

  .tile {
    display: grid;
    gap: 4px;
    padding: 16px;
    border: 1px solid var(--border);
    border-radius: 16px;
    color: var(--text);
    background: var(--surface);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: transform 160ms, border-color 160ms, background 160ms;
    animation: fade-up 420ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 35ms);
  }

  .tile:hover {
    transform: translateY(-2px);
    border-color: var(--border-strong);
    background: var(--surface-strong);
  }

  .tile-icon {
    display: grid;
    width: 38px;
    height: 38px;
    margin-bottom: 8px;
    place-items: center;
    border-radius: 12px;
    background: var(--surface-strong);
  }

  .tile small { color: var(--muted); }

  .tip {
    display: flex;
    align-items: baseline;
    gap: 14px;
    padding: 14px 18px;
    border: 1px dashed var(--border-strong);
    border-radius: 14px;
  }

  .tip-label {
    color: var(--muted);
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .tip p { margin: 0; color: var(--muted-strong); }
</style>
