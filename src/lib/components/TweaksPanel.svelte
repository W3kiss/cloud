<script lang="ts">
  import { onMount } from "svelte";
  import { api, humanizeError, type ToastTone, type TweakStatus } from "$lib/api";
  import Icon from "./Icon.svelte";
  import ToggleSwitch from "./ToggleSwitch.svelte";

  type TweakDefinition = {
    id: string;
    title: string;
    description: string;
    icon: string;
    accent: string;
    note: string;
    requiresAdmin: boolean;
    warning?: string;
  };

  const tweaks: TweakDefinition[] = [
    {
      id: "disable_telemetry",
      title: "Конфиденциальность системы",
      description:
        "Отключает политику телеметрии и останавливает фоновые службы DiagTrack и dmwappushservice.",
      icon: "shield",
      accent: "#f4f4f5",
      note: "HKLM policy + services",
      requiresAdmin: true,
    },
    {
      id: "classic_context_menu",
      title: "Интерфейс проводника",
      description:
        "Возвращает классическое контекстное меню Windows 10 в Проводнике Windows 11.",
      icon: "window",
      accent: "#e4e4e7",
      note: "HKCU CLSID override",
      requiresAdmin: false,
    },
    {
      id: "disable_bing_search",
      title: "Чистый поиск в «Пуске»",
      description:
        "Убирает веб-результаты и подсказки Bing из поиска Windows, оставляя только локальные файлы и приложения.",
      icon: "search",
      accent: "#d4d4d8",
      note: "HKCU Search policy",
      requiresAdmin: false,
    },
    {
      id: "disable_game_dvr",
      title: "Game DVR без фоновой записи",
      description:
        "Отключает фоновую запись Game DVR и Game Bar, чтобы игры не теряли кадры на лишний захват.",
      icon: "gamepad",
      accent: "#c9c9d0",
      note: "HKCU GameDVR keys",
      requiresAdmin: false,
    },
    {
      id: "enable_long_paths",
      title: "Длинные пути Windows",
      description:
        "Разрешает пути длиннее 260 символов — полезно для сборок, репозиториев и инструментов разработки.",
      icon: "folder",
      accent: "#b8b8c0",
      note: "HKLM FileSystem",
      requiresAdmin: true,
    },
    {
      id: "disable_hibernation",
      title: "Гибернация и быстрый запуск",
      description:
        "Отключает гибернацию и быстрый запуск, освобождая несколько гигабайт на системном диске.",
      icon: "power",
      accent: "#a8a8b0",
      note: "powercfg /h off",
      requiresAdmin: true,
      warning: "Быстрый запуск и гибернация перестанут работать до повторного включения.",
    },
    {
      id: "show_file_extensions",
      title: "Расширения файлов",
      description:
        "Показывает расширения известных типов файлов (.exe, .txt, .png) — проще заметить подделку вроде photo.jpg.exe.",
      icon: "copy",
      accent: "#a0a0a8",
      note: "HKCU Explorer\\Advanced",
      requiresAdmin: false,
    },
    {
      id: "show_hidden_files",
      title: "Скрытые файлы и папки",
      description:
        "Показывает скрытые файлы и папки в Проводнике, включая AppData и служебные каталоги проектов.",
      icon: "folder",
      accent: "#9a9aa2",
      note: "HKCU Explorer\\Advanced",
      requiresAdmin: false,
    },
    {
      id: "hide_taskbar_search",
      title: "Поиск на панели задач",
      description:
        "Скрывает поле поиска с панели задач, освобождая место для значков приложений.",
      icon: "search",
      accent: "#949498",
      note: "HKCU Search",
      requiresAdmin: false,
    },
    {
      id: "dark_mode",
      title: "Тёмная тема Windows",
      description:
        "Включает тёмное оформление для системы и приложений. Выключение возвращает светлую тему.",
      icon: "spark",
      accent: "#8e8e94",
      note: "HKCU Themes\\Personalize",
      requiresAdmin: false,
    },
    {
      id: "disable_advertising_id",
      title: "Рекламный идентификатор",
      description:
        "Запрещает приложениям использовать рекламный идентификатор для персональной рекламы.",
      icon: "user",
      accent: "#88888e",
      note: "HKCU AdvertisingInfo",
      requiresAdmin: false,
    },
    {
      id: "disable_suggestions",
      title: "Без рекомендаций Windows",
      description:
        "Отключает советы, рекомендации в «Пуске» и автоматическую установку рекламных приложений.",
      icon: "message",
      accent: "#82828a",
      note: "HKCU ContentDeliveryManager",
      requiresAdmin: false,
    },
    {
      id: "disable_copilot",
      title: "Отключить Copilot",
      description:
        "Отключает Windows Copilot через пользовательскую политику. Выключение снимает политику.",
      icon: "cpu",
      accent: "#7c7c84",
      note: "HKCU WindowsCopilot policy",
      requiresAdmin: false,
    },
    {
      id: "disable_activity_history",
      title: "Журнал действий",
      description:
        "Отключает журнал действий Windows и отправку истории действий в Microsoft.",
      icon: "refresh",
      accent: "#76767e",
      note: "HKLM System policy",
      requiresAdmin: true,
    },
    {
      id: "cleanup_startup",
      title: "Оптимизация автозапуска",
      description:
        "Убирает из автозагрузки OneDrive, Teams, Edge autolaunch и Spotify. Действие необратимо.",
      icon: "rocket",
      accent: "#9898a0",
      note: "OneDrive + bloatware",
      requiresAdmin: false,
      warning: "Удалённые записи не восстанавливаются автоматически.",
    },
    {
      id: "custom_hook",
      title: "Дополнительные утилиты",
      description:
        "Безопасный хук для собственного PowerShell-скрипта. Скрипт должен начинаться с маркера одобрения.",
      icon: "terminal",
      accent: "#8a8a92",
      note: "SETUP-HUB-CUSTOM-HOOK",
      requiresAdmin: false,
    },
  ];

  const EXPLORER_RESTART_IDS = [
    "classic_context_menu",
    "disable_bing_search",
    "show_file_extensions",
    "show_hidden_files",
    "hide_taskbar_search",
    "disable_copilot",
  ];

  function plural(count: number, one: string, few: string, many: string): string {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return one;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few;
    return many;
  }

  let moduleLabel = $derived(plural(tweaks.length, "модуль", "модуля", "модулей"));

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  let statuses = $state<TweakStatus[]>([]);
  let admin = $state(false);
  let hookPath = $state("");
  let loading = $state(true);
  let loadError = $state("");
  let busy = $state<Record<string, boolean>>({});
  let explorerBusy = $state(false);
  let elevating = $state(false);

  onMount(() => {
    load();
  });

  async function load() {
    loading = true;
    loadError = "";
    try {
      const [nextStatuses, nextAdmin, nextHookPath] = await Promise.all([
        api.tweakStates(),
        api.adminStatus(),
        api.customHookPath(),
      ]);
      statuses = nextStatuses;
      admin = nextAdmin;
      hookPath = nextHookPath;
    } catch (caught) {
      loadError = humanizeError(caught);
      onNotify("error", "Не удалось загрузить состояние твиков", loadError);
    } finally {
      loading = false;
    }
  }

  function statusOf(id: string): TweakStatus {
    return (
      statuses.find((item) => item.id === id) ?? {
        id,
        enabled: false,
        requiresAdmin: false,
      }
    );
  }

  async function toggleTweak(id: string, next: boolean) {
    busy = { ...busy, [id]: true };
    try {
      const message = await api.applyTweak(id, next);
      statuses = statuses.map((item) =>
        item.id === id ? { ...item, enabled: next } : item,
      );
      onNotify(
        "success",
        next ? "Модуль активирован" : "Модуль отключён",
        message,
      );
    } catch (caught) {
      onNotify("error", "Не удалось применить изменение", humanizeError(caught));
    } finally {
      busy = { ...busy, [id]: false };
    }
  }

  async function restartExplorer() {
    explorerBusy = true;
    try {
      const message = await api.restartExplorer();
      onNotify("success", "Проводник перезапущен", message);
    } catch (caught) {
      onNotify("error", "Не удалось перезапустить Проводник", humanizeError(caught));
    } finally {
      explorerBusy = false;
    }
  }

  async function elevate() {
    elevating = true;
    try {
      await api.restartElevated();
    } catch (caught) {
      elevating = false;
      onNotify("error", "Не удалось запросить UAC", humanizeError(caught));
    }
  }
