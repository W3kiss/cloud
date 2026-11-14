<script lang="ts">
  import { tick } from "svelte";
  import { t } from "$lib/i18n.svelte";
  import { THEMES, settings, saveSettings } from "$lib/settings.svelte";
  import type { CatalogApp } from "$lib/api";
  import Icon from "./Icon.svelte";

  type Entry = { id: string; group: string; label: string; hint: string; icon: string; run: () => void };

  let {
    open,
    tabs,
    apps,
    onClose,
    onNavigate,
    onToggleApp,
    onCheckUpdate,
  }: {
    open: boolean;
    tabs: { id: string; label: string; hint: string; icon: string }[];
    apps: CatalogApp[];
    onClose: () => void;
    onNavigate: (id: string) => void;
    onToggleApp: (app: CatalogApp) => void;
    onCheckUpdate: () => void;
  } = $props();

  let query = $state("");
  let cursor = $state(0);
  let input = $state<HTMLInputElement>();
  let list = $state<HTMLDivElement>();

  let entries = $derived.by<Entry[]>(() => {
    const result: Entry[] = [];
    for (const tab of tabs) {
      result.push({ id: `tab:${tab.id}`, group: t("pal.section"), label: tab.label, hint: tab.hint, icon: tab.icon, run: () => onNavigate(tab.id) });
    }
    result.push({ id: "act:update", group: t("pal.actions"), label: t("pal.update"), hint: "GitHub", icon: "refresh", run: onCheckUpdate });
    for (const theme of THEMES) {
      result.push({
        id: `theme:${theme.id}`,
        group: t("pal.actions"),
        label: `${t("pal.theme")}: ${t(`theme.${theme.id}`)}`,
        hint: t(`theme.${theme.id}.hint`),
        icon: "settings",
        run: () => {
          settings.theme = theme.id;
          saveSettings();
        },
      });
    }
    for (const app of apps) {
      result.push({ id: `app:${app.id}`, group: t("pal.apps"), label: app.name, hint: app.wingetId ?? app.summary, icon: "download", run: () => onToggleApp(app) });
    }
    return result;
  });

  let results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return entries.filter((entry) => entry.group === t("pal.section"));
    return entries
      .filter((entry) => `${entry.label} ${entry.hint}`.toLowerCase().includes(q))
      .slice(0, 40);
  });

  $effect(() => {
    if (open) {
      query = "";
      cursor = 0;
      void tick().then(() => input?.focus());
    }
  });

  $effect(() => {
    void query;
    cursor = 0;
  });

  function choose(entry: Entry | undefined) {
    if (!entry) return;
    onClose();
    entry.run();
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      cursor = Math.min(cursor + 1, results.length - 1);
      scrollToCursor();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      cursor = Math.max(cursor - 1, 0);
      scrollToCursor();
    } else if (event.key === "Enter") {
      event.preventDefault();
      choose(results[cursor]);
    }
  }

  function scrollToCursor() {
    void tick().then(() => list?.querySelector<HTMLElement>(".row.active")?.scrollIntoView({ block: "nearest" }));
  }
</script>

{#if open}
  <div class="pal-layer" role="presentation" onmousedown={(event) => event.target === event.currentTarget && onClose()}>
    <div class="pal" role="dialog" aria-modal="true" aria-label={t("home.palette")}>
      <div class="pal-search">
        <Icon name="command" size={17} />
        <input
          bind:this={input}
          bind:value={query}
          onkeydown={onKey}
          spellcheck="false"
          placeholder={t("pal.placeholder")}
          aria-label={t("pal.placeholder")}
        />
      </div>

      <div class="pal-list" bind:this={list}>
        {#if results.length === 0}
          <p class="empty">{t("pal.empty")}</p>
        {:else}
          {#each results as entry, i (entry.id)}
            {#if i === 0 || results[i - 1].group !== entry.group}
              <div class="group">{entry.group}</div>
            {/if}
            <button
              type="button"
              class="row"
              class:active={i === cursor}
              onmousemove={() => (cursor = i)}
              onclick={() => choose(entry)}
            >
              <span class="ico"><Icon name={entry.icon} size={15} /></span>
              <span class="lbl">{entry.label}</span>
              <small>{entry.hint}</small>
            </button>
          {/each}
        {/if}
      </div>

      <div class="pal-foot">{t("pal.hint")}</div>
    </div>
  </div>
{/if}

<style>
  .pal-layer {
    position: fixed;
    inset: 0;
    z-index: 900;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(6px);
    animation: fade-in 140ms ease both;
  }

  .pal {
    width: min(620px, calc(100vw - 32px));
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: 18px;
    background: rgb(var(--panel) / 0.98);
    box-shadow: 0 40px 100px rgba(0, 0, 0, 0.6);
    animation: scale-in 180ms var(--ease-out) both;
  }

  .pal-search {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    color: var(--muted-strong);
  }

  .pal-search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    color: var(--text);
    background: transparent;
    font: inherit;
    font-size: 15px;
  }

  .pal-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 6px;
  }

  .group {
    padding: 10px 10px 4px;
    color: var(--muted);
    font-size: 10px;
    font-weight: 650;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 9px 10px;
    border: 0;
    border-radius: 10px;
    color: var(--text);
    background: transparent;
    font: inherit;
    font-size: 14px;
    text-align: left;
    cursor: pointer;
  }

  .row.active {
    background: var(--surface-strong);
  }

  .ico {
    display: grid;
    width: 26px;
    height: 26px;
    flex: none;
    place-items: center;
    border-radius: 8px;
    background: var(--surface);
    color: var(--muted-strong);
  }

  .lbl {
    flex: none;
    max-width: 55%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row small {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--muted);
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    padding: 24px;
    color: var(--muted);
    text-align: center;
  }

  .pal-foot {
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    color: var(--muted);
    font-size: 11px;
  }
</style>
