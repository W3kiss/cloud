<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    humanizeError,
    type JunkCategory,
    type MenuItem,
    type ServiceInfo,
    type StartupItem,
    type ToastTone,
  } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  type Tab = "services" | "startup" | "cleaner" | "menu";
  let tab = $state<Tab>("services");
  let query = $state("");
  let busyKey = $state("");

  // ---------------------------------------------------------------- services
  let services = $state<ServiceInfo[]>([]);
  let loadingServices = $state(false);
  let onlyThird = $state(true);

  const shownServices = $derived(
    services.filter(
      (s) =>
        (!onlyThird || s.thirdParty) &&
        (s.display + s.name).toLowerCase().includes(query.trim().toLowerCase()),
    ),
  );

  function modeOf(s: ServiceInfo): string {
    const m = s.mode.toLowerCase();
    if (m === "auto") return s.delayed ? "delayed" : "auto";
    if (m === "manual") return "manual";
    if (m === "disabled") return "disabled";
    return m;
  }

  async function loadServices() {
    loadingServices = true;
    try {
      services = await api.listServices();
    } catch (caught) {
      onNotify("error", "Не удалось получить службы", humanizeError(caught));
    } finally {
      loadingServices = false;
    }
  }

  async function changeService(s: ServiceInfo, mode: string) {
    busyKey = s.name;
    try {
      await api.setServiceMode(s.name, mode);
      onNotify("success", s.display, "Режим запуска изменён");
      await loadServices();
    } catch (caught) {
      onNotify("error", `Не удалось изменить: ${s.display}`, humanizeError(caught));
    } finally {
      busyKey = "";
    }
  }

  // ---------------------------------------------------------------- startup
  let startup = $state<StartupItem[]>([]);
  let loadingStartup = $state(false);

  async function loadStartup() {
    loadingStartup = true;
    try {
      startup = await api.listStartup();
    } catch (caught) {
      onNotify("error", "Не удалось получить автозагрузку", humanizeError(caught));
    } finally {
      loadingStartup = false;
    }
  }

  async function toggleStartup(item: StartupItem) {
    busyKey = item.id;
    try {
      await api.setStartupEnabled(item.location, item.name, !item.enabled);
      item.enabled = !item.enabled;
    } catch (caught) {
      onNotify("error", `Не удалось изменить: ${item.name}`, humanizeError(caught));
    } finally {
      busyKey = "";
    }
  }

  // ---------------------------------------------------------------- cleaner
  let junk = $state<JunkCategory[]>([]);
  let junkPick = $state<Record<string, boolean>>({});
  let scanningJunk = $state(false);
  let cleaning = $state(false);
  let cleanResult = $state("");

  const junkTotal = $derived(
    junk.filter((j) => junkPick[j.id]).reduce((sum, j) => sum + j.sizeMb, 0),
  );

  function fmt(mb: number): string {
    return mb >= 1024 ? `${(mb / 1024).toFixed(2)} ГБ` : `${mb.toFixed(1)} МБ`;
  }

  async function scanJunk() {
    scanningJunk = true;
    cleanResult = "";
    try {
      junk = await api.scanJunk();
      junkPick = Object.fromEntries(junk.map((j) => [j.id, j.recommended && j.files > 0]));
    } catch (caught) {
      onNotify("error", "Сканирование не удалось", humanizeError(caught));
    } finally {
      scanningJunk = false;
    }
  }

  async function cleanJunk() {
    cleaning = true;
    try {
      const report = await api.cleanJunk(junk.filter((j) => junkPick[j.id]).map((j) => j.id));
      cleanResult =
        `Освобождено: ${fmt(report.freedMb)}` +
        (report.skippedFiles ? `. Пропущено занятых файлов: ${report.skippedFiles}` : "") +
        (report.notes.length ? `\n${report.notes.join("\n")}` : "");
      onNotify("success", "Очистка завершена", `Освобождено ${fmt(report.freedMb)}`);
      await scanJunk();
    } catch (caught) {
      onNotify("error", "Очистка не удалась", humanizeError(caught));
    } finally {
      cleaning = false;
    }
  }

  async function emptyBin() {
    try {
      onNotify("success", await api.emptyRecycleBin());
    } catch (caught) {
      onNotify("error", "Не удалось очистить корзину", humanizeError(caught));
    }
  }

  // ---------------------------------------------------------------- context menu
  const SCOPES = [
    { id: "files", label: "Файлы" },
    { id: "folders", label: "Папки" },
    { id: "desktop", label: "Рабочий стол / фон" },
  ] as const;

  let menu = $state<MenuItem[]>([]);
  let menuScope = $state<"files" | "folders" | "desktop">("files");
  let loadingMenu = $state(false);
  let formKey = $state("");
  let formLabel = $state("");
  let formCommand = $state("");
  let formIcon = $state("");
  let formExtended = $state(false);

  const shownMenu = $derived(menu.filter((m) => m.scope === menuScope));

  async function loadMenu() {
    loadingMenu = true;
    try {
      menu = await api.listMenu();
    } catch (caught) {
      onNotify("error", "Не удалось получить меню", humanizeError(caught));
    } finally {
      loadingMenu = false;
    }
  }

  async function addMenu() {
    try {
      await api.addMenuItem(menuScope, formKey.trim(), formLabel.trim(), formCommand.trim(), formIcon.trim(), formExtended);
      formKey = formLabel = formCommand = formIcon = "";
      formExtended = false;
      await loadMenu();
    } catch (caught) {
      onNotify("error", "Не удалось добавить пункт", humanizeError(caught));
    }
  }

  async function toggleMenu(m: MenuItem) {
    try {
      await api.setMenuDisabled(m.scope, m.key, m.origin, !m.disabled);
      await loadMenu();
    } catch (caught) {
      onNotify("error", `Не удалось изменить: ${m.label}`, humanizeError(caught));
    }
  }

  async function removeMenu(m: MenuItem) {
    try {
      await api.removeMenuItem(m.scope, m.key);
      await loadMenu();
    } catch (caught) {
      onNotify("error", "Не удалось удалить", humanizeError(caught));
    }
  }

  function open(next: Tab) {
    tab = next;
    query = "";
    if (next === "services" && services.length === 0) void loadServices();
    if (next === "startup" && startup.length === 0) void loadStartup();
    if (next === "cleaner" && junk.length === 0) void scanJunk();
    if (next === "menu" && menu.length === 0) void loadMenu();
  }

  onMount(() => open("services"));
