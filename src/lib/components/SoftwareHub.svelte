<script lang="ts">
  import type { CatalogApp, JobState } from "$lib/api";
  import AppCard from "./AppCard.svelte";
  import Icon from "./Icon.svelte";

  type Category = {
    id: CatalogApp["category"];
    title: string;
    subtitle: string;
    icon: string;
  };

  const categories: Category[] = [
    {
      id: "system",
      title: "System & Tools",
      subtitle: "Архиваторы, редакторы и системные утилиты для чистой системы.",
      icon: "cpu",
    },
    {
      id: "browsers",
      title: "Browsers & Communication",
      subtitle: "Браузеры, мессенджеры и голосовые каналы для связи.",
      icon: "globe",
    },
    {
      id: "gaming",
      title: "Gaming & Launchers",
      subtitle: "Игровые библиотеки и шаблоны лаунчеров с прямыми ссылками.",
      icon: "gamepad",
    },
    {
      id: "production",
      title: "Production & Custom",
      subtitle: "Медиа-станции и установщики с пользовательскими параметрами.",
      icon: "film",
    },
  ];

  let {
    apps = [],
    loading = false,
    error = "",
    catalogVersion = 0,
    jobs = {},
    selectedIds = [],
    selectionDisabled = false,
    onToggleSelect,
    onDetails,
    onRetry,
  }: {
    apps?: CatalogApp[];
    loading?: boolean;
    error?: string;
    catalogVersion?: number;
    jobs?: Record<string, JobState>;
    selectedIds?: string[];
    selectionDisabled?: boolean;
    onToggleSelect: (app: CatalogApp) => void;
    onDetails: (app: CatalogApp) => void;
    onRetry: () => void;
  } = $props();

  let query = $state("");
  let searchInput = $state<HTMLInputElement>();

  function handleShortcut(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      searchInput?.focus();
    }
  }

  let selectedSet = $derived(new Set(selectedIds));
  let normalizedQuery = $derived(query.trim().toLowerCase());
  let filtered = $derived(
    apps.filter((app) => {
      if (!normalizedQuery) return true;
      const haystack = [
        app.name,
        app.summary,
        app.wingetId ?? "",
        app.category,
        ...Object.keys(app.customConfig),
      ]
        .join(" ")
        .toLowerCase();
      return haystack.includes(normalizedQuery);
    }),
  );

  let readyCount = $derived(apps.filter((app) => !app.placeholder).length);
  let templateCount = $derived(apps.filter((app) => app.placeholder).length);

  function appsFor(category: Category): CatalogApp[] {
    return filtered.filter((app) => app.category === category.id);
  }
</script>

<svelte:window onkeydown={handleShortcut} />

