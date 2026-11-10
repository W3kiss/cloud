<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import ToggleSwitch from "./ToggleSwitch.svelte";
  import { t, LANGS } from "$lib/i18n.svelte";
  import { THEMES, resetSettings, saveSettings, settings } from "$lib/settings.svelte";
  import type { ToastTone } from "$lib/api";

  const APP_VERSION = "1.0.0";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  const effects = [
    {
      key: "animations" as const,
      title: "Анимации",
      description: "Плавные появления карточек и переходы. Отключите на слабых ПК.",
      icon: "spark",
    },
    {
      key: "blur" as const,
      title: "Эффект стекла",
      description: "Размытие фона панелей. Отключение снижает нагрузку на видеокарту.",
      icon: "window",
    },
    {
      key: "grid" as const,
      title: "Фоновая сетка",
      description: "Декоративная сетка на заднем плане приложения.",
      icon: "grid",
    },
    {
      key: "compact" as const,
      title: "Компактный режим",
      description: "Уменьшает высоту пунктов меню и расстояния между карточками.",
      icon: "sliders",
    },
  ];

  const links = [
    {
      title: "Аккаунт разработчика",
      note: "github.com/W3kiss",
      description: "Профиль автора w3kiss на GitHub: другие проекты и контакты.",
      url: "https://github.com/W3kiss",
      icon: "user",
      art: "hero-authors",
    },
    {
      title: "Репозиторий проекта",
      note: "W3kiss/cloud",
      description: "Исходный код Setup Hub, история изменений и свежие сборки.",
      url: "https://github.com/W3kiss/cloud",
      icon: "terminal",
      art: "hero-tweaks",
    },
    {
      title: "Сообщить о проблеме",
      note: "GitHub Issues",
      description: "Нашли ошибку или хотите новую функцию? Создайте обращение.",
      url: "https://github.com/W3kiss/cloud/issues",
      icon: "message",
      art: "hero-stripped",
    },
    {
      title: "Релизы и обновления",
      note: "GitHub Releases",
      description: "Скачайте новую версию приложения, когда она появится.",
      url: "https://github.com/W3kiss/cloud/releases",
      icon: "download",
      art: "hero-usb",
    },
  ];

  let confirmReset = $state(false);

  function setTheme(id: (typeof THEMES)[number]["id"]) {
    settings.theme = id;
    saveSettings();
  }

  function setLang(id: (typeof LANGS)[number]["id"]) {
    settings.lang = id;
    saveSettings();
  }

  function toggle(key: (typeof effects)[number]["key"], next: boolean) {
    settings[key] = next;
    saveSettings();
  }

  async function openLink(url: string, title: string) {
    try {
      await openUrl(url);
    } catch (caught) {
      onNotify("error", `Не удалось открыть: ${title}`, String(caught));
    }
  }

  function doReset() {
    resetSettings();
    confirmReset = false;
    onNotify("success", "Настройки сброшены", "Тема и параметры возвращены к значениям по умолчанию.");
  }
</script>