</script>

<div class="sys-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>System maintenance</p>
      <h1>Обслуживание <span>системы</span></h1>
      <p class="lead">Службы, автозагрузка, очистка мусора и контекстное меню. Изменения служб и HKLM требуют прав администратора.</p>
    </div>
    <img class="hero-art" src="/art/hero-system.svg" alt="" />
  </header>

  <div class="tabs">
    <button type="button" class:active={tab === "services"} onclick={() => open("services")}>Службы</button>
    <button type="button" class:active={tab === "startup"} onclick={() => open("startup")}>Автозагрузка</button>
    <button type="button" class:active={tab === "cleaner"} onclick={() => open("cleaner")}>Очистка</button>
    <button type="button" class:active={tab === "menu"} onclick={() => open("menu")}>Контекстное меню</button>
  </div>

  {#if tab === "services"}
    <div class="toolbar">
      <input class="search" placeholder="Поиск службы…" bind:value={query} />
      <label class="check"><input type="checkbox" bind:checked={onlyThird} />Только сторонние</label>
      <button type="button" class="btn" disabled={loadingServices} onclick={loadServices}><Icon name="refresh" size={15} />{loadingServices ? "Загрузка…" : "Обновить"}</button>
    </div>
    <div class="list">
      {#each shownServices as s (s.name)}
        <div class="row-item">
          <span class="meta"><strong>{s.display}</strong><small>{s.name} · {s.state}</small></span>
          {#if s.protected}
            <span class="tag">системная</span>
          {:else}
            <select value={modeOf(s)} disabled={busyKey === s.name} onchange={(e) => changeService(s, e.currentTarget.value)}>
              <option value="auto">Авто</option>
              <option value="delayed">Отложенный</option>
              <option value="manual">Вручную</option>
              <option value="disabled">Отключена</option>
            </select>
          {/if}
        </div>
      {:else}
        <p class="empty">{loadingServices ? "Загружаем…" : "Ничего не найдено."}</p>
      {/each}
    </div>
  {:else if tab === "startup"}
    <div class="toolbar">
      <span class="muted">Записи реестра Run и папки «Автозагрузка». Отключение не удаляет запись.</span>
      <div class="spacer"></div>
      <button type="button" class="btn" disabled={loadingStartup} onclick={loadStartup}><Icon name="refresh" size={15} />{loadingStartup ? "Загрузка…" : "Обновить"}</button>
    </div>
    <div class="list">
      {#each startup as item (item.id)}
        <div class="row-item">
          <span class="meta"><strong>{item.name}</strong><small>{item.command}</small></span>
          <button type="button" class="switch" class:on={item.enabled} disabled={busyKey === item.id} aria-label={item.name} onclick={() => toggleStartup(item)}><i></i></button>
        </div>
      {:else}
        <p class="empty">{loadingStartup ? "Загружаем…" : "Автозагрузка пуста."}</p>
      {/each}
    </div>
  {:else if tab === "cleaner"}
    <div class="card total">
      <strong class="big">{fmt(junkTotal)}</strong>
      <span class="muted">{scanningJunk ? "Сканирование…" : "будет освобождено"}</span>
    </div>
    <div class="list">
      {#each junk as j (j.id)}
        <label class="row-item" class:active={junkPick[j.id]}>
          <input type="checkbox" bind:checked={junkPick[j.id]} disabled={cleaning} />
          <span class="meta"><strong>{j.title}</strong><small>{j.hint}{j.requiresAdmin ? " · нужен админ" : ""}</small></span>
          <span class="ver">{fmt(j.sizeMb)} · {j.files}</span>
        </label>
      {/each}
    </div>
    <div class="actions">
      <button type="button" class="btn" disabled={scanningJunk || cleaning} onclick={scanJunk}><Icon name="refresh" size={15} />Пересканировать</button>
      <button type="button" class="btn" disabled={cleaning} onclick={emptyBin}>Очистить корзину</button>
      <button type="button" class="btn primary" disabled={cleaning || scanningJunk || junkTotal === 0} onclick={cleanJunk}>{cleaning ? "Очищаем…" : "Очистить выбранное"}</button>
    </div>
    {#if cleanResult}<p class="ok">{cleanResult}</p>{/if}
  {:else}
    <div class="tabs sub">
      {#each SCOPES as sc}
        <button type="button" class:active={menuScope === sc.id} onclick={() => (menuScope = sc.id)}>{sc.label}</button>
      {/each}
    </div>
    <div class="list">
      {#each shownMenu as m (m.origin + m.key)}
        <div class="row-item">
          <span class="meta"><strong>{m.label}</strong><small>{m.origin === "user" ? "пользователь" : "система"} · {m.command || "обработчик без команды"}</small></span>
          <button type="button" class="switch" class:on={!m.disabled} aria-label={m.label} onclick={() => toggleMenu(m)}><i></i></button>
          {#if m.ours}<button type="button" class="btn" onclick={() => removeMenu(m)}><Icon name="x" size={14} /></button>{/if}
        </div>
      {:else}
        <p class="empty">{loadingMenu ? "Загружаем…" : "Пунктов нет."}</p>
      {/each}
    </div>
    <div class="card">
      <strong>Добавить пункт ({SCOPES.find((s) => s.id === menuScope)?.label})</strong>
      <input class="search" placeholder="Название пункта" bind:value={formLabel} />
      <input class="search" placeholder="Идентификатор (латиница, например OpenTerminal)" bind:value={formKey} />
      <input class="search" placeholder={'Команда, например: wt.exe -d "%V"  (для файлов: notepad.exe "%1")'} bind:value={formCommand} />
      <input class="search" placeholder="Путь к иконке (необязательно)" bind:value={formIcon} />
      <label class="check"><input type="checkbox" bind:checked={formExtended} />Только при нажатой Shift</label>
      <button type="button" class="btn primary" disabled={!formKey.trim() || !formLabel.trim() || !formCommand.trim()} onclick={addMenu}>Добавить</button>
      <small class="muted">Пункт записывается в профиль пользователя (HKCU), права администратора не нужны. В Windows 11 он появится в разделе «Показать дополнительные параметры».</small>
    </div>
  {/if}
</div>

<style>
  .sys-page { padding-bottom: 40px; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 640px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .tabs { display: flex; gap: 8px; margin: 22px 4px 14px; flex-wrap: wrap; }
  .tabs.sub { margin-top: 4px; }
  .tabs button { padding: 9px 16px; font-size: 13px; color: var(--muted-strong); border: 1px solid var(--border); border-radius: 99px; }
  .tabs button.active { background: var(--text); color: var(--bg); border-color: var(--text); }
  .toolbar { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; margin: 0 4px 14px; }
  .spacer { flex: 1; }
  .search { width: 100%; padding: 10px 14px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); outline: none; user-select: text; }
  .toolbar .search { flex: 1; min-width: 200px; width: auto; }
  .check { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted-strong); }
  .list { display: grid; gap: 8px; }
  .row-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--surface); }
  .row-item.active { border-color: var(--border-strong); background: var(--surface-strong); }
  .meta { flex: 1; min-width: 0; display: grid; gap: 2px; }
  .meta strong, .meta small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small, .muted { color: var(--muted); font-size: 12px; }
  .ver { font-size: 12px; color: var(--muted-strong); white-space: nowrap; }
  .tag { font-size: 11px; color: var(--muted); border: 1px solid var(--border-strong); border-radius: 99px; padding: 2px 10px; }
  select { padding: 7px 10px; background: var(--bg-soft); border: 1px solid var(--border-strong); border-radius: 10px; color: var(--text); }
  .switch { width: 40px; height: 22px; flex: none; border-radius: 99px; border: 1px solid var(--border-strong); background: var(--surface-strong); position: relative; transition: background 200ms; }
  .switch i { position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: var(--muted-strong); transition: transform 200ms var(--ease-out), background 200ms; }
  .switch.on { background: var(--accent); }
  .switch.on i { transform: translateX(18px); background: var(--bg); }
  .empty { margin: 8px 4px; color: var(--muted); }
  .card { padding: 18px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 10px; margin: 14px 0; }
  .card.total { justify-items: center; text-align: center; }
  .big { font-size: 38px; letter-spacing: -0.03em; }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font-size: 13px; background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); justify-self: start; }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.primary { background: var(--text); color: var(--bg); font-weight: 600; }
  .actions { display: flex; gap: 10px; margin: 18px 4px 0; flex-wrap: wrap; }
  .ok { margin: 14px 4px 0; color: var(--success); white-space: pre-wrap; }
</style>
