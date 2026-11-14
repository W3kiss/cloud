<script lang="ts">
  import Icon from "./Icon.svelte";
  import { t, LANGS } from "$lib/i18n.svelte";
  import { THEMES, settings, saveSettings } from "$lib/settings.svelte";

  let {
    onAccept,
    onExit,
    leaving = false,
    skipTerms = false,
  }: { onAccept: () => void; onExit: () => void; leaving?: boolean; skipTerms?: boolean } = $props();

  let step = $state(0);

  function pickLang(id: (typeof LANGS)[number]["id"]) {
    settings.lang = id;
    saveSettings();
  }

  function pickTheme(id: (typeof THEMES)[number]["id"]) {
    settings.theme = id;
    saveSettings();
  }
</script>

<div
  class="welcome-layer"
  class:leaving
  role="dialog"
  aria-modal="true"
  aria-labelledby="welcome-title"
>
  <div class="welcome-backdrop"></div>

  <div class="welcome-card">
    <div class="orb orb-one"></div>
    <div class="orb orb-two"></div>

    <div class="welcome-top">
      <div class="welcome-icon">
        <Icon name="shield" size={30} stroke={1.6} />
      </div>
      <div class="welcome-meta">
        <span class="eyebrow">Setup Hub</span>
        <span class="meta-line"></span>
        <span class="meta-note">{t("welcome.step1.eyebrow")}</span>
      </div>
    </div>

    {#if step === 0}
      <h1 id="welcome-title">{t("welcome.hello")}</h1>
      <p class="disclaimer">{t("welcome.sub")}</p>

      <h2 class="pick-title">{t("welcome.lang")}</h2>
      <div class="pick-grid langs">
        {#each LANGS as lang (lang.id)}
          <button
            type="button"
            class="pick"
            class:active={settings.lang === lang.id}
            onclick={() => pickLang(lang.id)}
          >
            <b>{lang.flag}</b>
            <span>{lang.native}</span>
          </button>
        {/each}
      </div>

      <h2 class="pick-title">{t("welcome.theme")}</h2>
      <div class="pick-grid themes">
        {#each THEMES as theme (theme.id)}
          <button
            type="button"
            class="pick theme"
            class:active={settings.theme === theme.id}
            style={`--sw-bg:${theme.bg}; --sw-accent:${theme.accent}; --sw-panel:rgb(${theme.panel});`}
            onclick={() => pickTheme(theme.id)}
          >
            <i class="swatch"></i>
            <span>{t(`theme.${theme.id}`)}</span>
          </button>
        {/each}
      </div>

      <div class="welcome-actions">
        <button type="button" class="btn-exit" onclick={onExit}>
          <Icon name="power" size={17} />
          {t("welcome.exit")}
        </button>
        <button type="button" class="btn-agree" onclick={() => (skipTerms ? onAccept() : (step = 1))}>
          <span class="agree-shine"></span>
          {t("welcome.next")}
          <Icon name="arrow" size={17} stroke={2.2} />
        </button>
      </div>
    {:else}
      <h1 id="welcome-title">{t("welcome.title")}</h1>

      <p class="disclaimer">{t("welcome.disclaimer")}</p>

      <div class="notice">
        <Icon name="info" size={16} />
        <span>{t("welcome.notice")}</span>
      </div>

      <div class="welcome-actions">
        <button type="button" class="btn-exit" onclick={() => (step = 0)}>
          {t("welcome.back")}
        </button>
        <button type="button" class="btn-agree" onclick={onAccept}>
          <span class="agree-shine"></span>
          <Icon name="check" size={18} stroke={2.2} />
          {t("welcome.agree")}
        </button>
      </div>

      <p class="fineprint">{t("welcome.fine")}</p>
    {/if}
  </div>
</div>

<style>
  .welcome-layer {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: grid;
    padding: 28px;
    place-items: center;
    transition:
      opacity 260ms ease,
      visibility 260ms ease;
  }

  .welcome-layer.leaving {
    visibility: hidden;
    opacity: 0;
  }

  .welcome-backdrop {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(circle at 50% 38%, rgb(var(--fg) / 0.045), transparent 34%),
      radial-gradient(circle at 18% 75%, rgb(var(--fg) / 0.03), transparent 30%),
      rgb(var(--panel) / 0.7);
    backdrop-filter: blur(24px) saturate(105%);
    -webkit-backdrop-filter: blur(24px) saturate(105%);
    animation: fade-in 220ms ease both;
  }

  .welcome-card {
    position: relative;
    width: min(680px, 100%);
    max-height: calc(100vh - 56px);
    padding: 34px 36px 28px;
    overflow: auto;
    border: 1px solid rgb(var(--fg) / 0.12);
    border-radius: 30px;
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.07), transparent 42%),
      linear-gradient(180deg, rgb(var(--panel) / 0.96), rgb(var(--panel) / 0.98));
    box-shadow:
      0 40px 120px rgba(0, 0, 0, 0.66),
      0 0 0 1px rgb(var(--fg) / 0.05),
      inset 0 1px 0 rgb(var(--fg) / 0.12);
    animation: scale-in 420ms var(--ease-out) both;
  }

  .welcome-card::before {
    position: absolute;
    inset: -1px;
    z-index: -1;
    border-radius: inherit;
    padding: 1px;
    background: linear-gradient(
      135deg,
      rgb(var(--fg) / 0.64),
      rgb(var(--fg) / 0.08) 32%,
      rgb(var(--fg) / 0.04) 68%,
      rgb(var(--fg) / 0.28)
    );
    content: "";
    -webkit-mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }

  .orb {
    position: absolute;
    border-radius: 50%;
    filter: blur(70px);
    pointer-events: none;
  }

  .orb-one {
    top: -60px;
    right: -30px;
    width: 210px;
    height: 210px;
    background: rgb(var(--fg) / 0.075);
  }

  .orb-two {
    bottom: -100px;
    left: -40px;
    width: 240px;
    height: 240px;
    background: rgb(var(--fg) / 0.045);
  }

  .welcome-top {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 20px;
  }

  .welcome-icon {
    display: grid;
    width: 58px;
    height: 58px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.28);
    border-radius: 18px;
    color: var(--accent);
    background:
      radial-gradient(circle at 30% 20%, rgb(var(--fg) / 0.13), transparent 42%),
      rgb(var(--fg) / 0.045);
    box-shadow:
      0 0 34px rgb(var(--fg) / 0.09),
      inset 0 1px 0 rgb(var(--fg) / 0.12);
  }

  .welcome-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .eyebrow {
    color: var(--accent);
    font-size: 10px;
    font-weight: 650;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  .meta-line {
    width: 32px;
    height: 1px;
    background: rgb(var(--fg) / 0.16);
  }

  .meta-note {
    color: var(--muted);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h1 {
    position: relative;
    z-index: 1;
    margin: 0 0 18px;
    font-size: clamp(28px, 3.1vw, 40px);
    font-weight: 640;
    letter-spacing: -0.04em;
    line-height: 1.08;
  }

  .disclaimer {
    position: relative;
    z-index: 1;
    margin: 0;
    padding: 20px 22px;
    border: 1px solid rgb(var(--fg) / 0.075);
    border-radius: 18px;
    color: #c9ced6;
    background: rgb(var(--fg) / 0.025);
    font-size: 13px;
    line-height: 1.72;
  }

  .notice {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin-top: 14px;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.55;
  }

  .notice :global(svg) {
    flex: 0 0 auto;
    margin-top: 2px;
    color: rgb(var(--fg) / 0.7);
  }

  .welcome-actions {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 26px;
  }

  .btn-exit,
  .btn-agree {
    position: relative;
    display: inline-flex;
    height: 52px;
    align-items: center;
    justify-content: center;
    gap: 9px;
    overflow: hidden;
    border-radius: 15px;
    font-size: 13px;
    font-weight: 650;
    letter-spacing: 0.01em;
    transition:
      transform 220ms var(--ease-out),
      box-shadow 220ms ease,
      background 220ms ease,
      border-color 220ms ease;
  }

  .btn-exit {
    min-width: 150px;
    border: 1px solid rgba(255, 107, 107, 0.34);
    color: #ffb1b1;
    background: rgba(255, 107, 107, 0.09);
  }

  .btn-exit:hover {
    border-color: rgba(255, 107, 107, 0.65);
    background: rgba(255, 107, 107, 0.15);
    box-shadow: 0 0 28px rgba(255, 107, 107, 0.16);
  }

  .btn-agree {
    min-width: 190px;
    border: 1px solid rgb(var(--fg) / 0.74);
    color: #09090b;
    background:
      radial-gradient(circle at 32% 12%, #ffffff, transparent 42%),
      linear-gradient(145deg, #ffffff, #e4e4e7 55%, #bdbdc4);
    box-shadow:
      0 0 0 1px rgb(var(--fg) / 0.2),
      0 0 26px rgb(var(--fg) / 0.22),
      inset 0 1px 0 rgb(var(--fg) / 0.8);
    animation: breathe 3s ease-in-out infinite;
  }

  .btn-agree:hover {
    transform: translateY(-1px) scale(1.015);
    box-shadow:
      0 0 0 1px rgb(var(--fg) / 0.36),
      0 0 38px rgb(var(--fg) / 0.34),
      inset 0 1px 0 rgb(var(--fg) / 0.9);
  }

  .btn-exit:active,
  .btn-agree:active {
    transform: scale(0.98);
  }

  .agree-shine {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      105deg,
      transparent 20%,
      rgb(var(--fg) / 0.65) 46%,
      transparent 72%
    );
    transform: translateX(-110%);
    animation: shimmer 2.8s ease-in-out infinite;
    pointer-events: none;
  }

  .fineprint {
    position: relative;
    z-index: 1;
    margin: 18px 0 0;
    color: rgb(var(--fg) / 0.3);
    font-size: 10px;
    line-height: 1.55;
    text-align: center;
  }

  @media (max-width: 720px) {
    .welcome-card {
      padding: 26px 22px 22px;
    }

    .welcome-actions {
      flex-direction: column-reverse;
    }

    .btn-exit,
    .btn-agree {
      width: 100%;
    }
  }

  .pick-title {
    position: relative;
    z-index: 1;
    margin: 18px 0 8px;
    color: var(--muted-strong);
    font-size: 11px;
    font-weight: 650;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .pick-grid {
    position: relative;
    z-index: 1;
    display: grid;
    gap: 8px;
  }

  .pick-grid.langs { grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); }
  .pick-grid.themes { grid-template-columns: repeat(auto-fill, minmax(112px, 1fr)); }

  .pick {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px 11px;
    border: 1px solid rgb(var(--fg) / 0.1);
    border-radius: 12px;
    background: rgb(var(--fg) / 0.03);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    transition: border-color 160ms, background 160ms;
  }

  .pick b {
    display: grid;
    place-items: center;
    width: 28px;
    height: 20px;
    border-radius: 6px;
    background: rgb(var(--fg) / 0.08);
    font-size: 10px;
    letter-spacing: 0.08em;
  }

  .pick:hover { border-color: rgb(var(--fg) / 0.25); }
  .pick.active { border-color: rgb(var(--fg) / 0.7); background: rgb(var(--fg) / 0.09); }

  .swatch {
    width: 18px;
    height: 18px;
    flex: none;
    border-radius: 50%;
    border: 2px solid var(--sw-accent);
    background: var(--sw-bg);
    box-shadow: inset 0 0 0 3px var(--sw-panel);
  }
</style>
