<script lang="ts">
  import Icon from "./Icon.svelte";
</script>

<div class="tweaks-page">
  <header class="page-head">
    <div>
      <p class="eyebrow">
        <span class="live-dot"></span>
        Credits / Authors
      </p>
      <h1>Авторы <span>Setup Hub</span></h1>
      <p class="lead">Люди, которые создали проект и работают над его развитием.</p>
    </div>
    <img class="hero-art" src="/art/hero-authors.svg" alt="" />
  </header>

  <div class="tweak-grid">
    <article class="tweak-card enabled" style="--accent:#22d3ee; --i:0;">
      <div class="tweak-banner"><img src="/art/hero-authors.svg" alt="" /></div>
      <div class="tweak-aura"></div>
      <header class="tweak-head">
        <span class="tweak-icon"><Icon name="user" size={20} /></span>
        <div class="tweak-title">
          <h2>w3kiss</h2>
          <span class="tweak-note">AUTHOR</span>
        </div>
      </header>
      <p class="tweak-description">Автор и разработчик Setup Hub.</p>
      <footer class="tweak-foot">
        <span class="state-pill on">
          <span class="state-dot"></span>
          Автор проекта
        </span>
      </footer>
    </article>
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

  @media (max-width: 980px) {
    .tweak-grid {
      grid-template-columns: 1fr;
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
  .tweak-banner {
    position: relative;
    height: 78px;
    margin: -19px -19px 16px;
    overflow: hidden;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
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