<div class="tweaks-page">
  <header class="page-head">
    <div>
      <p class="eyebrow">
        <span class="live-dot"></span>
        App / Settings
      </p>
      <h1>Настройки <span>Setup Hub</span></h1>
      <p class="lead">
        Оформление, эффекты и ссылки на проект. Изменения применяются сразу и
        сохраняются между запусками.
      </p>
    </div>
    <img class="hero-art" src="/art/hero-settings.svg" alt="" />
  </header>

  <section class="group">
    <div class="group-head">
      <h2>{t("settings.language")}</h2>
      <p>{t("settings.language.hint")}</p>
    </div>
    <div class="lang-grid">
      {#each LANGS as lang (lang.id)}
        <button
          type="button"
          class="lang-chip"
          class:active={settings.lang === lang.id}
          onclick={() => setLang(lang.id)}
        >
          <b>{lang.flag}</b>
          <span>{lang.native}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="group">
    <div class="group-head">
      <h2>Тема оформления</h2>
      <p>Выберите цветовой акцент интерфейса.</p>
    </div>
    <div class="theme-grid">
      {#each THEMES as theme, index (theme.id)}
        <button
          type="button"
          class="theme-card"
          class:active={settings.theme === theme.id}
          style={`--accent:${theme.accent}; --i:${index};`}
          onclick={() => setTheme(theme.id)}
        >
          <span
            class="theme-preview"
            class:star={theme.id === "star"}
            style={`--pv-bg:${theme.bg}; --pv-panel:rgb(${theme.panel}); --pv-line:rgb(${theme.fg} / 0.16); --pv-accent:${theme.accent};`}
          >
            <i class="pv-side"></i>
            <i class="pv-card"></i>
            <i class="pv-card short"></i>
            <i class="pv-accent"></i>
          </span>
          <span class="theme-meta">
            <strong>{t(`theme.${theme.id}`)}</strong>
            <small>{t(`theme.${theme.id}.hint`)}</small>
          </span>
          {#if settings.theme === theme.id}
            <span class="theme-check"><Icon name="check" size={13} stroke={2.4} /></span>
          {/if}
        </button>
      {/each}
    </div>
  </section>

  <section class="group">
    <div class="group-head">
      <h2>Эффекты и поведение</h2>
      <p>Тонкая настройка внешнего вида и производительности самого приложения.</p>
    </div>
    <div class="tweak-grid">
      {#each effects as item, index (item.key)}
        <article
          class="tweak-card"
          class:enabled={settings[item.key]}
          style={`--accent:#d4d4d8; --i:${index};`}
        >
          <div class="tweak-aura"></div>
          <header class="tweak-head">
            <span class="tweak-icon"><Icon name={item.icon} size={20} /></span>
            <div class="tweak-title">
              <h2>{item.title}</h2>
              <span class="tweak-note">{settings[item.key] ? "ВКЛЮЧЕНО" : "ВЫКЛЮЧЕНО"}</span>
            </div>
            <ToggleSwitch
              checked={settings[item.key]}
              label={item.title}
              onChange={(next) => toggle(item.key, next)}
            />
          </header>
          <p class="tweak-description">{item.description}</p>
        </article>
      {/each}
    </div>
  </section>

  <section class="group">
    <div class="group-head">
      <h2>Разработчик и проект</h2>
      <p>Ссылки откроются в браузере по умолчанию.</p>
    </div>
    <div class="tweak-grid">
      {#each links as link, index (link.url)}
        <article class="tweak-card" style={`--accent:#22d3ee; --i:${index};`}>
          <div class="tweak-banner"><img src={`/art/${link.art}.svg`} alt="" loading="lazy" /></div>
          <div class="tweak-aura"></div>
          <header class="tweak-head">
            <span class="tweak-icon"><Icon name={link.icon} size={20} /></span>
            <div class="tweak-title">
              <h2>{link.title}</h2>
              <span class="tweak-note">{link.note}</span>
            </div>
          </header>
          <p class="tweak-description">{link.description}</p>
          <footer class="tweak-foot">
            <button type="button" class="btn small left" onclick={() => openLink(link.url, link.title)}>
              <Icon name="external" size={13} />
              Открыть
            </button>
          </footer>
        </article>
      {/each}
    </div>
  </section>

  <section class="group">
    <div class="group-head">
      <h2>О программе</h2>
      <p>Setup Hub {APP_VERSION} · Tauri 2 + Svelte 5 · автор w3kiss</p>
    </div>
    <div class="reset-row">
      {#if confirmReset}
        <span>Сбросить тему и все параметры?</span>
        <div>
          <button type="button" class="btn small ghost" onclick={() => (confirmReset = false)}>Отмена</button>
          <button type="button" class="btn small danger" onclick={doReset}>Да, сбросить</button>
        </div>
      {:else}
        <span>Вернуть тему и параметры к значениям по умолчанию.</span>
        <button type="button" class="btn small" onclick={() => (confirmReset = true)}>
          <Icon name="refresh" size={13} />
          Сбросить настройки
        </button>
      {/if}
    </div>
  </section>
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

  .theme-preview {
    position: relative;
    display: block;
    height: 84px;
    border-radius: 12px;
    background: var(--pv-bg);
    border: 1px solid var(--pv-line);
    overflow: hidden;
  }

  .theme-preview.star {
    background-image:
      radial-gradient(circle at 12% 22%, #fff 0 1px, transparent 1.6px),
      radial-gradient(circle at 78% 14%, #fff 0 1.2px, transparent 1.8px),
      radial-gradient(circle at 55% 82%, #fff 0 1px, transparent 1.6px),
      radial-gradient(circle at 92% 70%, #fff 0 0.9px, transparent 1.5px),
      radial-gradient(circle at 30% 60%, #fff 0 0.8px, transparent 1.4px);
  }

  .theme-preview i {
    position: absolute;
    display: block;
    border-radius: 6px;
    background: var(--pv-panel);
    border: 1px solid var(--pv-line);
  }

  .pv-side { top: 8px; bottom: 8px; left: 8px; width: 26%; }
  .pv-card { top: 8px; left: 36%; right: 8px; height: 26px; }
  .pv-card.short { top: 40px; right: 40%; height: 34px; }
  .pv-accent { top: 44px; right: 10px; width: 22%; height: 8px; background: var(--pv-accent) !important; border: 0 !important; }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
    margin-top: 16px;
  }

  .theme-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px 8px 12px;
    text-align: left;
    border: 1px solid rgb(var(--fg) / 0.1);
    border-radius: 18px;
    background: rgb(var(--fg) / 0.03);
    transition: transform 0.25s var(--ease-out), border-color 0.25s ease;
    animation: fade-up 500ms var(--ease-out) both;
    animation-delay: calc(var(--i) * 50ms);
  }

  .theme-card:hover {
    transform: translateY(-3px);
    border-color: rgb(var(--fg) / 0.24);
  }

  .theme-card.active {
    border-color: var(--accent);
    box-shadow: 0 0 24px color-mix(in srgb, var(--accent) 22%, transparent);
  }

  .theme-card img {
    width: 100%;
    height: 82px;
    object-fit: cover;
    border-radius: 12px;
    pointer-events: none;
    -webkit-user-drag: none;
  }

  .theme-meta {
    display: grid;
    gap: 2px;
    padding: 0 4px;
  }

  .theme-meta strong {
    font-size: 12px;
    font-weight: 620;
  }

  .theme-meta small {
    color: var(--muted);
    font-size: 9.5px;
  }

  .theme-check {
    position: absolute;
    top: 14px;
    right: 14px;
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    color: #09090b;
    border-radius: 50%;
    background: var(--accent);
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 14px;
    color: var(--text);
    font-size: 11px;
    font-weight: 600;
    border: 1px solid rgb(var(--fg) / 0.18);
    border-radius: 11px;
    background: rgb(var(--fg) / 0.07);
    transition: background 0.2s ease;
  }

  .btn:hover {
    background: rgb(var(--fg) / 0.13);
  }

  .btn.small {
    height: 30px;
    padding: 0 12px;
    font-size: 10px;
  }

  .btn.ghost {
    background: transparent;
  }

  .btn.danger {
    border-color: rgba(255, 107, 107, 0.6);
    background: rgba(255, 107, 107, 0.22);
  }

  .reset-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 14px;
    padding: 14px 16px;
    color: var(--muted-strong);
    font-size: 11px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 16px;
    background: rgb(var(--fg) / 0.03);
  }

  .reset-row > div {
    display: flex;
    gap: 8px;
  }

  @media (max-width: 980px) {
    .hero-art {
      display: none;
    }
  }

  .lang-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 8px;
  }

  .lang-chip {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
    cursor: pointer;
    transition: border-color 160ms, background 160ms;
  }

  .lang-chip b {
    display: grid;
    place-items: center;
    width: 30px;
    height: 22px;
    border-radius: 6px;
    background: var(--surface-strong);
    font-size: 10px;
    letter-spacing: 0.08em;
  }

  .lang-chip:hover { border-color: var(--border-strong); }
  .lang-chip.active { border-color: var(--accent); background: var(--accent-soft); }
</style>