</script>

<div class="tweaks-page">
  <header class="page-head">
    <div>
      <p class="eyebrow">
        <span class="live-dot"></span>
        System / Windows Tweaks
      </p>
      <h1>Тонкая настройка <span>Windows</span></h1>
      <p class="lead">
        Скрипты выполняются асинхронно через PowerShell в скрытом режиме.
        Для изменений HKLM и служб нужны права администратора.
      </p>
    </div>
    <div class="head-stats">
      <div class="head-stat">
        <Icon name="sliders" size={15} />
        <span><strong>{tweaks.length}</strong> {moduleLabel}</span>
      </div>
      <div class="head-stat">
        <Icon name="shield" size={15} />
        <span><strong>{admin ? "UAC ON" : "UAC OFF"}</strong></span>
      </div>
    </div>
  </header>

  {#if !admin}
    <div class="admin-banner">
      <span class="banner-icon"><Icon name="lock" size={18} /></span>
      <div class="banner-copy">
        <strong>Ограниченный режим</strong>
        <p>
          Конфиденциальность системы изменяет HKLM и службы Windows. Запустите
          приложение с повышенными правами, чтобы активировать все модули.
        </p>
      </div>
      <button type="button" class="banner-action" disabled={elevating} onclick={elevate}>
        {#if elevating}
          <span class="mini-spinner"></span>
        {:else}
          <Icon name="power" size={14} />
        {/if}
        Перезапустить с UAC
      </button>
    </div>
  {:else}
    <div class="admin-ok">
      <span class="ok-dot"></span>
      Приложение запущено с правами администратора — все модули доступны.
    </div>
  {/if}

  {#if loading}
    <div class="skeleton-grid">
      {#each Array(tweaks.length) as _, index (index)}
        <div class="skeleton-card" style={`--i:${index}`}>
          <span class="skeleton-line w-40"></span>
          <span class="skeleton-line w-80"></span>
          <span class="skeleton-line w-60"></span>
          <span class="skeleton-line w-30"></span>
        </div>
      {/each}
    </div>
  {:else if loadError}
    <div class="error-panel">
      <span class="error-icon"><Icon name="alert" size={20} /></span>
      <div>
        <strong>Состояние твиков недоступно</strong>
        <p>{loadError}</p>
      </div>
      <button type="button" class="retry" onclick={load}>
        <Icon name="refresh" size={14} />
        Повторить
      </button>
    </div>
  {:else}
    <div class="tweak-grid">
      {#each tweaks as tweak, index (tweak.id)}
        {@const status = statusOf(tweak.id)}
        <article
          class="tweak-card"
          class:enabled={status.enabled}
          style={`--accent:${tweak.accent}; --i:${index};`}
        >
          <div class="tweak-aura"></div>
          <header class="tweak-head">
            <span class="tweak-icon"><Icon name={tweak.icon} size={20} /></span>
            <div class="tweak-title">
              <h2>{tweak.title}</h2>
              <span class="tweak-note">{tweak.note}</span>
            </div>
            <ToggleSwitch
              checked={status.enabled}
              disabled={busy[tweak.id] ?? false}
              loading={busy[tweak.id] ?? false}
              label={tweak.title}
              onChange={(next) => toggleTweak(tweak.id, next)}
            />
          </header>

          <p class="tweak-description">{tweak.description}</p>

          {#if tweak.warning}
            <div class="warning-row">
              <Icon name="alert" size={13} />
              {tweak.warning}
            </div>
          {/if}

          {#if tweak.id === "custom_hook"}
            <div class="hook-path">
              <span>Script path</span>
              <code>{hookPath || "%APPDATA%\\SetupHub\\scripts\\custom.ps1"}</code>
            </div>
          {/if}

          <footer class="tweak-foot">
            <span class="state-pill" class:on={status.enabled}>
              <span class="state-dot"></span>
              {status.enabled ? "Включено" : "Выключено"}
            </span>
            {#if tweak.requiresAdmin}
              <span class="admin-tag">Требуется UAC</span>
            {/if}
            {#if EXPLORER_RESTART_IDS.includes(tweak.id) && status.enabled}
              <button
                type="button"
                class="inline-action"
                disabled={explorerBusy}
                onclick={restartExplorer}
              >
                {#if explorerBusy}
                  <span class="mini-spinner"></span>
                {:else}
                  <Icon name="refresh" size={12} />
                {/if}
                Перезапустить Проводник
              </button>
            {/if}
          </footer>
        </article>
      {/each}
    </div>
  {/if}

  <div class="safety">
    <span class="safety-icon"><Icon name="info" size={17} /></span>
    <div>
      <strong>Перед изменениями создайте точку восстановления Windows.</strong>
      <p>
        Модули работают с реестром и службами. Приложение не отправляет журналы
        на сторонние серверы; все команды выполняются локально.
      </p>
    </div>
  </div>
</div>

<style>
  .tweaks-page {
    padding-bottom: 30px;
  }

  .page-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 24px;
    padding: 14px 4px 0;
    animation: fade-up 500ms var(--ease-out) both;
  }

  .page-head > div:first-child {
    max-width: 680px;
  }

  .eyebrow {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: rgba(255, 255, 255, 0.62);
    font-size: 9.5px;
    font-weight: 650;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  .live-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.65);
    animation: pulse-dot 2.2s ease-in-out infinite;
  }

  h1 {
    margin: 13px 0 0;
    font-size: clamp(30px, 3.8vw, 48px);
    font-weight: 620;
    letter-spacing: -0.05em;
    line-height: 1;
  }

  h1 span {
    color: transparent;
    background: linear-gradient(100deg, #ffffff, #c8c8ce 52%, #7b7b84);
    background-clip: text;
    -webkit-background-clip: text;
  }

  .lead {
    max-width: 560px;
    margin: 14px 0 0;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.6;
  }

  .head-stats {
    display: flex;
    gap: 8px;
  }

  .head-stat {
    display: flex;
    height: 40px;
    align-items: center;
    gap: 7px;
    padding: 0 12px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 12px;
    color: var(--muted);
    background: rgba(255, 255, 255, 0.025);
    font-size: 9.5px;
    white-space: nowrap;
  }

  .head-stat strong {
    color: var(--text);
    font-weight: 650;
  }

  .admin-banner {
    display: flex;
    align-items: center;
    gap: 14px;
    margin: 28px 4px 18px;
    padding: 16px 18px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 17px;
    background:
      linear-gradient(120deg, rgba(255, 255, 255, 0.045), transparent 60%),
      rgba(255, 255, 255, 0.018);
    animation: fade-up 520ms var(--ease-out) both;
  }

  .banner-icon {
    display: grid;
    width: 40px;
    height: 40px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.055);
  }

  .banner-copy {
    min-width: 0;
  }

  .banner-copy strong {
    font-size: 12px;
    font-weight: 620;
  }

  .banner-copy p {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 10px;
    line-height: 1.5;
  }

  .banner-action {
    display: inline-flex;
    height: 36px;
    flex: 0 0 auto;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    padding: 0 12px;
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 10px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.055);
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
  }

  .banner-action:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .admin-ok {
    display: flex;
    align-items: center;
    gap: 9px;
    margin: 26px 4px 18px;
    padding: 11px 15px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 13px;
    color: rgba(255, 255, 255, 0.66);
    background: rgba(255, 255, 255, 0.03);
    font-size: 10.5px;
  }

  .ok-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.6);
  }

  .tweak-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    margin-top: 18px;
  }

  .tweak-card {
    position: relative;
    display: flex;
    flex-direction: column;
    padding: 19px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.075);
    border-radius: 22px;
    background:
      linear-gradient(155deg, rgba(255, 255, 255, 0.04), transparent 40%),
      linear-gradient(180deg, rgba(17, 17, 20, 0.86), rgba(10, 10, 12, 0.92));
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.05),
      0 14px 34px rgba(0, 0, 0, 0.22);
    transition:
      transform 260ms var(--ease-out),
      border-color 260ms ease,
      box-shadow 260ms ease;
    animation: fade-up 500ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 60ms);
  }

  .tweak-card:hover {
    border-color: color-mix(in srgb, var(--accent) 30%, transparent);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.08),
      0 22px 48px rgba(0, 0, 0, 0.32),
      0 0 30px color-mix(in srgb, var(--accent) 10%, transparent);
    transform: translateY(-3px);
  }

  .tweak-card.enabled {
    border-color: color-mix(in srgb, var(--accent) 24%, transparent);
  }

  .tweak-aura {
    position: absolute;
    top: -80px;
    right: -60px;
    width: 180px;
    height: 180px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    filter: blur(60px);
    opacity: 0.35;
    pointer-events: none;
    transition: opacity 260ms ease;
  }

  .tweak-card.enabled .tweak-aura {
    opacity: 0.7;
  }

  .tweak-head {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .tweak-icon {
    display: grid;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-radius: 13px;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 7%, transparent);
  }

  .tweak-title {
    min-width: 0;
    padding-top: 2px;
  }

  .tweak-title h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 620;
    letter-spacing: -0.02em;
    line-height: 1.25;
  }

  .tweak-note {
    display: block;
    margin-top: 3px;
    color: rgba(255, 255, 255, 0.3);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8.5px;
    letter-spacing: 0.03em;
  }

  .tweak-head :global(.toggle) {
    margin-left: auto;
  }

  .tweak-description {
    margin: 14px 0 0;
    color: var(--muted);
    font-size: 10.5px;
    line-height: 1.6;
  }

  .warning-row {
    display: flex;
    align-items: flex-start;
    gap: 7px;
    margin-top: 11px;
    padding: 8px 10px;
    border: 1px solid rgba(255, 255, 255, 0.11);
    border-radius: 9px;
    color: var(--muted-strong);
    background: rgba(255, 255, 255, 0.025);
    font-size: 9px;
    line-height: 1.45;
  }

  .warning-row :global(svg) {
    flex: 0 0 auto;
    margin-top: 1px;
  }

  .hook-path {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 11px;
    padding: 9px 10px;
    border: 1px solid rgba(255, 255, 255, 0.055);
    border-radius: 9px;
    background: rgba(0, 0, 0, 0.2);
  }

  .hook-path span {
    color: rgba(255, 255, 255, 0.28);
    font-size: 8px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .hook-path code {
    overflow: hidden;
    color: var(--muted-strong);
    font-size: 8.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tweak-foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    padding-top: 16px;
  }

  .state-pill {
    display: inline-flex;
    height: 25px;
    align-items: center;
    gap: 6px;
    padding: 0 9px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 999px;
    color: rgba(255, 255, 255, 0.36);
    background: rgba(255, 255, 255, 0.025);
    font-size: 8.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .state-pill.on {
    border-color: color-mix(in srgb, var(--accent) 26%, transparent);
    color: color-mix(in srgb, var(--accent) 76%, white);
    background: color-mix(in srgb, var(--accent) 7%, transparent);
  }

  .state-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.3);
  }

  .state-pill.on .state-dot {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }

  .admin-tag {
    display: inline-flex;
    height: 25px;
    align-items: center;
    padding: 0 8px;
    border: 1px solid rgba(255, 255, 255, 0.11);
    border-radius: 999px;
    color: rgba(255, 255, 255, 0.46);
    background: rgba(255, 255, 255, 0.025);
    font-size: 8px;
    letter-spacing: 0.03em;
  }

  .inline-action {
    display: inline-flex;
    height: 27px;
    align-items: center;
    gap: 5px;
    margin-left: auto;
    padding: 0 9px;
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 8px;
    color: var(--muted-strong);
    background: rgba(255, 255, 255, 0.035);
    font-size: 8.5px;
    white-space: nowrap;
  }

  .inline-action:hover {
    border-color: rgba(255, 255, 255, 0.28);
    color: var(--text);
    background: rgba(255, 255, 255, 0.07);
  }

  .mini-spinner {
    width: 11px;
    height: 11px;
    border: 1.5px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .safety {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin: 20px 4px 0;
    padding: 15px 17px;
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 15px;
    background: rgba(255, 255, 255, 0.02);
  }

  .safety-icon {
    display: grid;
    width: 34px;
    height: 34px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 10px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.045);
  }

  .safety strong {
    font-size: 10.5px;
    font-weight: 600;
  }

  .safety p {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 9.5px;
    line-height: 1.5;
  }

  .skeleton-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    margin-top: 28px;
  }

  .skeleton-card {
    display: flex;
    min-height: 180px;
    flex-direction: column;
    gap: 11px;
    padding: 20px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 22px;
    background: rgba(255, 255, 255, 0.02);
    animation: fade-up 400ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 60ms);
  }

  .skeleton-line {
    height: 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.045);
  }

  .w-30 {
    width: 30%;
  }
  .w-40 {
    width: 40%;
  }
  .w-60 {
    width: 60%;
  }
  .w-80 {
    width: 80%;
  }

  .error-panel {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 26px;
    padding: 18px;
    border: 1px solid rgba(255, 107, 107, 0.24);
    border-radius: 16px;
    background: rgba(255, 107, 107, 0.04);
  }

  .error-icon {
    display: grid;
    width: 40px;
    height: 40px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 12px;
    color: var(--danger);
    background: rgba(255, 107, 107, 0.08);
  }

  .error-panel strong {
    font-size: 12px;
    font-weight: 600;
  }

  .error-panel p {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 10px;
  }

  .retry {
    display: inline-flex;
    height: 34px;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    padding: 0 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.04);
    font-size: 10px;
  }

  @media (max-width: 980px) {
    .tweak-grid,
    .skeleton-grid {
      grid-template-columns: 1fr;
    }

    .page-head {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
