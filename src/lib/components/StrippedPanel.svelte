<script lang="ts">
  import { api, humanizeError, type ToastTone, type TweakStatus } from "$lib/api";
  import Icon from "./Icon.svelte";
  import ToggleSwitch from "./ToggleSwitch.svelte";

  type Risk = "low" | "medium" | "high";
  type CategoryId = "fps" | "system" | "services" | "cleanup";

  type StrippedDefinition = {
    id: string;
    category: CategoryId;
    title: string;
    description: string;
    icon: string;
    accent: string;
    note: string;
    risk: Risk;
    requiresAdmin: boolean;
    action: boolean;
    warning?: string;
  };

  const categories: { id: CategoryId; title: string; hint: string }[] = [
    { id: "fps", title: "FPS и игры", hint: "Прирост кадров, снижение задержек и стабильность в играх." },
    { id: "system", title: "Система и облегчение", hint: "Меньше фоновых процессов, телеметрии и лишней нагрузки." },
    { id: "services", title: "Службы Windows", hint: "Отключение служб, которые нужны не всем." },
    { id: "cleanup", title: "Удаление и очистка", hint: "Одноразовые действия: удаление приложений и мусора." }
  ];

  const tweaks: StrippedDefinition[] = [
    {
      id: "fullscreen_optimizations",
      category: "fps",
      title: "Эксклюзивный полноэкранный режим",
      description:
        "Отключает «оптимизации во весь экран» Windows: меньше задержка ввода и стабильнее частота кадров в играх.",
      icon: "gamepad",
      accent: "#d4d4d8",
      note: "HKCU GameConfigStore",
      risk: "low",
      requiresAdmin: false,
      action: false,
    },
    {
      id: "ultimate_power_plan",
      category: "fps",
      title: "Схема «Максимальная производительность»",
      description:
        "Включает схему питания без агрессивной экономии: процессор не снижает частоты в играх. На ноутбуках сильно расходует батарею.",
      icon: "power",
      accent: "#d4d4d8",
      note: "powercfg ultimate",
      risk: "low",
      requiresAdmin: false,
      action: false,
      warning: "На ноутбуке батарея будет садиться быстрее. Выключение возвращает сбалансированную схему.",
    },
    {
      id: "power_throttling",
      category: "fps",
      title: "Без Power Throttling",
      description:
        "Запрещает Windows замедлять процессы ради экономии энергии. Помогает против просадок FPS на ноутбуках.",
      icon: "cpu",
      accent: "#d4d4d8",
      note: "HKLM PowerThrottling",
      risk: "medium",
      requiresAdmin: true,
      action: false,
      warning: "Нужна перезагрузка. Нагрев и расход энергии вырастут.",
    },
    {
      id: "network_throttling",
      category: "fps",
      title: "Сеть и отзывчивость мультимедиа",
      description:
        "Снимает сетевое ограничение мультимедиа и повышает отзывчивость системы: ниже пинг и меньше фризов в онлайн-играх.",
      icon: "globe",
      accent: "#d4d4d8",
      note: "HKLM Multimedia\\SystemProfile",
      risk: "medium",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "games_priority",
      category: "fps",
      title: "Приоритет игр в планировщике",
      description:
        "Повышает приоритет задач категории Games в планировщике мультимедиа Windows.",
      icon: "gamepad",
      accent: "#d4d4d8",
      note: "HKLM SystemProfile\\Tasks\\Games",
      risk: "medium",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "mouse_acceleration",
      category: "fps",
      title: "Мышь без ускорения",
      description:
        "Отключает «повышенную точность указателя»: движение мыши одинаково на любой скорости, как требуется в шутерах.",
      icon: "arrow",
      accent: "#d4d4d8",
      note: "HKCU Control Panel\\Mouse",
      risk: "low",
      requiresAdmin: false,
      action: false,
      warning: "Вступает в силу после повторного входа в Windows.",
    },
    {
      id: "sticky_keys_prompts",
      category: "fps",
      title: "Без окон залипания клавиш",
      description:
        "Отключает всплывающие окна залипания и фильтрации ввода, которые сворачивают игру при многократном нажатии Shift.",
      icon: "terminal",
      accent: "#d4d4d8",
      note: "HKCU Accessibility",
      risk: "low",
      requiresAdmin: false,
      action: false,
    },
    {
      id: "visual_effects_performance",
      category: "fps",
      title: "Визуальные эффекты: производительность",
      description:
        "Отключает анимации окон и прозрачность. Интерфейс выглядит проще, но работает быстрее.",
      icon: "sliders",
      accent: "#d4d4d8",
      note: "HKCU VisualEffects",
      risk: "low",
      requiresAdmin: false,
      action: false,
      warning: "Вступает в силу после повторного входа в Windows.",
    },
    {
      id: "hvci_memory_integrity",
      category: "fps",
      title: "Целостность памяти (HVCI / VBS)",
      description:
        "Отключает изоляцию ядра на основе виртуализации. На некоторых системах даёт заметный прирост FPS, но снижает защиту от вредоносного кода.",
      icon: "shield",
      accent: "#d4d4d8",
      note: "HKLM DeviceGuard",
      risk: "high",
      requiresAdmin: true,
      action: false,
      warning: "Снижает безопасность системы. Нужна перезагрузка.",
    },
    {
      id: "background_apps",
      category: "system",
      title: "Фоновые приложения",
      description:
        "Запрещает приложениям Microsoft Store работать в фоне и потреблять ресурсы, когда вы играете.",
      icon: "grid",
      accent: "#d4d4d8",
      note: "HKCU BackgroundAccessApplications",
      risk: "low",
      requiresAdmin: false,
      action: false,
    },
    {
      id: "ntfs_last_access",
      category: "system",
      title: "Диск без отметок доступа",
      description:
        "Отключает запись времени последнего доступа к файлам: меньше лишних операций записи на диск.",
      icon: "archive",
      accent: "#d4d4d8",
      note: "HKLM FileSystem",
      risk: "low",
      requiresAdmin: true,
      action: false,
      warning: "Нужна перезагрузка.",
    },
    {
      id: "delivery_optimization",
      category: "system",
      title: "Без раздачи обновлений (P2P)",
      description:
        "Запрещает Windows раздавать скачанные обновления другим компьютерам через ваш интернет-канал.",
      icon: "download",
      accent: "#d4d4d8",
      note: "HKLM DeliveryOptimization policy",
      risk: "low",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "widgets_news",
      category: "system",
      title: "Виджеты и новости",
      description:
        "Отключает панель виджетов и ленту новостей, которые постоянно работают в фоне.",
      icon: "window",
      accent: "#d4d4d8",
      note: "HKLM Dsh policy",
      risk: "low",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "telemetry_tasks",
      category: "system",
      title: "Задачи телеметрии",
      description:
        "Отключает задачи планировщика, которые собирают и отправляют диагностические данные (Compatibility Appraiser, CEIP и др.).",
      icon: "shield",
      accent: "#d4d4d8",
      note: "Task Scheduler",
      risk: "medium",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "reserved_storage",
      category: "system",
      title: "Зарезервированное хранилище",
      description:
        "Отключает резерв на обновления Windows и освобождает несколько гигабайт на системном диске.",
      icon: "archive",
      accent: "#d4d4d8",
      note: "DISM ReservedStorage",
      risk: "medium",
      requiresAdmin: true,
      action: false,
      warning: "Нужна перезагрузка. Для установки крупных обновлений может понадобиться больше свободного места.",
    },
    {
      id: "svc_sysmain",
      category: "services",
      title: "Служба SysMain (Superfetch)",
      description:
        "Отключает предзагрузку программ в память. На SSD почти не нужна, снижает фоновую нагрузку на диск и ОЗУ.",
      icon: "cpu",
      accent: "#d4d4d8",
      note: "service SysMain",
      risk: "medium",
      requiresAdmin: true,
      action: false,
      warning: "На жёстких дисках (HDD) запуск программ может стать медленнее.",
    },
    {
      id: "svc_search_indexing",
      category: "services",
      title: "Индексирование Windows Search",
      description:
        "Отключает индексацию файлов — меньше нагрузки на диск и процессор. Поиск файлов станет медленнее.",
      icon: "search",
      accent: "#d4d4d8",
      note: "service WSearch",
      risk: "medium",
      requiresAdmin: true,
      action: false,
      warning: "Поиск по файлам и в Outlook станет заметно медленнее.",
    },
    {
      id: "svc_maps",
      category: "services",
      title: "Служба карт",
      description:
        "Отключает службу загруженных карт Windows, которая работает в фоне.",
      icon: "globe",
      accent: "#d4d4d8",
      note: "service MapsBroker",
      risk: "low",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "svc_error_reporting",
      category: "services",
      title: "Отчёты об ошибках",
      description:
        "Отключает службу отправки отчётов об ошибках в Microsoft.",
      icon: "alert",
      accent: "#d4d4d8",
      note: "service WerSvc",
      risk: "low",
      requiresAdmin: true,
      action: false,
    },
    {
      id: "svc_xbox",
      category: "services",
      title: "Службы Xbox",
      description:
        "Отключает фоновые службы Xbox Live, сохранений и аксессуаров.",
      icon: "gamepad",
      accent: "#d4d4d8",
      note: "services Xbl*, Xbox*",
      risk: "high",
      requiresAdmin: true,
      action: false,
      warning: "Перестанут работать Xbox, Game Pass и игры из Microsoft Store с входом в Xbox Live.",
    },
    {
      id: "svc_print_spooler",
      category: "services",
      title: "Диспетчер печати",
      description:
        "Отключает службу печати. Подходит, если вы не пользуетесь принтерами и PDF-принтером.",
      icon: "terminal",
      accent: "#d4d4d8",
      note: "service Spooler",
      risk: "medium",
      requiresAdmin: true,
      action: false,
      warning: "Печать, включая «Печать в PDF», работать не будет.",
    },
    {
      id: "remove_bloat_apps",
      category: "cleanup",
      title: "Удалить встроенные приложения",
      description:
        "Удаляет предустановленный мусор: Новости, Погоду, Solitaire, Teams, Clipchamp, Phone Link, Feedback Hub, 3D-программы, рекламные Netflix, TikTok, Spotify и подобные.",
      icon: "archive",
      accent: "#d4d4d8",
      note: "Appx removal",
      risk: "high",
      requiresAdmin: true,
      action: true,
      warning: "Приложения удаляются для всех пользователей. Вернуть их можно только из Microsoft Store.",
    },
    {
      id: "remove_onedrive",
      category: "cleanup",
      title: "Удалить OneDrive",
      description:
        "Полностью удаляет OneDrive и его фоновую синхронизацию. Локальные файлы в папке OneDrive остаются на месте.",
      icon: "folder",
      accent: "#d4d4d8",
      note: "OneDriveSetup /uninstall",
      risk: "medium",
      requiresAdmin: false,
      action: true,
      warning: "Синхронизация с облаком прекратится. Убедитесь, что нужные файлы есть и на диске.",
    },
    {
      id: "clean_temp",
      category: "cleanup",
      title: "Очистить временные файлы",
      description:
        "Удаляет временные файлы пользователя и системы, кэш загрузки обновлений и сбрасывает кэш DNS.",
      icon: "refresh",
      accent: "#d4d4d8",
      note: "TEMP + SoftwareDistribution",
      risk: "low",
      requiresAdmin: true,
      action: true,
    },
  ];

  const riskLabel: Record<Risk, string> = {
    low: "Низкий риск",
    medium: "Средний риск",
    high: "Высокий риск",
  };

  let {
    onNotify,
    onLeave,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
    onLeave: () => void;
  } = $props();

  let acknowledged = $state(false);
  let statuses = $state<TweakStatus[]>([]);
  let admin = $state(false);
  let loading = $state(false);
  let loadError = $state("");
  let busy = $state<Record<string, boolean>>({});
  let pending = $state<{ id: string; next: boolean } | null>(null);
  let restoreBusy = $state(false);
  let elevating = $state(false);

  let enabledCount = $derived(statuses.filter((item) => item.enabled).length);

  function accept() {
    acknowledged = true;
    load();
  }

  async function load() {
    loading = true;
    loadError = "";
    try {
      const [nextStatuses, nextAdmin] = await Promise.all([
        api.strippedStates(),
        api.adminStatus(),
      ]);
      statuses = nextStatuses;
      admin = nextAdmin;
    } catch (caught) {
      loadError = humanizeError(caught);
      onNotify("error", "Не удалось загрузить состояние модулей", loadError);
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

  function request(tweak: StrippedDefinition, next: boolean) {
    const needsConfirm = next && (tweak.action || tweak.risk !== "low");
    if (needsConfirm) {
      pending = { id: tweak.id, next };
      return;
    }
    run(tweak, next);
  }

  async function run(tweak: StrippedDefinition, next: boolean) {
    pending = null;
    busy = { ...busy, [tweak.id]: true };
    try {
      const message = await api.applyStripped(tweak.id, next);
      statuses = statuses.some((item) => item.id === tweak.id)
        ? statuses.map((item) =>
            item.id === tweak.id ? { ...item, enabled: next } : item,
          )
        : [...statuses, { id: tweak.id, enabled: next, requiresAdmin: tweak.requiresAdmin }];
      onNotify("success", tweak.action ? "Действие выполнено" : next ? "Модуль активирован" : "Модуль отключён", message);
    } catch (caught) {
      onNotify("error", "Не удалось применить изменение", humanizeError(caught));
    } finally {
      busy = { ...busy, [tweak.id]: false };
    }
  }

  async function restorePoint() {
    restoreBusy = true;
    try {
      const message = await api.createRestorePoint();
      onNotify("success", "Точка восстановления", message);
    } catch (caught) {
      onNotify("error", "Не удалось создать точку восстановления", humanizeError(caught));
    } finally {
      restoreBusy = false;
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

{#if !acknowledged}
  <div class="gate">
    <div class="gate-card">
      <span class="beta-badge">BETA</span>
      <span class="gate-icon"><Icon name="alert" size={30} /></span>
      <h1>Раздел <span>Stripped-down</span></h1>
      <p class="gate-lead">
        Это <strong>бета-раздел</strong>, он ещё не готов полностью. Здесь собраны
        агрессивные настройки: отключение служб, удаление встроенных приложений и
        правки реестра для повышения FPS и облегчения Windows.
      </p>
      <ul class="gate-list">
        <li>Это <strong>опасно</strong>: часть изменений может нарушить работу программ и самой системы.</li>
        <li>Некоторые действия (удаление приложений, очистка) нельзя отменить одной кнопкой.</li>
        <li>Перед началом создайте точку восстановления Windows и сохраните важные данные.</li>
        <li>Вы используете раздел <strong>на свой страх и риск</strong>. Автор ответственности не несёт.</li>
      </ul>
      <div class="gate-actions">
        <button type="button" class="btn ghost" onclick={onLeave}>Вернуться назад</button>
        <button type="button" class="btn danger" onclick={accept}>
          Я понимаю риски и продолжаю
        </button>
      </div>
    </div>
  </div>
{:else}
  <div class="tweaks-page">
    <div class="beta-banner">
      <span class="beta-badge small">BETA</span>
      <div>
        <strong>Бета-раздел: не готов полностью и опасен.</strong>
        <p>Работайте на свой страх и риск. Сначала создайте точку восстановления.</p>
      </div>
    </div>

    <header class="page-head">
      <div>
        <p class="eyebrow">
          <span class="live-dot"></span>
          System / Stripped-down
        </p>
        <h1>Урезанная <span>Windows</span></h1>
        <p class="lead">
          Облегчение системы и повышение FPS в играх: службы, реестр, лишние
          приложения. Большинство модулей обратимы, кроме удаления приложений и очистки.
        </p>
      </div>
      <div class="head-stats">
        <div class="head-stat">
          <Icon name="sliders" size={15} />
          <span><strong>{tweaks.length}</strong> модулей</span>
        </div>
        <div class="head-stat">
          <Icon name="check" size={15} />
          <span><strong>{enabledCount}</strong> активно</span>
        </div>
        <div class="head-stat">
          <Icon name="shield" size={15} />
          <span><strong>{admin ? "UAC ON" : "UAC OFF"}</strong></span>
        </div>
      </div>
      <img class="hero-art" src="/art/hero-stripped.svg" alt="" />
  </header>

    <div class="restore-bar">
      <span class="restore-icon"><Icon name="archive" size={18} /></span>
      <div class="restore-copy">
        <strong>Точка восстановления Windows</strong>
        <p>Создайте её перед изменениями, чтобы при проблемах откатить систему.</p>
      </div>
      <button type="button" class="btn" disabled={restoreBusy || !admin} onclick={restorePoint}>
        {#if restoreBusy}<span class="mini-spinner"></span>{:else}<Icon name="archive" size={14} />{/if}
        Создать точку
      </button>
    </div>

    {#if !admin}
      <div class="admin-banner">
        <span class="banner-icon"><Icon name="lock" size={18} /></span>
        <div class="banner-copy">
          <strong>Ограниченный режим</strong>
          <p>
            Большинству модулей нужны права администратора, а точка восстановления без них
            недоступна. Перезапустите приложение с повышенными правами.
          </p>
        </div>
        <button type="button" class="banner-action" disabled={elevating} onclick={elevate}>
          {#if elevating}<span class="mini-spinner"></span>{:else}<Icon name="power" size={14} />{/if}
          Перезапустить с UAC
        </button>
      </div>
    {/if}

    {#if loading}
      <div class="skeleton-grid">
        {#each Array(6) as _, index (index)}
          <div class="skeleton-card" style={`--i:${index}`}>
            <span class="skeleton-line w-40"></span>
            <span class="skeleton-line w-80"></span>
            <span class="skeleton-line w-60"></span>
          </div>
        {/each}
      </div>
    {:else if loadError}
      <div class="error-panel">
        <span class="error-icon"><Icon name="alert" size={20} /></span>
        <div>
          <strong>Состояние модулей недоступно</strong>
          <p>{loadError}</p>
        </div>
        <button type="button" class="retry" onclick={load}>
          <Icon name="refresh" size={14} />
          Повторить
        </button>
      </div>
    {:else}
      {#each categories as category (category.id)}
        <section class="group">
          <div class="group-head">
            <h2>{category.title}</h2>
            <p>{category.hint}</p>
          </div>
          <div class="tweak-grid">
            {#each tweaks.filter((item) => item.category === category.id) as tweak, index (tweak.id)}
              {@const status = statusOf(tweak.id)}
              <article
                class="tweak-card"
                class:enabled={status.enabled}
                style={`--accent:${tweak.accent}; --i:${index};`}
              >
                <div class="tweak-banner"><img src={`/art/${tweak.id}.svg`} alt="" loading="lazy" /></div>
                <div class="tweak-aura"></div>
                <header class="tweak-head">
                  <span class="tweak-icon"><Icon name={tweak.icon} size={20} /></span>
                  <div class="tweak-title">
                    <h2>{tweak.title}</h2>
                    <span class="tweak-note">{tweak.note}</span>
                  </div>
                  {#if tweak.action}
                    <button
                      type="button"
                      class="btn small"
                      disabled={busy[tweak.id] ?? false}
                      onclick={() => request(tweak, true)}
                    >
                      {#if busy[tweak.id]}<span class="mini-spinner"></span>{/if}
                      {status.enabled ? "Повторить" : "Запустить"}
                    </button>
                  {:else}
                    <ToggleSwitch
                      checked={status.enabled}
                      disabled={busy[tweak.id] ?? false}
                      loading={busy[tweak.id] ?? false}
                      label={tweak.title}
                      onChange={(next) => request(tweak, next)}
                    />
                  {/if}
                </header>

                <p class="tweak-description">{tweak.description}</p>

                {#if tweak.warning}
                  <div class="warning-row">
                    <Icon name="alert" size={13} />
                    {tweak.warning}
                  </div>
                {/if}

                {#if pending?.id === tweak.id}
                  <div class="confirm-row" class:high={tweak.risk === "high"}>
                    <span>
                      {tweak.risk === "high"
                        ? "Опасное действие. Вы точно хотите продолжить?"
                        : "Применить изменение?"}
                    </span>
                    <div>
                      <button type="button" class="btn small ghost" onclick={() => (pending = null)}>Отмена</button>
                      <button type="button" class="btn small danger" onclick={() => run(tweak, pending?.next ?? true)}>
                        Да, применить
                      </button>
                    </div>
                  </div>
                {/if}

                <footer class="tweak-foot">
                  <span class="state-pill" class:on={status.enabled}>
                    <span class="state-dot"></span>
                    {tweak.action
                      ? status.enabled ? "Выполнено" : "Не выполнялось"
                      : status.enabled ? "Включено" : "Выключено"}
                  </span>
                  <span class={`risk-pill ${tweak.risk}`}>{riskLabel[tweak.risk]}</span>
                  {#if tweak.requiresAdmin}
                    <span class="admin-tag">Требуется UAC</span>
                  {/if}
                </footer>
              </article>
            {/each}
          </div>
        </section>
      {/each}
    {/if}

    <div class="safety">
      <span class="safety-icon"><Icon name="info" size={17} /></span>
      <div>
        <strong>Раздел в стадии бета-тестирования.</strong>
        <p>
          Часть изменений вступает в силу после перезагрузки или повторного входа.
          Приложение не отправляет данные на сторонние серверы, все команды выполняются локально.
        </p>
      </div>
    </div>
  </div>
{/if}

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
    color: rgb(var(--fg) / 0.62);
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
    box-shadow: 0 0 10px rgb(var(--fg) / 0.65);
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
    border: 1px solid rgb(var(--fg) / 0.07);
    border-radius: 12px;
    color: var(--muted);
    background: rgb(var(--fg) / 0.025);
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
    border: 1px solid rgb(var(--fg) / 0.12);
    border-radius: 17px;
    background:
      linear-gradient(120deg, rgb(var(--fg) / 0.045), transparent 60%),
      rgb(var(--fg) / 0.018);
    animation: fade-up 520ms var(--ease-out) both;
  }

  .banner-icon {
    display: grid;
    width: 40px;
    height: 40px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.1);
    border-radius: 12px;
    color: var(--text);
    background: rgb(var(--fg) / 0.055);
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
    border: 1px solid rgb(var(--fg) / 0.18);
    border-radius: 10px;
    color: var(--text);
    background: rgb(var(--fg) / 0.055);
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
  }

  .banner-action:hover {
    background: rgb(var(--fg) / 0.1);
  }

  .admin-ok {
    display: flex;
    align-items: center;
    gap: 9px;
    margin: 26px 4px 18px;
    padding: 11px 15px;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 13px;
    color: rgb(var(--fg) / 0.66);
    background: rgb(var(--fg) / 0.03);
    font-size: 10.5px;
  }

  .ok-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 10px rgb(var(--fg) / 0.6);
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
    border: 1px solid rgb(var(--fg) / 0.075);
    border-radius: 22px;
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.04), transparent 40%),
      linear-gradient(180deg, rgb(var(--panel) / 0.86), rgb(var(--panel) / 0.92));
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.05),
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
      inset 0 1px 0 rgb(var(--fg) / 0.08),
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
    color: rgb(var(--fg) / 0.3);
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
    border: 1px solid rgb(var(--fg) / 0.11);
    border-radius: 9px;
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.025);
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
    border: 1px solid rgb(var(--fg) / 0.055);
    border-radius: 9px;
    background: rgba(0, 0, 0, 0.2);
  }

  .hook-path span {
    color: rgb(var(--fg) / 0.28);
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
    border: 1px solid rgb(var(--fg) / 0.07);
    border-radius: 999px;
    color: rgb(var(--fg) / 0.36);
    background: rgb(var(--fg) / 0.025);
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
    background: rgb(var(--fg) / 0.3);
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
    border: 1px solid rgb(var(--fg) / 0.11);
    border-radius: 999px;
    color: rgb(var(--fg) / 0.46);
    background: rgb(var(--fg) / 0.025);
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
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 8px;
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.035);
    font-size: 8.5px;
    white-space: nowrap;
  }

  .inline-action:hover {
    border-color: rgb(var(--fg) / 0.28);
    color: var(--text);
    background: rgb(var(--fg) / 0.07);
  }

  .mini-spinner {
    width: 11px;
    height: 11px;
    border: 1.5px solid rgb(var(--fg) / 0.2);
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
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 15px;
    background: rgb(var(--fg) / 0.02);
  }

  .safety-icon {
    display: grid;
    width: 34px;
    height: 34px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 10px;
    color: var(--text);
    background: rgb(var(--fg) / 0.045);
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
    border: 1px solid rgb(var(--fg) / 0.05);
    border-radius: 22px;
    background: rgb(var(--fg) / 0.02);
    animation: fade-up 400ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 60ms);
  }

  .skeleton-line {
    height: 9px;
    border-radius: 999px;
    background: rgb(var(--fg) / 0.045);
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
    border: 1px solid rgb(var(--fg) / 0.12);
    border-radius: 9px;
    background: rgb(var(--fg) / 0.04);
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

  .group {
    margin-top: 26px;
  }

  .group-head h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 620;
    letter-spacing: -0.03em;
  }

  .group-head p {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 11px;
  }

  .gate {
    display: grid;
    place-items: center;
    min-height: 70vh;
    padding: 20px 4px;
    animation: fade-up 500ms var(--ease-out) both;
  }

  .gate-card {
    position: relative;
    max-width: 640px;
    padding: 34px 34px 28px;
    border: 1px solid rgba(255, 107, 107, 0.35);
    border-radius: 26px;
    background:
      linear-gradient(155deg, rgba(255, 107, 107, 0.1), transparent 45%),
      linear-gradient(rgb(var(--panel) / 0.92), rgb(var(--panel) / 0.96));
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5), 0 0 60px rgba(255, 107, 107, 0.08);
  }

  .gate-card h1 {
    font-size: clamp(26px, 3vw, 36px);
  }

  .gate-icon {
    display: grid;
    place-items: center;
    width: 58px;
    height: 58px;
    margin-bottom: 16px;
    color: var(--danger);
    border: 1px solid rgba(255, 107, 107, 0.4);
    border-radius: 18px;
    background: rgba(255, 107, 107, 0.08);
  }

  .gate-lead {
    margin: 16px 0 0;
    color: var(--muted-strong);
    font-size: 12.5px;
    line-height: 1.65;
  }

  .gate-list {
    display: grid;
    gap: 9px;
    margin: 18px 0 0;
    padding: 0;
    list-style: none;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.55;
  }

  .gate-list li {
    padding: 10px 12px;
    border: 1px solid rgb(var(--fg) / 0.08);
    border-radius: 12px;
    background: rgb(var(--fg) / 0.03);
  }

  .gate-list strong,
  .gate-lead strong {
    color: var(--text);
  }

  .gate-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 24px;
  }

  .beta-badge {
    position: absolute;
    top: 18px;
    right: 20px;
    padding: 4px 10px;
    color: #1a0606;
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 0.16em;
    border-radius: 999px;
    background: var(--danger);
  }

  .beta-badge.small {
    position: static;
    flex: none;
  }

  .beta-banner {
    display: flex;
    align-items: center;
    gap: 14px;
    margin: 6px 4px 0;
    padding: 12px 16px;
    border: 1px solid rgba(255, 107, 107, 0.35);
    border-radius: 16px;
    background: rgba(255, 107, 107, 0.07);
    animation: fade-up 500ms var(--ease-out) both;
  }

  .beta-banner strong {
    font-size: 11.5px;
    font-weight: 620;
  }

  .beta-banner p {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 10.5px;
  }

  .restore-bar {
    display: flex;
    align-items: center;
    gap: 14px;
    margin: 18px 4px 0;
    padding: 14px 16px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 16px;
    background: rgb(var(--fg) / 0.03);
  }

  .restore-icon {
    display: grid;
    place-items: center;
    flex: none;
    width: 38px;
    height: 38px;
    border-radius: 12px;
    background: rgb(var(--fg) / 0.06);
  }

  .restore-copy {
    flex: 1;
    min-width: 0;
  }

  .restore-copy strong {
    font-size: 11.5px;
    font-weight: 620;
  }

  .restore-copy p {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 10.5px;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 36px;
    padding: 0 15px;
    color: var(--text);
    font-size: 11px;
    font-weight: 600;
    border: 1px solid rgb(var(--fg) / 0.18);
    border-radius: 11px;
    background: rgb(var(--fg) / 0.07);
    transition: background 0.2s ease, transform 0.2s ease;
  }

  .btn:hover:not(:disabled) {
    background: rgb(var(--fg) / 0.13);
  }

  .btn:disabled {
    opacity: 0.5;
  }

  .btn.small {
    height: 30px;
    padding: 0 11px;
    font-size: 10px;
    margin-left: auto;
  }

  .btn.ghost {
    background: transparent;
  }

  .btn.danger {
    color: #fff;
    border-color: rgba(255, 107, 107, 0.6);
    background: rgba(255, 107, 107, 0.22);
  }

  .btn.danger:hover:not(:disabled) {
    background: rgba(255, 107, 107, 0.34);
  }

  .risk-pill {
    display: inline-flex;
    align-items: center;
    height: 25px;
    padding: 0 9px;
    font-size: 8.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 999px;
    color: rgb(var(--fg) / 0.6);
  }

  .risk-pill.medium {
    color: #f5c26b;
    border-color: rgba(245, 194, 107, 0.35);
    background: rgba(245, 194, 107, 0.08);
  }

  .risk-pill.high {
    color: var(--danger);
    border-color: rgba(255, 107, 107, 0.45);
    background: rgba(255, 107, 107, 0.1);
  }

  .confirm-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-top: 12px;
    padding: 10px 12px;
    color: var(--muted-strong);
    font-size: 10.5px;
    border: 1px solid rgba(245, 194, 107, 0.3);
    border-radius: 12px;
    background: rgba(245, 194, 107, 0.07);
  }

  .confirm-row.high {
    border-color: rgba(255, 107, 107, 0.45);
    background: rgba(255, 107, 107, 0.09);
  }

  .confirm-row > div {
    display: flex;
    gap: 8px;
  }

  .confirm-row .btn.small {
    margin-left: 0;
  }
  .hero-art {
    flex: none;
    width: 206px;
    height: 116px;
    object-fit: cover;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 18px;
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.4);
    pointer-events: none;
    -webkit-user-drag: none;
  }

  @media (max-width: 980px) {
    .hero-art {
      display: none;
    }
  }
  .tweak-banner {
    position: relative;
    height: 78px;
    margin: -19px -19px 16px;
    overflow: hidden;
    border-bottom: 1px solid rgb(var(--fg) / 0.08);
    -webkit-mask-image: linear-gradient(#000 55%, transparent);
    mask-image: linear-gradient(#000 55%, transparent);
  }

  .tweak-banner img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0.88;
    pointer-events: none;
    -webkit-user-drag: none;
  }
</style>
