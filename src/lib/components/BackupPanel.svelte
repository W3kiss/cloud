<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    api,
    humanizeError,
    type AppFolder,
    type BackupManifest,
    type ToastTone,
  } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  type Mode = "backup" | "restore";

  let mode = $state<Mode>("backup");
  let folders = $state<AppFolder[]>([]);
  let scanning = $state(false);
  let query = $state("");
  let onlyKnown = $state(false);
  let includeCache = $state(false);
  let picked = $state<Record<string, boolean>>({});

  let archivePath = $state("");
  let manifest = $state<BackupManifest | null>(null);
  let restorePicked = $state<Record<string, boolean>>({});
  let confirmingRestore = $state(false);

  let busy = $state(false);
  let percent = $state(0);
  let phase = $state("");
  let message = $state("");
  let failure = $state("");

  const keyOf = (root: string, name: string) => `${root}/${name}`;

  const visible = $derived(
    folders.filter(
      (f) =>
        (!onlyKnown || f.known) &&
        (f.name + (f.known ?? "")).toLowerCase().includes(query.trim().toLowerCase()),
    ),
  );
  const selectedFolders = $derived(folders.filter((f) => picked[keyOf(f.root, f.name)]));
  const selectedMb = $derived(selectedFolders.reduce((sum, f) => sum + f.sizeMb, 0));
  const restoreSelection = $derived(
    manifest?.entries.filter((e) => restorePicked[keyOf(e.root, e.name)]) ?? [],
  );

  function formatSize(mb: number): string {
    return mb >= 1024 ? `${(mb / 1024).toFixed(1)} ГБ` : `${mb.toFixed(1)} МБ`;
  }

  function formatDate(seconds: number): string {
    return seconds ? new Date(seconds * 1000).toLocaleDateString("ru-RU") : "—";
  }

  async function scan() {
    scanning = true;
    try {
      folders = await api.scanAppdata();
    } catch (caught) {
      onNotify("error", "Не удалось просканировать AppData", humanizeError(caught));
    } finally {
      scanning = false;
    }
  }

  function begin(label: string) {
    busy = true;
    percent = 0;
    phase = label;
    message = "";
    failure = "";
  }

  async function createBackup() {
    if (selectedFolders.length === 0) return;
    let destination: string | null = null;
    try {
      destination = await save({
        title: "Сохранить резервную копию",
        defaultPath: `setup-hub-backup-${new Date().toISOString().slice(0, 10)}.zip`,
        filters: [{ name: "ZIP", extensions: ["zip"] }],
      });
    } catch (caught) {
      onNotify("error", "Не удалось выбрать файл", String(caught));
      return;
    }
    if (!destination) return;
    begin("Подготовка");
    try {
      message = await api.createBackup(
        selectedFolders.map((f) => ({ root: f.root, name: f.name })),
        destination,
        includeCache,
      );
      onNotify("success", "Резервная копия создана");
    } catch (caught) {
      failure = humanizeError(caught);
      onNotify("error", "Не удалось создать копию", failure);
    } finally {
      busy = false;
    }
  }

  async function pickArchive() {
    try {
      const selected = await open({
        multiple: false,
        title: "Выберите архив резервной копии",
        filters: [{ name: "ZIP", extensions: ["zip"] }],
      });
      if (typeof selected !== "string") return;
      archivePath = selected;
      manifest = null;
      failure = "";
      message = "";
      confirmingRestore = false;
      manifest = await api.inspectBackup(selected);
      restorePicked = Object.fromEntries(
        manifest.entries.map((e) => [keyOf(e.root, e.name), true]),
      );
    } catch (caught) {
      manifest = null;
      failure = humanizeError(caught);
      onNotify("error", "Архив не подходит", failure);
    }
  }

  async function restore() {
    confirmingRestore = false;
    if (!archivePath || restoreSelection.length === 0) return;
    begin("Подготовка");
    try {
      message = await api.restoreArchive(
        archivePath,
        restoreSelection.map((e) => ({ root: e.root, name: e.name })),
      );
      onNotify("success", "Настройки восстановлены");
    } catch (caught) {
      failure = humanizeError(caught);
      onNotify("error", "Не удалось восстановить", failure);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    void listen<{ percent: number; phase: string }>("backup-progress", (event) => {
      percent = event.payload.percent;
      phase = event.payload.phase;
    }).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    void scan();
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<div class="bk-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>AppData backup</p>
      <h1>Резервные <span>копии</span></h1>
      <p class="lead">
        Сохраните настройки программ из AppData в один архив и восстановите их на любом ПК.
      </p>
    </div>
    <img class="hero-art" src="/art/hero-backup.svg" alt="" />
  </header>

  <div class="tabs">
    <button type="button" class:active={mode === "backup"} onclick={() => (mode = "backup")}>
      <Icon name="archive" size={15} />Создать копию
    </button>
    <button type="button" class:active={mode === "restore"} onclick={() => (mode = "restore")}>
      <Icon name="refresh" size={15} />Восстановить
    </button>
  </div>

  {#if mode === "backup"}
    <div class="toolbar">
      <input class="search" placeholder="Поиск программы…" bind:value={query} />
      <label class="check"><input type="checkbox" bind:checked={onlyKnown} />Только известные</label>
      <label class="check"><input type="checkbox" bind:checked={includeCache} />Включая кэш</label>
      <button type="button" class="btn" disabled={scanning || busy} onclick={scan}>
        <Icon name="refresh" size={15} />{scanning ? "Сканирование…" : "Обновить"}
      </button>
    </div>

    <div class="grid">
      {#each visible as f, i (keyOf(f.root, f.name))}
        {@const key = keyOf(f.root, f.name)}
        <button
          type="button"
          class="folder"
          class:active={picked[key]}
          style="--i:{Math.min(i, 14)};"
          disabled={busy}
          onclick={() => (picked[key] = !picked[key])}
        >
          <span class="box">{#if picked[key]}<Icon name="check" size={13} stroke={2.6} />{/if}</span>
          <span class="meta">
            <strong>{f.known ?? f.name}</strong>
            <small>
              {f.root === "roaming" ? "Roaming" : "Local"} · {f.name} · {formatSize(f.sizeMb)} · {f.files} файл.
            </small>
          </span>
          {#if f.running}<em class="warn">запущено</em>{/if}
        </button>
      {:else}
        <p class="empty">{scanning ? "Сканируем AppData…" : "Ничего не найдено."}</p>
      {/each}
    </div>

    <div class="actions">
      <span class="muted">Выбрано: {selectedFolders.length} · {formatSize(selectedMb)}</span>
      <button type="button" class="btn primary" disabled={busy || selectedFolders.length === 0} onclick={createBackup}>
        <Icon name="archive" size={16} />Создать архив
      </button>
    </div>
  {:else}
    <div class="card row">
      <div class="grow">
        <strong>{archivePath ? archivePath.split(/[\\/]/).pop() : "Архив не выбран"}</strong>
        <span class="muted">
          {manifest
            ? `Создан ${formatDate(manifest.created)} на ПК ${manifest.computer || "—"}`
            : "Выберите .zip, созданный Setup Hub."}
        </span>
      </div>
      <button type="button" class="btn" disabled={busy} onclick={pickArchive}>
        <Icon name="folder" size={15} />Выбрать архив
      </button>
    </div>

    {#if manifest}
      <div class="grid">
        {#each manifest.entries as e (keyOf(e.root, e.name))}
          {@const key = keyOf(e.root, e.name)}
          <button type="button" class="folder" class:active={restorePicked[key]} disabled={busy}
            onclick={() => (restorePicked[key] = !restorePicked[key])}>
            <span class="box">{#if restorePicked[key]}<Icon name="check" size={13} stroke={2.6} />{/if}</span>
            <span class="meta">
              <strong>{e.name}</strong>
              <small>{e.root === "roaming" ? "Roaming" : "Local"} · {formatSize(e.bytes / 1048576)} · {e.files} файл.</small>
            </span>
          </button>
        {/each}
      </div>

      <div class="actions">
        {#if confirmingRestore}
          <div class="confirm">
            <Icon name="alert" size={18} />
            <span>Закройте эти программы. Текущие настройки будут переименованы в <b>*.setuphub-bak-*</b>, а затем заменены.</span>
            <button type="button" class="btn danger" onclick={restore}>Восстановить</button>
            <button type="button" class="btn" onclick={() => (confirmingRestore = false)}>Отмена</button>
          </div>
        {:else}
          <span class="muted">К восстановлению: {restoreSelection.length}</span>
          <button type="button" class="btn primary" disabled={busy || restoreSelection.length === 0} onclick={() => (confirmingRestore = true)}>
            <Icon name="refresh" size={16} />Восстановить выбранное
          </button>
        {/if}
      </div>
    {/if}
  {/if}

  {#if busy || message || failure}
    <div class="card progress">
      <div class="row"><strong>{busy ? phase : failure ? "Ошибка" : "Готово"}</strong><span class="muted">{Math.round(percent)}%</span></div>
      <div class="bar"><div class="fill" style="width:{percent}%"></div></div>
      {#if message}<p class="ok">{message}</p>{/if}
      {#if failure}<p class="err">{failure}</p>{/if}
    </div>
  {/if}
</div>

<style>
  .bk-page { padding-bottom: 40px; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 640px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .tabs { display: flex; gap: 8px; margin: 22px 4px 14px; }
  .tabs button { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; font-size: 13px; color: var(--muted-strong); border: 1px solid var(--border); border-radius: 99px; }
  .tabs button.active { background: var(--text); color: var(--bg); border-color: var(--text); }
  .toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; margin: 0 4px 14px; }
  .search { flex: 1; min-width: 200px; padding: 10px 14px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); outline: none; user-select: text; }
  .check { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted-strong); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px; }
  .folder { display: flex; align-items: center; gap: 12px; text-align: left; padding: 12px 14px; border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--surface); animation: fade-up 400ms var(--ease-out) both; animation-delay: calc(var(--i, 0) * 30ms); }
  .folder:hover:not(:disabled) { border-color: var(--border-strong); }
  .folder.active { border-color: var(--text); background: var(--surface-strong); }
  .box { width: 20px; height: 20px; flex: none; display: grid; place-items: center; border-radius: 6px; border: 1px solid var(--border-strong); }
  .folder.active .box { background: var(--text); color: var(--bg); }
  .meta { flex: 1; min-width: 0; display: grid; gap: 2px; }
  .meta strong, .meta small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small, .muted { color: var(--muted); font-size: 12px; }
  .warn { font-style: normal; font-size: 11px; color: var(--danger); border: 1px solid color-mix(in srgb, var(--danger) 50%, transparent); border-radius: 99px; padding: 2px 8px; }
  .empty { grid-column: 1 / -1; color: var(--muted); margin: 8px 4px; }
  .card { padding: 16px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 12px; margin-bottom: 14px; }
  .row { display: flex; align-items: center; gap: 14px; }
  .grow { flex: 1; min-width: 0; display: grid; gap: 3px; }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font-size: 13px; background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.primary { background: var(--text); color: var(--bg); font-weight: 600; }
  .btn.danger { background: var(--danger); color: #111; border-color: var(--danger); font-weight: 600; }
  .actions { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 18px 4px 0; flex-wrap: wrap; }
  .confirm { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; padding: 14px 16px; width: 100%; border-radius: var(--radius-md); border: 1px solid color-mix(in srgb, var(--danger) 50%, transparent); color: var(--danger); }
  .confirm span { flex: 1; min-width: 240px; color: var(--text); }
  .progress { margin-top: 20px; }
  .bar { height: 8px; border-radius: 99px; background: var(--surface-strong); overflow: hidden; }
  .fill { height: 100%; background: var(--text); transition: width 300ms var(--ease-out); }
  .ok { margin: 0; color: var(--success); word-break: break-word; }
  .err { margin: 0; color: var(--danger); white-space: pre-wrap; }
</style>
