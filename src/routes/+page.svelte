<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    api,
    humanizeError,
    DISCLAIMER_KEY,
    type CatalogApp,
    type InstallProgress,
    type JobState,
    type ToastItem,
    type ToastTone,
  } from "$lib/api";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SoftwareHub from "$lib/components/SoftwareHub.svelte";
  import TweaksPanel from "$lib/components/TweaksPanel.svelte";
  import BackupPanel from "$lib/components/BackupPanel.svelte";
  import WelcomeGate from "$lib/components/WelcomeGate.svelte";
  import AppDetails from "$lib/components/AppDetails.svelte";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import StrippedPanel from "$lib/components/StrippedPanel.svelte";
  import UsbPanel from "$lib/components/UsbPanel.svelte";
  import DiagnosticsPanel from "$lib/components/DiagnosticsPanel.svelte";
  import SoftwareManager from "$lib/components/SoftwareManager.svelte";
  import SystemPanel from "$lib/components/SystemPanel.svelte";
  import MonitorPanel from "$lib/components/MonitorPanel.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import AuthorsPanel from "$lib/components/AuthorsPanel.svelte";
  import StarField from "$lib/components/StarField.svelte";
  import Icon from "$lib/components/Icon.svelte";

  type TabId = "apps" | "manager" | "system" | "monitor" | "tweaks" | "stripped" | "usb" | "backup" | "diagnostics" | "settings" | "authors";

  type DownloadBatch = {
    total: number;
    completed: number;
    failed: number;
    failures: string[];
    currentId: string | null;
    running: boolean;
    finished: boolean;
  };

  const navItems = [
    {
      id: "apps",
      label: "Software Hub",
      hint: "Каталог и установка",
      icon: "grid",
    },
    {
      id: "manager",
      label: "Управление ПО",
      hint: "Обновления, удаление, пресеты",
      icon: "package",
    },
    {
      id: "system",
      label: "Обслуживание",
      hint: "Службы, очистка, меню",
      icon: "terminal",
    },
    {
      id: "monitor",
      label: "Монитор и тесты",
      hint: "Железо и бенчмарк",
      icon: "gamepad",
    },
    {
      id: "tweaks",
      label: "Windows Tweaks",
      hint: "Оптимизация системы",
      icon: "sliders",
    },
    {
      id: "stripped",
      label: "Stripped-down",
      hint: "Бета · урезание Windows",
      icon: "rocket",
    },
    {
      id: "usb",
      label: "Windows на флешку",
      hint: "Установочный USB",
      icon: "usb",
    },
    {
      id: "backup",
      label: "Backups",
      hint: "Восстановление AppData",
      icon: "archive",
    },
    {
      id: "diagnostics",
      label: "Диагностика",
      hint: "Проверка и логи",
      icon: "shield",
    },
    {
      id: "settings",
      label: "Настройки",
      hint: "Тема и параметры",
      icon: "settings",
    },
    {
      id: "authors",
      label: "Авторы",
      hint: "Создатели проекта",
      icon: "user",
    },
  ];

  let booted = $state(false);
  let accepted = $state(false);
  let welcomeLeaving = $state(false);
  let activeTab = $state<TabId>("apps");

  let catalogApps = $state<CatalogApp[]>([]);
  let catalogVersion = $state(0);
  let catalogLoading = $state(true);
  let catalogError = $state("");

  let jobs = $state<Record<string, JobState>>({});
  let toasts = $state<ToastItem[]>([]);
  let selectedApp = $state<CatalogApp | null>(null);
  let selectedIds = $state<string[]>([]);
  let batch = $state<DownloadBatch | null>(null);
  let batchToken = 0;
  let admin = $state(false);

  let toastSequence = 0;

  let selectedCount = $derived(
    catalogApps.filter((app) => selectedIds.includes(app.id)).length,
  );
  let batchRunning = $derived(batch?.running ?? false);
  let dockVisible = $derived(selectedCount > 0 || batch !== null);

  let currentApp = $derived.by(() => {
    const id = batch?.currentId;
    return id ? (catalogApps.find((app) => app.id === id) ?? null) : null;
  });

  let currentJob = $derived.by(() => {
    const id = batch?.currentId;
    return id ? jobs[id] : undefined;
  });

  let dockPercent = $derived.by(() => {
    if (!batch) return 0;
    if (batch.finished) return 100;
    const partial =
      currentJob?.percent === null || currentJob?.percent === undefined
        ? 0
        : Math.max(0, Math.min(100, currentJob.percent)) / 100;
    return Math.min(100, ((batch.completed + partial) / batch.total) * 100);
  });

  let dockPercentLabel = $derived.by(() => {
    if (!batch) return "";
    if (batch.finished) return "100%";
    if (currentJob?.percent !== null && currentJob?.percent !== undefined) {
      return `${Math.round(dockPercent)}%`;
    }
    return `${batch.completed} / ${batch.total}`;
  });

  let dockTitle = $derived.by(() => {
    if (!batch) {
      return `Выбрано ${selectedCount} ${plural(selectedCount, "приложение", "приложения", "приложений")}`;
    }
    if (batch.finished) {
      return batch.failed === 0
        ? `Установлено ${batch.total} ${plural(batch.total, "приложение", "приложения", "приложений")}`
        : `Завершено: ${batch.completed - batch.failed} из ${batch.total}`;
    }
    const step = Math.min(batch.completed + 1, batch.total);
    return `Установка ${step} из ${batch.total}`;
  });

  let dockSubtitle = $derived.by(() => {
    if (!batch) {
      return "Проверьте список и нажмите «Скачать выбранное».";
    }
    if (batch.finished) {
      return batch.failed === 0
        ? "Очередь загрузки завершена. Все приложения установлены."
        : `Не удалось установить: ${batch.failures.join(", ")}. Подробности в уведомлениях.`;
    }
    const appName = currentApp?.name ?? "Подготовка очереди";
    const phase = currentJob?.phase ?? "Ожидание установщика";
    return `${appName} · ${phase}`;
  });

  function plural(count: number, one: string, few: string, many: string): string {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return one;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few;
    return many;
  }

  onMount(() => {
    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    accepted = localStorage.getItem(DISCLAIMER_KEY) === "1";
    booted = true;

    loadCatalog();
    api
      .adminStatus()
      .then((value) => {
        if (!disposed) admin = value;
      })
      .catch(() => {
        if (!disposed) admin = false;
      });

    listen<InstallProgress>("install-progress", (event) => {
      const progress = event.payload;
      jobs = {
        ...jobs,
        [progress.appId]: {
          status: "running",
          percent: progress.percent,
          phase: progress.phase,
          message: progress.message ?? undefined,
        },
      };
    }).then((stop) => {
      unlisten = stop;
      if (disposed) stop();
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  async function loadCatalog() {
    catalogLoading = true;
    catalogError = "";
    try {
      const catalog = await api.catalog();
      catalogApps = catalog.apps;
      catalogVersion = catalog.version;
      const known = new Set(catalog.apps.map((app) => app.id));
      selectedIds = selectedIds.filter((id) => known.has(id));
    } catch (caught) {
      catalogError = humanizeError(caught);
    } finally {
      catalogLoading = false;
    }
  }

  function notify(tone: ToastTone, title: string, message?: string) {
    const id = ++toastSequence;
    toasts = [...toasts, { id, tone, title, message }];
    window.setTimeout(() => dismissToast(id), 5600);
  }

  function dismissToast(id: number) {
    toasts = toasts.filter((toast) => toast.id !== id);
  }

  function toggleSelect(app: CatalogApp) {
    if (app.placeholder || batchRunning) return;
    if (batch && !batch.running) batch = null;
    selectedIds = selectedIds.includes(app.id)
      ? selectedIds.filter((id) => id !== app.id)
      : [...selectedIds, app.id];
  }

  function clearSelection() {
    selectedIds = [];
  }

  async function downloadSelected() {
    if (batchRunning) return;

    const targets = catalogApps.filter(
      (app) =>
        selectedIds.includes(app.id) &&
        !app.placeholder &&
        jobs[app.id]?.status !== "running",
    );
    if (targets.length === 0) return;

    const token = ++batchToken;
    selectedIds = [];
    batch = {
      total: targets.length,
      completed: 0,
      failed: 0,
      failures: [],
      currentId: targets[0]?.id ?? null,
      running: true,
      finished: false,
    };

    for (const app of targets) {
      if (token !== batchToken || !batch) return;
      batch.currentId = app.id;
      const ok = await runInstall(app);
      if (token !== batchToken || !batch) return;
      batch.completed += 1;
      if (!ok) {
        batch.failed += 1;
        batch.failures = [...batch.failures, app.name];
      }
    }

    batch.running = false;
    batch.finished = true;
    batch.currentId = null;

    if (batch.failed === 0) {
      notify(
        "success",
        `Готово: ${batch.total} ${plural(batch.total, "приложение", "приложения", "приложений")}`,
        "Все приложения из очереди установлены.",
      );
      window.setTimeout(() => {
        if (token === batchToken && batch?.finished) batch = null;
      }, 8000);
    } else {
      notify(
        "error",
        `Завершено с ошибками: ${batch.failed} из ${batch.total}`,
        `Не удалось установить: ${batch.failures.join(", ")}.`,
      );
    }
  }

  async function runInstall(app: CatalogApp): Promise<boolean> {
    jobs = {
      ...jobs,
      [app.id]: {
        status: "running",
        percent: null,
        phase: "Запуск установщика",
      },
    };

    try {
      const report = await api.installEntry(app.id);
      jobs = {
        ...jobs,
        [app.id]: {
          status: "done",
          percent: 100,
          phase: "Готово",
        },
      };
      window.setTimeout(() => clearJob(app.id), 2600);
      return true;
    } catch (caught) {
      const message = humanizeError(caught);
      jobs = {
        ...jobs,
        [app.id]: {
          status: "error",
          percent: null,
          phase: "Ошибка установки",
          message,
        },
      };
      notify("error", `Не удалось установить ${app.name}`, message);
      window.setTimeout(() => clearJob(app.id), 7000);
      return false;
    }
  }

  function clearJob(id: string) {
    if (jobs[id]?.status === "running") return;
    const next = { ...jobs };
    delete next[id];
    jobs = next;
  }

  function closeDock() {
    if (batch?.running) return;
    batch = null;
    selectedIds = [];
  }

  function switchTab(id: string) {
    activeTab = id as TabId;
    selectedApp = null;
    if (id !== "apps") selectedIds = [];
  }

  function acceptDisclaimer() {
    localStorage.setItem(DISCLAIMER_KEY, "1");
    welcomeLeaving = true;
    window.setTimeout(() => {
      accepted = true;
      welcomeLeaving = false;
      notify(
        "success",
        "Добро пожаловать в Setup Hub",
        "Выберите приложения в каталоге и скачайте их одной очередью.",
      );
    }, 240);
  }

  function exitApp() {
    api.exit().catch(() => {
      window.close();
    });
  }

  function elevate() {
    if (admin) return;
    api.restartElevated().catch((caught) => {
      notify("error", "Не удалось запросить UAC", humanizeError(caught));
    });
  }
</script>

<div class="app-root">
  <div class="ambient ambient-a"></div>
  <div class="ambient ambient-b"></div>
  <div class="ambient ambient-c"></div>
  <div class="grid-overlay"></div>
  <StarField />

  <div
    class="app-frame"
    class:locked={!accepted}
    inert={!accepted}
    aria-hidden={!accepted}
  >
    <Sidebar
      active={activeTab}
      items={navItems}
      {admin}
      onNavigate={switchTab}
      onElevate={elevate}
    />

    <main class="workspace">
      <div class="workspace-scroll" class:with-dock={dockVisible}>
        {#if activeTab === "apps"}
          <SoftwareHub
            apps={catalogApps}
            loading={catalogLoading}
            error={catalogError}
            {catalogVersion}
            {jobs}
            {selectedIds}
            selectionDisabled={batchRunning}
            onToggleSelect={toggleSelect}
            onDetails={(app) => (selectedApp = app)}
            onRetry={loadCatalog}
          />
        {:else if activeTab === "manager"}
          <SoftwareManager onNotify={notify} />
        {:else if activeTab === "system"}
          <SystemPanel onNotify={notify} />
        {:else if activeTab === "monitor"}
          <MonitorPanel onNotify={notify} />
        {:else if activeTab === "tweaks"}
          <TweaksPanel onNotify={notify} />
        {:else if activeTab === "stripped"}
          <StrippedPanel onNotify={notify} onLeave={() => switchTab("tweaks")} />
        {:else if activeTab === "usb"}
          <UsbPanel onNotify={notify} />
        {:else if activeTab === "diagnostics"}
          <DiagnosticsPanel onNotify={notify} />
        {:else if activeTab === "settings"}
          <SettingsPanel onNotify={notify} />
        {:else if activeTab === "authors"}
          <AuthorsPanel />
        {:else}
          <BackupPanel onNotify={notify} />
        {/if}
      </div>

      {#if dockVisible}
        <div
          class="download-dock"
          class:running={batch?.running}
          class:done={batch?.finished}
          class:failed={(batch?.failed ?? 0) > 0}
        >
          <div class="dock-top">
            <span class="dock-icon">
              {#if batch?.running}
                <span class="dock-spinner"></span>
              {:else if batch?.finished}
                <Icon name={(batch.failed ?? 0) > 0 ? "alert" : "check"} size={19} />
              {:else}
                <Icon name="download" size={19} />
              {/if}
            </span>

            <div class="dock-copy">
              <div class="dock-line">
                <strong>{dockTitle}</strong>
                {#if batch}
                  <span class="dock-percent">{dockPercentLabel}</span>
                {/if}
              </div>
              <p>{dockSubtitle}</p>
            </div>

            <div class="dock-actions">
              {#if batch?.running}
                <span class="dock-busy">
                  <span class="dock-busy-dot"></span>
                  Выполняется
                </span>
              {:else if batch?.finished}
                <button type="button" class="dock-button ghost" onclick={closeDock}>
                  Закрыть
                </button>
              {:else}
                <button type="button" class="dock-button ghost" onclick={clearSelection}>
                  Очистить
                </button>
                <button type="button" class="dock-button primary" onclick={downloadSelected}>
                  <Icon name="download" size={15} />
                  Скачать выбранное ({selectedCount})
                </button>
              {/if}
            </div>
          </div>

          <div
            class="dock-progress"
            class:indeterminate={batch?.running &&
              (currentJob?.percent === null || currentJob?.percent === undefined)}
          >
            <span class="dock-progress-fill" style={`width:${batch ? dockPercent : 0}%`}></span>
            <span class="dock-progress-sheen"></span>
          </div>
        </div>
      {/if}
    </main>
  </div>
</div>

{#if booted && !accepted}
  <WelcomeGate
    leaving={welcomeLeaving}
    onAccept={acceptDisclaimer}
    onExit={exitApp}
  />
{/if}

{#if selectedApp}
  <AppDetails app={selectedApp} onClose={() => (selectedApp = null)} />
{/if}

<ToastStack
  {toasts}
  offset={dockVisible ? 126 : 24}
  onDismiss={dismissToast}
/>

{#if !booted}
  <div class="boot-screen">
    <div class="boot-logo"><Icon name="logo" size={34} stroke={1.6} /></div>
    <span class="boot-name">SETUP HUB</span>
    <span class="boot-line"><i></i></span>
    <span class="boot-note">decrypting local catalog</span>
  </div>
{/if}

<style>
  .app-root {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background:
      radial-gradient(circle at 50% -20%, rgb(var(--fg) / 0.06), transparent 45%),
      linear-gradient(180deg, rgb(var(--panel) / 0.94), rgb(var(--panel) / 0.98));
  }

  .grid-overlay {
    position: fixed;
    inset: 0;
    opacity: 0.13;
    background-image:
      linear-gradient(rgb(var(--fg) / 0.035) 1px, transparent 1px),
      linear-gradient(90deg, rgb(var(--fg) / 0.035) 1px, transparent 1px);
    background-size: 42px 42px;
    mask-image: radial-gradient(circle at 50% 0%, #000 0%, transparent 72%);
    pointer-events: none;
  }

  .ambient {
    position: fixed;
    border-radius: 50%;
    filter: blur(110px);
    pointer-events: none;
  }

  .ambient-a {
    top: -16%;
    left: 22%;
    width: 42vw;
    height: 42vw;
    background: rgb(var(--fg) / 0.035);
  }

  .ambient-b {
    right: -14%;
    bottom: -20%;
    width: 38vw;
    height: 38vw;
    background: rgb(var(--fg) / 0.022);
  }

  .ambient-c {
    bottom: 12%;
    left: 8%;
    width: 24vw;
    height: 24vw;
    background: rgb(var(--fg) / 0.016);
  }

  .app-frame {
    position: relative;
    z-index: 1;
    display: grid;
    height: 100vh;
    grid-template-columns: 270px minmax(0, 1fr);
    gap: 16px;
    padding: 16px;
    transition:
      filter 260ms ease,
      opacity 260ms ease;
  }

  .app-frame.locked {
    filter: brightness(0.68) saturate(0.7);
    pointer-events: none;
    user-select: none;
  }

  .workspace {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border-radius: var(--radius-xl);
  }

  .workspace::before {
    position: absolute;
    top: 0;
    right: 10%;
    left: 10%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgb(var(--fg) / 0.14), transparent);
    content: "";
    pointer-events: none;
  }

  .workspace-scroll {
    height: 100%;
    overflow-x: hidden;
    overflow-y: auto;
    padding: 10px 10px 40px 6px;
    scroll-behavior: smooth;
    transition: padding-bottom 260ms var(--ease-out);
  }

  .workspace-scroll.with-dock {
    padding-bottom: 152px;
  }

  .download-dock {
    position: absolute;
    right: 14px;
    bottom: 14px;
    left: 14px;
    z-index: 60;
    padding: 15px 18px 0;
    overflow: hidden;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 20px;
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.055), transparent 42%),
      linear-gradient(180deg, rgb(var(--panel) / 0.94), rgb(var(--panel) / 0.97));
    box-shadow:
      0 28px 64px rgba(0, 0, 0, 0.56),
      0 0 0 1px rgb(var(--fg) / 0.025),
      inset 0 1px 0 rgb(var(--fg) / 0.08);
    backdrop-filter: blur(28px) saturate(115%);
    -webkit-backdrop-filter: blur(28px) saturate(115%);
    animation: dock-in 340ms var(--ease-out) both;
  }

  .download-dock.done {
    border-color: rgb(var(--fg) / 0.22);
  }

  .download-dock.failed {
    border-color: rgba(255, 107, 107, 0.24);
  }

  .dock-top {
    display: flex;
    align-items: center;
    gap: 13px;
  }

  .dock-icon {
    display: grid;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.16);
    border-radius: 13px;
    color: var(--text);
    background:
      radial-gradient(circle at 32% 18%, rgb(var(--fg) / 0.14), transparent 46%),
      rgb(var(--fg) / 0.055);
    box-shadow: inset 0 1px 0 rgb(var(--fg) / 0.08);
  }

  .download-dock.failed .dock-icon {
    border-color: rgba(255, 107, 107, 0.28);
    color: var(--danger);
    background: rgba(255, 107, 107, 0.06);
  }

  .dock-spinner {
    width: 17px;
    height: 17px;
    border: 1.7px solid rgb(var(--fg) / 0.16);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .dock-copy {
    min-width: 0;
    flex: 1;
  }

  .dock-line {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .dock-line strong {
    overflow: hidden;
    font-size: 13px;
    font-weight: 650;
    letter-spacing: -0.015em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dock-percent {
    flex: 0 0 auto;
    color: rgb(var(--fg) / 0.58);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
  }

  .dock-copy p {
    overflow: hidden;
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 10px;
    line-height: 1.4;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dock-actions {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    gap: 8px;
  }

  .dock-button {
    display: inline-flex;
    height: 38px;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 0 14px;
    border-radius: 11px;
    font-size: 10.5px;
    font-weight: 650;
    white-space: nowrap;
    transition:
      transform 200ms var(--ease-out),
      border-color 200ms ease,
      background 200ms ease,
      box-shadow 200ms ease;
  }

  .dock-button:active {
    transform: scale(0.97);
  }

  .dock-button.ghost {
    border: 1px solid rgb(var(--fg) / 0.11);
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.035);
  }

  .dock-button.ghost:hover {
    border-color: rgb(var(--fg) / 0.28);
    color: var(--text);
    background: rgb(var(--fg) / 0.07);
  }

  .dock-button.primary {
    border: 1px solid rgb(var(--fg) / 0.88);
    color: #09090b;
    background: linear-gradient(145deg, #ffffff, #d9d9de);
    box-shadow:
      0 0 24px rgb(var(--fg) / 0.14),
      inset 0 1px 0 rgb(var(--fg) / 0.65);
  }

  .dock-button.primary:hover {
    background: linear-gradient(145deg, #ffffff, #e8e8eb);
    box-shadow:
      0 0 32px rgb(var(--fg) / 0.24),
      inset 0 1px 0 rgb(var(--fg) / 0.8);
    transform: translateY(-1px);
  }

  .dock-busy {
    display: inline-flex;
    height: 34px;
    align-items: center;
    gap: 7px;
    padding: 0 11px;
    border: 1px solid rgb(var(--fg) / 0.08);
    border-radius: 10px;
    color: var(--muted);
    background: rgb(var(--fg) / 0.025);
    font-size: 9.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .dock-busy-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 8px rgb(var(--fg) / 0.6);
    animation: pulse-dot 1.3s ease-in-out infinite;
  }

  .dock-progress {
    position: relative;
    height: 3px;
    margin: 15px -18px 0;
    overflow: hidden;
    background: rgb(var(--fg) / 0.07);
  }

  .dock-progress-fill {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: 0 999px 999px 0;
    background: linear-gradient(90deg, #73737c, #dfdfe3 45%, #ffffff 72%, #a6a6ae);
    background-size: 220% 100%;
    box-shadow: 0 0 14px rgb(var(--fg) / 0.34);
    transition: width 260ms linear;
  }

  .dock-progress.indeterminate .dock-progress-fill {
    position: absolute;
    width: 34% !important;
    animation:
      dock-indeterminate 1.5s cubic-bezier(0.45, 0, 0.55, 1) infinite,
      progress-shift 1.4s linear infinite;
  }

  .dock-progress-sheen {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      100deg,
      transparent 30%,
      rgb(var(--fg) / 0.28) 50%,
      transparent 70%
    );
    transform: translateX(-100%);
    animation: shimmer 2.2s ease-in-out infinite;
    pointer-events: none;
  }

  @keyframes dock-in {
    from {
      opacity: 0;
      transform: translateY(18px) scale(0.985);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes dock-indeterminate {
    0% {
      transform: translateX(-110%);
    }
    100% {
      transform: translateX(310%);
    }
  }

  .boot-screen {
    position: fixed;
    inset: 0;
    z-index: 1200;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 11px;
    background:
      radial-gradient(circle at 50% 45%, rgb(var(--fg) / 0.04), transparent 28%),
      #060608;
    animation: fade-in 180ms ease both;
  }

  .boot-logo {
    display: grid;
    width: 64px;
    height: 64px;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.28);
    border-radius: 20px;
    color: var(--accent);
    background: rgb(var(--fg) / 0.045);
    box-shadow: 0 0 38px rgb(var(--fg) / 0.08);
    animation: breathe 2.4s ease-in-out infinite;
  }

  .boot-name {
    margin-top: 5px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.32em;
  }

  .boot-line {
    position: relative;
    width: 120px;
    height: 2px;
    margin-top: 4px;
    overflow: hidden;
    border-radius: 999px;
    background: rgb(var(--fg) / 0.07);
  }

  .boot-line i {
    position: absolute;
    inset: 0 auto 0 0;
    width: 42%;
    border-radius: inherit;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    animation: progress-indeterminate 1.4s ease-in-out infinite;
  }

  .boot-note {
    color: rgb(var(--fg) / 0.26);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  @media (max-width: 1080px) {
    .dock-top {
      flex-wrap: wrap;
    }

    .dock-copy {
      min-width: 160px;
    }

    .dock-actions {
      width: 100%;
      justify-content: flex-end;
      padding-bottom: 2px;
    }

    .workspace-scroll.with-dock {
      padding-bottom: 190px;
    }
  }

  @media (max-width: 920px) {
    .app-frame {
      grid-template-columns: 232px minmax(0, 1fr);
    }
  }
</style>
