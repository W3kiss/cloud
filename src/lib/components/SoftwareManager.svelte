<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api, humanizeError, type Package, type ToastTone } from "$lib/api";
  import { THEMES, saveSettings, settings } from "$lib/settings.svelte";
  import Icon from "./Icon.svelte";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  type Tab = "updates" | "uninstall" | "presets";
  type RowState = "idle" | "running" | "done" | "error";

  let tab = $state<Tab>("updates");

  // ---- shared list state
  let updates = $state<Package[]>([]);
  let installed = $state<Package[]>([]);
  let loadingUpdates = $state(false);
  let loadingInstalled = $state(false);
  let query = $state("");
  let rowState = $state<Record<string, RowState>>({});
  let rowNote = $state<Record<string, string>>({});
  let queueRunning = $state(false);
  let stopRequested = false;

  let updatePick = $state<Record<string, boolean>>({});
  let removePick = $state<Record<string, boolean>>({});
  let confirmRemove = $state(false);

  const filteredInstalled = $derived(
    installed.filter((p) => (p.name + p.id).toLowerCase().includes(query.trim().toLowerCase())),
  );
  const pickedUpdates = $derived(updates.filter((p) => updatePick[p.id]));
  const pickedRemovals = $derived(installed.filter((p) => removePick[p.id]));

  async function loadUpdates() {
    loadingUpdates = true;
    try {
      updates = await api.listUpdates();
      updatePick = Object.fromEntries(updates.map((p) => [p.id, true]));
    } catch (caught) {
      onNotify("error", "Не удалось проверить обновления", humanizeError(caught));
    } finally {
      loadingUpdates = false;
    }
  }

  async function loadInstalled() {
    loadingInstalled = true;
    try {
      installed = await api.listInstalled();
    } catch (caught) {
      onNotify("error", "Не удалось получить список программ", humanizeError(caught));
    } finally {
      loadingInstalled = false;
    }
  }

  /** Runs items strictly one by one (winget cannot run installers in parallel). */
  async function runQueue(
    items: Package[],
    action: (id: string) => Promise<string>,
    label: string,
  ) {
    queueRunning = true;
    stopRequested = false;
    let ok = 0;
    let failed = 0;
    for (const item of items) {
      if (stopRequested) break;
      rowState[item.id] = "running";
      try {
        rowNote[item.id] = await action(item.id);
        rowState[item.id] = "done";
        ok += 1;
      } catch (caught) {
        rowNote[item.id] = humanizeError(caught);
        rowState[item.id] = "error";
        failed += 1;
      }
    }
    queueRunning = false;
    onNotify(failed ? "error" : "success", label, `Успешно: ${ok}, с ошибкой: ${failed}`);
  }

  async function updateAll() {
    await runQueue(pickedUpdates, api.upgradePackage, "Обновление завершено");
    await loadUpdates();
  }

  async function removeSelected() {
    confirmRemove = false;
    await runQueue(pickedRemovals, api.uninstallPackage, "Удаление завершено");
    removePick = {};
    await loadInstalled();
  }

  // ---- presets
  interface Preset {
    schema: 1;
    createdAt: string;
    apps: string[];
    tweaks: Record<string, boolean>;
    stripped: Record<string, boolean>;
    settings: { theme: string; animations: boolean; blur: boolean; grid: boolean; compact: boolean };
  }

  let incApps = $state(true);
  let incTweaks = $state(true);
  let incStripped = $state(false);
  let incSettings = $state(true);
  let presetBusy = $state(false);
  let presetPath = $state("");
  let loaded = $state<Preset | null>(null);
  let applyStripped = $state(false);
  let confirmApply = $state(false);
  let presetLog = $state<string[]>([]);

  const wingetApps = $derived(installed.filter((p) => p.source.toLowerCase() === "winget"));
  const ID_RE = /^[A-Za-z0-9._+-]{1,120}$/;

  async function exportPreset() {
    presetBusy = true;
    try {
      if (incApps && installed.length === 0) await loadInstalled();
      const preset: Preset = {
        schema: 1,
        createdAt: new Date().toISOString(),
        apps: incApps ? wingetApps.map((p) => p.id) : [],
        tweaks: {},
        stripped: {},
        settings: { theme: settings.theme, animations: settings.animations, blur: settings.blur, grid: settings.grid, compact: settings.compact },
      };
      if (incTweaks) {
        for (const t of await api.tweakStates()) preset.tweaks[t.id] = t.enabled;
        delete preset.tweaks["custom_hook"];
      }
      if (incStripped) {
        for (const t of await api.strippedStates()) preset.stripped[t.id] = t.enabled;
      }
      if (!incSettings) preset.settings = { ...preset.settings, theme: "" };

      const path = await save({
        title: "Сохранить пресет",
        defaultPath: "setup-hub-preset.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      await api.writePreset(path, JSON.stringify(preset, null, 2));
      onNotify("success", "Пресет сохранён", `Программ: ${preset.apps.length}`);
    } catch (caught) {
      onNotify("error", "Не удалось сохранить пресет", humanizeError(caught));
    } finally {
      presetBusy = false;
    }
  }

  function validate(raw: unknown): Preset {
    const p = raw as Partial<Preset>;
    if (!p || p.schema !== 1) throw new Error("Неизвестная версия пресета");
    const apps = Array.isArray(p.apps) ? p.apps.filter((id): id is string => typeof id === "string") : [];
    if (apps.some((id) => !ID_RE.test(id))) throw new Error("В пресете есть недопустимые идентификаторы программ");
    const flags = (value: unknown): Record<string, boolean> =>
      Object.fromEntries(
        Object.entries((value && typeof value === "object" ? value : {}) as Record<string, unknown>).filter(
          ([k, v]) => /^[a-z0-9_]{1,64}$/.test(k) && typeof v === "boolean",
        ),
      ) as Record<string, boolean>;
    const s = (p.settings ?? {}) as Partial<Preset["settings"]>;
    return {
      schema: 1,
      createdAt: String(p.createdAt ?? ""),
      apps,
      tweaks: flags(p.tweaks),
      stripped: flags(p.stripped),
      settings: {
        theme: typeof s.theme === "string" ? s.theme : "",
        animations: s.animations !== false,
        blur: s.blur !== false,
        grid: s.grid !== false,
        compact: s.compact === true,
      },
    };
  }

  async function importPreset() {
    try {
      const selected = await open({
        multiple: false,
        title: "Выберите пресет",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (typeof selected !== "string") return;
      loaded = validate(JSON.parse(await api.readPreset(selected)));
      presetPath = selected;
      presetLog = [];
      confirmApply = false;
    } catch (caught) {
      loaded = null;
      onNotify("error", "Пресет не подходит", humanizeError(caught));
    }
  }

  async function applyPreset() {
    if (!loaded) return;
    confirmApply = false;
    presetBusy = true;
    presetLog = [];
    const log = (line: string) => (presetLog = [...presetLog, line]);
    try {
      if (loaded.settings.theme && THEMES.some((t) => t.id === loaded!.settings.theme)) {
        settings.theme = loaded.settings.theme as (typeof THEMES)[number]["id"];
        settings.animations = loaded.settings.animations;
        settings.blur = loaded.settings.blur;
        settings.grid = loaded.settings.grid;
        settings.compact = loaded.settings.compact;
        saveSettings();
        log("Настройки приложения применены");
      }
      for (const [group, flags, read, write] of [
        ["Твик", loaded.tweaks, api.tweakStates, api.applyTweak],
        ...(applyStripped ? [["Stripped", loaded.stripped, api.strippedStates, api.applyStripped] as const] : []),
      ] as const) {
        if (Object.keys(flags).length === 0) continue;
        const current = new Map((await read()).map((t) => [t.id, t.enabled]));
        for (const [id, enabled] of Object.entries(flags)) {
          if (!current.has(id) || current.get(id) === enabled) continue;
          try {
            await write(id, enabled);
            log(`${group} ${id}: ${enabled ? "включён" : "выключен"}`);
          } catch (caught) {
            log(`${group} ${id}: ошибка — ${humanizeError(caught)}`);
          }
        }
      }
      for (const id of loaded.apps) {
        log(`Установка ${id}…`);
        try {
          await api.installWinget(id);
          log(`${id}: готово`);
        } catch (caught) {
          log(`${id}: ошибка — ${humanizeError(caught).split("\n")[0]}`);
        }
      }
      onNotify("success", "Пресет применён");
    } catch (caught) {
      onNotify("error", "Не удалось применить пресет", humanizeError(caught));
    } finally {
      presetBusy = false;
    }
  }

  onMount(() => {
    void loadUpdates();
    void loadInstalled();
  });
</script>

<div class="sm-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>Software management</p>
      <h1>Управление <span>ПО</span></h1>
      <p class="lead">Обновление, массовое удаление и перенос настроек на новый ПК. Все операции выполняются через winget в фоне.</p>
    </div>
    <img class="hero-art" src="/art/hero-apps.svg" alt="" />
  </header>

  <div class="tabs">
    <button type="button" class:active={tab === "updates"} onclick={() => (tab = "updates")}><Icon name="refresh" size={15} />Обновления{updates.length ? ` · ${updates.length}` : ""}</button>
    <button type="button" class:active={tab === "uninstall"} onclick={() => (tab = "uninstall")}><Icon name="x" size={15} />Удаление</button>
    <button type="button" class:active={tab === "presets"} onclick={() => (tab = "presets")}><Icon name="archive" size={15} />Пресеты</button>
  </div>

  {#if tab === "updates"}
    <div class="toolbar">
      <span class="muted">{loadingUpdates ? "Проверяем обновления…" : updates.length ? `Доступно обновлений: ${updates.length}` : "Все программы актуальны."}</span>
      <div class="spacer"></div>
      <button type="button" class="btn" disabled={loadingUpdates || queueRunning} onclick={loadUpdates}><Icon name="refresh" size={15} />Проверить</button>
      {#if queueRunning}
        <button type="button" class="btn" onclick={() => (stopRequested = true)}>Остановить после текущего</button>
      {:else}
        <button type="button" class="btn primary" disabled={pickedUpdates.length === 0} onclick={updateAll}><Icon name="download" size={15} />Обновить выбранное ({pickedUpdates.length})</button>
      {/if}
    </div>
    <div class="list">
      {#each updates as p (p.id)}
        <label class="row-item" class:active={updatePick[p.id]}>
          <input type="checkbox" bind:checked={updatePick[p.id]} disabled={queueRunning} />
          <span class="meta"><strong>{p.name}</strong><small>{p.id}</small></span>
          <span class="ver">{p.version} → <b>{p.available}</b></span>
          <span class="state {rowState[p.id] ?? 'idle'}" title={rowNote[p.id] ?? ""}>{rowState[p.id] === "running" ? "…" : rowState[p.id] === "done" ? "✓" : rowState[p.id] === "error" ? "!" : ""}</span>
        </label>
      {/each}
    </div>
  {:else if tab === "uninstall"}
    <div class="toolbar">
      <input class="search" placeholder="Поиск установленных программ…" bind:value={query} />
      <button type="button" class="btn" disabled={loadingInstalled || queueRunning} onclick={loadInstalled}><Icon name="refresh" size={15} />{loadingInstalled ? "Загрузка…" : "Обновить"}</button>
    </div>
    <div class="list">
      {#each filteredInstalled as p (p.id)}
        <label class="row-item" class:active={removePick[p.id]}>
          <input type="checkbox" bind:checked={removePick[p.id]} disabled={queueRunning} />
          <span class="meta"><strong>{p.name}</strong><small>{p.id}</small></span>
          <span class="ver">{p.version}</span>
          <span class="state {rowState[p.id] ?? 'idle'}" title={rowNote[p.id] ?? ""}>{rowState[p.id] === "running" ? "…" : rowState[p.id] === "done" ? "✓" : rowState[p.id] === "error" ? "!" : ""}</span>
        </label>
      {:else}
        <p class="empty">{loadingInstalled ? "Загружаем список…" : "Ничего не найдено."}</p>
      {/each}
    </div>
    <div class="actions">
      {#if confirmRemove}
        <div class="confirm">
          <Icon name="alert" size={18} />
          <span>Будет удалено программ: <b>{pickedRemovals.length}</b>. Действие необратимо.</span>
          <button type="button" class="btn danger" onclick={removeSelected}>Удалить</button>
          <button type="button" class="btn" onclick={() => (confirmRemove = false)}>Отмена</button>
        </div>
      {:else}
        <span class="muted">Выбрано: {pickedRemovals.length}</span>
        <button type="button" class="btn primary" disabled={queueRunning || pickedRemovals.length === 0} onclick={() => (confirmRemove = true)}>Удалить выбранное</button>
      {/if}
    </div>
  {:else}
    <div class="card">
      <strong>Экспорт</strong>
      <div class="checks">
        <label><input type="checkbox" bind:checked={incApps} />Установленные программы (winget{wingetApps.length ? `: ${wingetApps.length}` : ""})</label>
        <label><input type="checkbox" bind:checked={incTweaks} />Твики Windows</label>
        <label><input type="checkbox" bind:checked={incStripped} />Stripped-down (бета)</label>
        <label><input type="checkbox" bind:checked={incSettings} />Настройки приложения (тема и т.д.)</label>
      </div>
      <button type="button" class="btn primary" disabled={presetBusy} onclick={exportPreset}><Icon name="download" size={15} />Сохранить пресет (.json)</button>
    </div>

    <div class="card">
      <strong>Импорт</strong>
      <span class="muted">Развернуть настройки на чистой Windows: программы, твики и тема одним действием.</span>
      <button type="button" class="btn" disabled={presetBusy} onclick={importPreset}><Icon name="folder" size={15} />Выбрать пресет</button>
      {#if loaded}
        <div class="summary">
          <span>{presetPath.split(/[\\/]/).pop()}</span>
          <span>Программ: {loaded.apps.length} · Твиков: {Object.keys(loaded.tweaks).length} · Stripped: {Object.keys(loaded.stripped).length}</span>
          {#if Object.keys(loaded.stripped).length}
            <label class="check"><input type="checkbox" bind:checked={applyStripped} />Применить и Stripped-down (бета, рискованно)</label>
          {/if}
        </div>
        {#if confirmApply}
          <div class="confirm">
            <Icon name="alert" size={18} />
            <span>Будут установлены программы и изменены настройки Windows. Продолжить?</span>
            <button type="button" class="btn danger" onclick={applyPreset}>Применить</button>
            <button type="button" class="btn" onclick={() => (confirmApply = false)}>Отмена</button>
          </div>
        {:else}
          <button type="button" class="btn primary" disabled={presetBusy} onclick={() => (confirmApply = true)}>{presetBusy ? "Применяем…" : "Применить пресет"}</button>
        {/if}
      {/if}
      {#if presetLog.length}
        <pre class="log">{presetLog.join("\n")}</pre>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sm-page { padding-bottom: 40px; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 640px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .tabs { display: flex; gap: 8px; margin: 22px 4px 14px; flex-wrap: wrap; }
  .tabs button { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; font-size: 13px; color: var(--muted-strong); border: 1px solid var(--border); border-radius: 99px; }
  .tabs button.active { background: var(--text); color: var(--bg); border-color: var(--text); }
  .toolbar { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; margin: 0 4px 14px; }
  .spacer { flex: 1; }
  .search { flex: 1; min-width: 200px; padding: 10px 14px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); outline: none; user-select: text; }
  .list { display: grid; gap: 8px; }
  .row-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--surface); cursor: pointer; }
  .row-item.active { border-color: var(--border-strong); background: var(--surface-strong); }
  .meta { flex: 1; min-width: 0; display: grid; gap: 2px; }
  .meta strong, .meta small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small, .muted { color: var(--muted); font-size: 12px; }
  .ver { font-size: 12px; color: var(--muted-strong); white-space: nowrap; }
  .state { width: 18px; text-align: center; font-weight: 700; }
  .state.done { color: #4ade80; }
  .state.error { color: var(--danger); }
  .empty { margin: 8px 4px; color: var(--muted); }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font-size: 13px; background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.primary { background: var(--text); color: var(--bg); font-weight: 600; }
  .btn.danger { background: var(--danger); color: #111; border-color: var(--danger); font-weight: 600; }
  .actions { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 18px 4px 0; flex-wrap: wrap; }
  .confirm { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; padding: 14px 16px; width: 100%; border-radius: var(--radius-md); border: 1px solid color-mix(in srgb, var(--danger) 50%, transparent); color: var(--danger); }
  .confirm span { flex: 1; min-width: 240px; color: var(--text); }
  .card { padding: 18px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 12px; margin-bottom: 14px; justify-items: start; }
  .checks { display: grid; gap: 8px; font-size: 13px; }
  .checks label, .check { display: flex; align-items: center; gap: 8px; }
  .summary { display: grid; gap: 6px; font-size: 13px; }
  .log { width: 100%; max-height: 240px; overflow: auto; margin: 0; padding: 12px; border-radius: var(--radius-md); background: var(--bg-soft); font-size: 12px; white-space: pre-wrap; user-select: text; }
</style>
