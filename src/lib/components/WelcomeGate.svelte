<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    onAccept,
    onExit,
    leaving = false,
  }: { onAccept: () => void; onExit: () => void; leaving?: boolean } = $props();

  const disclaimer =
    "Это приложение является проектом с открытым исходным кодом (Open-Source). Данный инструмент предназначен исключительно для автоматизации рутинных задач после переустановки операционной системы. Автор не несет ответственности за действия пользователя и используемые им ссылки на сторонние ресурсы. Если вы не хотите рисковать или не согласны с данными условиями, пожалуйста, закройте приложение и прекратите его использование.";
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
        <span class="eyebrow">Setup Hub / Open Source</span>
        <span class="meta-line"></span>
        <span class="meta-note">Первый запуск</span>
      </div>
    </div>

    <h1 id="welcome-title">Перед началом работы</h1>

    <p class="disclaimer">{disclaimer}</p>

    <div class="notice">
      <Icon name="info" size={16} />
      <span>
        Все операции выполняются локально. Приложение не отправляет данные
        на сторонние серверы, кроме загрузки выбранных вами установщиков.
      </span>
    </div>

    <div class="welcome-actions">
      <button type="button" class="btn-exit" onclick={onExit}>
        <Icon name="power" size={17} />
        Выйти
      </button>
      <button type="button" class="btn-agree" onclick={onAccept}>
        <span class="agree-shine"></span>
        <Icon name="check" size={18} stroke={2.2} />
        Я согласен
      </button>
    </div>

    <p class="fineprint">
      Нажимая «Я согласен», вы подтверждаете, что прочитали и принимаете
      данные условия. Решение сохраняется локально и не передаётся третьим
      лицам.
    </p>
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
      radial-gradient(circle at 50% 38%, rgba(255, 255, 255, 0.045), transparent 34%),
      radial-gradient(circle at 18% 75%, rgba(255, 255, 255, 0.03), transparent 30%),
      rgba(4, 4, 6, 0.7);
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
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 30px;
    background:
      linear-gradient(155deg, rgba(255, 255, 255, 0.07), transparent 42%),
      linear-gradient(180deg, rgba(17, 17, 20, 0.96), rgba(9, 9, 12, 0.98));
    box-shadow:
      0 40px 120px rgba(0, 0, 0, 0.66),
      0 0 0 1px rgba(255, 255, 255, 0.05),
      inset 0 1px 0 rgba(255, 255, 255, 0.12);
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
      rgba(255, 255, 255, 0.64),
      rgba(255, 255, 255, 0.08) 32%,
      rgba(255, 255, 255, 0.04) 68%,
      rgba(255, 255, 255, 0.28)
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
    background: rgba(255, 255, 255, 0.075);
  }

  .orb-two {
    bottom: -100px;
    left: -40px;
    width: 240px;
    height: 240px;
    background: rgba(255, 255, 255, 0.045);
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
    border: 1px solid rgba(255, 255, 255, 0.28);
    border-radius: 18px;
    color: var(--accent);
    background:
      radial-gradient(circle at 30% 20%, rgba(255, 255, 255, 0.13), transparent 42%),
      rgba(255, 255, 255, 0.045);
    box-shadow:
      0 0 34px rgba(255, 255, 255, 0.09),
      inset 0 1px 0 rgba(255, 255, 255, 0.12);
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
    background: rgba(255, 255, 255, 0.16);
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
    border: 1px solid rgba(255, 255, 255, 0.075);
    border-radius: 18px;
    color: #c9ced6;
    background: rgba(255, 255, 255, 0.025);
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
    color: rgba(255, 255, 255, 0.7);
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
    border: 1px solid rgba(255, 255, 255, 0.74);
    color: #09090b;
    background:
      radial-gradient(circle at 32% 12%, #ffffff, transparent 42%),
      linear-gradient(145deg, #ffffff, #e4e4e7 55%, #bdbdc4);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.2),
      0 0 26px rgba(255, 255, 255, 0.22),
      inset 0 1px 0 rgba(255, 255, 255, 0.8);
    animation: breathe 3s ease-in-out infinite;
  }

  .btn-agree:hover {
    transform: translateY(-1px) scale(1.015);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.36),
      0 0 38px rgba(255, 255, 255, 0.34),
      inset 0 1px 0 rgba(255, 255, 255, 0.9);
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
      rgba(255, 255, 255, 0.65) 46%,
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
    color: rgba(255, 255, 255, 0.3);
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
</style>