<div class="hub">
  <header class="page-head">
    <div class="head-copy">
      <p class="eyebrow">
        <span class="live-dot"></span>
        Software Hub / Catalog v{catalogVersion || "—"}
      </p>
      <h1>
        Ваша система.
        <span>Собрана заново.</span>
      </h1>
      <p class="lead">
        Отметьте нужные приложения в каталоге, проверьте список и запустите
        установку одной кнопкой. Шаблоны можно адаптировать под собственный
        сценарий.
      </p>
    </div>

    <div class="search-shell">
      <Icon name="search" size={17} />
      <input
        type="text"
        bind:this={searchInput}
        bind:value={query}
        spellcheck="false"
        placeholder="Поиск приложений, ID или настроек..."
        aria-label="Поиск приложений"
      />
      {#if query}
        <button
          type="button"
          class="clear-search"
          aria-label="Очистить поиск"
          onclick={() => (query = "")}
        >
          <Icon name="x" size={13} />
        </button>
      {/if}
      <span class="search-key">Ctrl K</span>
    </div>
    <img class="hero-art" src="/art/hero-apps.svg" alt="" />
  </header>

  <div class="stats">
    <div class="stat">
      <span class="stat-icon"><Icon name="download" size={15} /></span>
      <span class="stat-copy">
        <strong>{readyCount}</strong>
        <small>готовы к установке</small>
      </span>
    </div>
    <div class="stat">
      <span class="stat-icon"><Icon name="spark" size={15} /></span>
      <span class="stat-copy">
        <strong>{templateCount}</strong>
        <small>шаблонов с параметрами</small>
      </span>
    </div>
    <div class="stat">
      <span class="stat-icon"><Icon name="grid" size={15} /></span>
      <span class="stat-copy">
        <strong>{categories.length}</strong>
        <small>визуальные категории</small>
      </span>
    </div>
    <div class="stat stat-note">
      <span class="status-line"></span>
      <span>
        {selectedIds.length > 0
          ? `Выбрано ${selectedIds.length} — нажмите «Скачать выбранное»`
          : "Каталог собран локально и защищён от строкового анализа"}
      </span>
    </div>
  </div>

  {#if error}
    <div class="state state-error">
      <span class="state-icon"><Icon name="alert" size={22} /></span>
      <div>
        <strong>Не удалось прочитать каталог</strong>
        <p>{error}</p>
      </div>
      <button type="button" class="state-action" onclick={onRetry}>
        <Icon name="refresh" size={15} />
        Повторить
      </button>
    </div>
  {:else if loading}
    <div class="loading-block">
      <div class="loading-head">
        <span class="loader"></span>
        <span>Расшифровка каталога и подготовка карточек...</span>
      </div>
      <div class="skeleton-grid">
        {#each Array(8) as _, index (index)}
          <div class="skeleton-card" style={`--i:${index}`}>
            <span class="skeleton-icon"></span>
            <span class="skeleton-line w-40"></span>
            <span class="skeleton-line w-80"></span>
            <span class="skeleton-line w-60"></span>
            <span class="skeleton-line w-30"></span>
          </div>
        {/each}
      </div>
    </div>
  {:else if filtered.length === 0}
    <div class="state">
      <span class="state-icon"><Icon name="search" size={22} /></span>
      <div>
        <strong>Ничего не найдено</strong>
        <p>Попробуйте другой запрос или сбросьте фильтр.</p>
      </div>
      <button type="button" class="state-action" onclick={() => (query = "")}>
        <Icon name="x" size={15} />
        Сбросить
      </button>
    </div>
  {:else}
    {#each categories as category, categoryIndex (category.id)}
      {@const list = appsFor(category)}
      {#if list.length > 0}
        <section class="category" style={`--ci:${categoryIndex};`}>
          <header class="category-head">
            <span class="category-icon">
              <Icon name={category.icon} size={18} />
            </span>
            <div class="category-copy">
              <div class="category-title">
                <h2>{category.title}</h2>
                <span class="category-count">{list.length}</span>
              </div>
              <p>{category.subtitle}</p>
            </div>
            <span class="category-line"></span>
          </header>

          <div class="card-grid">
            {#each list as app, index (app.id)}
              <AppCard
                {app}
                index={index}
                job={jobs[app.id]}
                selected={selectedSet.has(app.id)}
                disabled={selectionDisabled}
                onToggleSelect={onToggleSelect}
                onDetails={onDetails}
              />
            {/each}
          </div>
        </section>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .hub {
    padding-bottom: 30px;
  }

  .page-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 28px;
    padding: 14px 4px 0;
    animation: fade-up 500ms var(--ease-out) both;
  }

  .head-copy {
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
    font-size: clamp(32px, 4.2vw, 55px);
    font-weight: 620;
    letter-spacing: -0.055em;
    line-height: 0.98;
  }

  h1 span {
    display: block;
    color: transparent;
    background: linear-gradient(100deg, #ffffff, #c8c8ce 48%, #7b7b84);
    background-clip: text;
    -webkit-background-clip: text;
  }

  .lead {
    max-width: 560px;
    margin: 16px 0 0;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.6;
  }

  .search-shell {
    display: flex;
    width: min(340px, 100%);
    height: 46px;
    flex: 0 0 auto;
    align-items: center;
    gap: 9px;
    padding: 0 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 15px;
    color: rgba(255, 255, 255, 0.35);
    background: rgba(8, 8, 10, 0.62);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    transition:
      border-color 220ms ease,
      box-shadow 220ms ease,
      background 220ms ease;
  }

  .search-shell:focus-within {
    border-color: rgba(255, 255, 255, 0.4);
    background: rgba(14, 14, 17, 0.84);
    box-shadow:
      0 0 0 3px rgba(255, 255, 255, 0.055),
      0 0 26px rgba(255, 255, 255, 0.05);
  }

  .search-shell input {
    min-width: 0;
    flex: 1;
    border: 0;
    outline: 0;
    color: var(--text);
    background: transparent;
    font-size: 11.5px;
  }

  .search-shell input::placeholder {
    color: rgba(255, 255, 255, 0.28);
  }

  .clear-search {
    display: grid;
    width: 22px;
    height: 22px;
    place-items: center;
    border-radius: 7px;
    color: var(--muted);
    background: rgba(255, 255, 255, 0.06);
  }

  .search-key {
    padding: 3px 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.3);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8px;
  }

  .stats {
    display: grid;
    grid-template-columns: auto auto auto 1fr;
    gap: 10px;
    margin: 28px 4px 34px;
    animation: fade-up 550ms var(--ease-out) both;
    animation-delay: 60ms;
  }

  .stat {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 11px;
    padding: 12px 14px;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 15px;
    background: rgba(255, 255, 255, 0.018);
  }

  .stat-icon {
    display: grid;
    width: 32px;
    height: 32px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.045);
  }

  .stat-copy {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .stat-copy strong {
    font-size: 17px;
    font-weight: 650;
    letter-spacing: -0.04em;
    line-height: 1;
  }

  .stat-copy small {
    color: var(--muted);
    font-size: 9px;
  }

  .stat-note {
    justify-content: flex-end;
    color: rgba(255, 255, 255, 0.36);
    font-size: 9.5px;
    line-height: 1.35;
    text-align: right;
  }

  .status-line {
    width: 26px;
    height: 1px;
    flex: 0 0 auto;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.62));
  }

  .category {
    margin-top: 34px;
    animation: fade-up 520ms var(--ease-out) both;
    animation-delay: calc(90ms + var(--ci) * 70ms);
  }

  .category-head {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 0 4px 14px;
  }

  .category-icon {
    display: grid;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 13px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.035);
    box-shadow: 0 0 22px rgba(255, 255, 255, 0.03);
  }

  .category-copy {
    min-width: 0;
  }

  .category-title {
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .category-title h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 620;
    letter-spacing: -0.025em;
  }

  .category-count {
    display: grid;
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 999px;
    color: rgba(255, 255, 255, 0.66);
    background: rgba(255, 255, 255, 0.035);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8.5px;
  }

  .category-copy p {
    margin: 3px 0 0;
    color: rgba(255, 255, 255, 0.36);
    font-size: 10.5px;
  }

  .category-line {
    height: 1px;
    flex: 1;
    margin-left: 8px;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.16), transparent);
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(255px, 1fr));
    gap: 14px;
  }

  .state {
    display: flex;
    align-items: center;
    gap: 16px;
    margin: 30px 4px;
    padding: 20px 22px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.02);
    animation: fade-up 400ms var(--ease-out) both;
  }

  .state-error {
    border-color: rgba(255, 107, 107, 0.24);
    background: rgba(255, 107, 107, 0.04);
  }

  .state-icon {
    display: grid;
    width: 46px;
    height: 46px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 14px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.05);
  }

  .state-error .state-icon {
    color: var(--danger);
    background: rgba(255, 107, 107, 0.08);
  }

  .state strong {
    font-size: 13px;
    font-weight: 600;
  }

  .state p {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 10.5px;
    line-height: 1.5;
  }

  .state-action {
    display: inline-flex;
    height: 36px;
    align-items: center;
    gap: 7px;
    margin-left: auto;
    padding: 0 13px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.045);
    font-size: 10.5px;
    font-weight: 600;
    white-space: nowrap;
  }

  .state-action:hover {
    border-color: rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.08);
  }

  .loading-block {
    margin-top: 10px;
  }

  .loading-head {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 0 4px 14px;
    color: var(--muted);
    font-size: 10.5px;
  }

  .loader {
    width: 14px;
    height: 14px;
    border: 1.5px solid rgba(255, 255, 255, 0.16);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.85s linear infinite;
  }

  .skeleton-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(255px, 1fr));
    gap: 14px;
  }

  .skeleton-card {
    position: relative;
    display: flex;
    min-height: 200px;
    flex-direction: column;
    gap: 10px;
    padding: 18px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 22px;
    background: rgba(255, 255, 255, 0.02);
    animation: fade-up 400ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 35ms);
  }

  .skeleton-card::after {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      100deg,
      transparent 20%,
      rgba(255, 255, 255, 0.035) 48%,
      transparent 72%
    );
    content: "";
    animation: shimmer 1.8s ease-in-out infinite;
  }

  .skeleton-icon {
    width: 48px;
    height: 48px;
    border-radius: 15px;
    background: rgba(255, 255, 255, 0.045);
  }

  .skeleton-line {
    height: 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.045);
  }

  .skeleton-line:first-of-type {
    margin-top: 8px;
    height: 14px;
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

  @media (max-width: 1120px) {
    .page-head {
      flex-direction: column;
      align-items: stretch;
    }

    .search-shell {
      width: 100%;
      max-width: 420px;
    }

    .stats {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .stat-note {
      display: none;
    }
  }
  .hero-art {
    flex: none;
    width: 206px;
    height: 116px;
    object-fit: cover;
    border: 1px solid rgba(255, 255, 255, 0.14);
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
</style>
