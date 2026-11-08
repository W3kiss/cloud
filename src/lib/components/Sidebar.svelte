<script lang="ts">
  import Icon from "./Icon.svelte";

  export type NavItem = {
    id: string;
    label: string;
    hint: string;
    icon: string;
  };

  let {
    active,
    items,
    admin = false,
    onNavigate,
    onElevate,
  }: {
    active: string;
    items: NavItem[];
    admin?: boolean;
    onNavigate: (id: string) => void;
    onElevate: () => void;
  } = $props();

  let activeIndex = $derived(
    Math.max(
      0,
      items.findIndex((item) => item.id === active),
    ),
  );
</script>

<aside class="sidebar">
  <div class="brand">
    <div class="brand-mark">
      <Icon name="logo" size={24} stroke={1.7} />
    </div>
    <div class="brand-copy">
      <strong>Setup Hub</strong>
      <span>launcher workspace</span>
    </div>
    <span class="brand-status" title="Локальный режим"></span>
  </div>

  <div class="nav-caption">Рабочее пространство</div>

  <nav class="nav" style={`--index:${activeIndex}`} aria-label="Основная навигация">
    <span class="glider" aria-hidden="true"></span>
    {#each items as item (item.id)}
      <button
        type="button"
        class="nav-item"
        class:active={item.id === active}
        onclick={() => onNavigate(item.id)}
      >
        <span class="nav-icon">
          <Icon name={item.icon} size={19} />
        </span>
        <span class="nav-copy">
          <strong>{item.label}</strong>
          <small>{item.hint}</small>
        </span>
        <span class="nav-arrow">
          <Icon name="arrow" size={14} />
        </span>
      </button>
    {/each}
  </nav>

  <div class="sidebar-spacer"></div>

  <div class="insight-card">
    <div class="insight-head">
      <Icon name="spark" size={15} />
      <span>Локальная сборка</span>
    </div>
    <p>Каталог и ссылки защищены на этапе компиляции.</p>
    <div class="insight-line"><span></span></div>
  </div>

  <button
    type="button"
    class="admin-card"
    class:elevated={admin}
    onclick={onElevate}
    title={admin ? "Приложение запущено с правами администратора" : "Перезапустить с правами администратора"}
  >
    <span class="admin-orb"></span>
    <span class="admin-copy">
      <strong>{admin ? "Администратор" : "Ограниченный режим"}</strong>
      <small>{admin ? "Все твики доступны" : "Для части твиков нужен UAC"}</small>
    </span>
    {#if !admin}
      <span class="admin-action"><Icon name="power" size={14} /></span>
    {/if}
  </button>

  <div class="version-row">
    <span>v0.1.0</span>
    <span class="dot"></span>
    <span>Open Source</span>
  </div>
</aside>

<style>
  .sidebar {
    position: relative;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 20px 16px 16px;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    background:
      linear-gradient(160deg, rgba(255, 255, 255, 0.055), transparent 38%),
      rgba(10, 10, 13, 0.72);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.06),
      0 24px 70px rgba(0, 0, 0, 0.36);
    backdrop-filter: blur(30px) saturate(130%);
  }

  .sidebar::before {
    position: absolute;
    top: -80px;
    left: -60px;
    width: 220px;
    height: 220px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.045);
    filter: blur(70px);
    content: "";
    pointer-events: none;
  }

  .brand {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 6px 22px;
  }

  .brand-mark {
    display: grid;
    width: 46px;
    height: 46px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.68);
    border-radius: 15px;
    color: #0a0a0c;
    background:
      radial-gradient(circle at 30% 15%, #ffffff, transparent 40%),
      linear-gradient(145deg, #f4f4f5, #cfcfd4);
    box-shadow:
      0 0 28px rgba(255, 255, 255, 0.14),
      inset 0 1px 0 rgba(255, 255, 255, 0.8);
  }

  .brand-copy {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .brand-copy strong {
    font-size: 17px;
    font-weight: 650;
    letter-spacing: -0.025em;
  }

  .brand-copy span {
    color: var(--muted);
    font-size: 10px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
  }

  .brand-status {
    width: 7px;
    height: 7px;
    margin-left: auto;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.7);
    animation: pulse-dot 2.4s ease-in-out infinite;
  }

  .nav-caption {
    padding: 0 12px 10px;
    color: rgba(255, 255, 255, 0.32);
    font-size: 9px;
    font-weight: 650;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  .nav {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px;
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 20px;
    background: rgba(0, 0, 0, 0.14);
  }

  .glider {
    position: absolute;
    top: calc(6px + var(--index) * 64px);
    right: 6px;
    left: 6px;
    height: 58px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 16px;
    background:
      linear-gradient(100deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.035) 62%, transparent),
      rgba(255, 255, 255, 0.025);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      0 0 26px rgba(255, 255, 255, 0.035);
    transition: top 380ms var(--ease-out);
    pointer-events: none;
  }

  .nav-item {
    position: relative;
    z-index: 1;
    display: flex;
    height: 58px;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    border-radius: 16px;
    text-align: left;
    transition:
      background 220ms ease,
      transform 220ms var(--ease-out);
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.035);
  }

  .nav-item:active {
    transform: scale(0.985);
  }

  .nav-icon {
    display: grid;
    width: 34px;
    height: 34px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 11px;
    color: var(--muted-strong);
    background: rgba(255, 255, 255, 0.035);
    transition:
      color 220ms ease,
      border-color 220ms ease,
      background 220ms ease;
  }

  .nav-item.active .nav-icon {
    border-color: rgba(255, 255, 255, 0.26);
    color: var(--accent);
    background: rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 18px rgba(255, 255, 255, 0.06);
  }

  .nav-copy {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .nav-copy strong {
    overflow: hidden;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.01em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-copy small {
    overflow: hidden;
    color: rgba(255, 255, 255, 0.34);
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color 220ms ease;
  }

  .nav-item.active .nav-copy small {
    color: rgba(255, 255, 255, 0.54);
  }

  .nav-arrow {
    display: flex;
    margin-left: auto;
    color: rgba(255, 255, 255, 0.2);
    opacity: 0;
    transform: translateX(-4px);
    transition:
      opacity 220ms ease,
      transform 220ms var(--ease-out),
      color 220ms ease;
  }

  .nav-item.active .nav-arrow {
    color: var(--accent);
    opacity: 1;
    transform: translateX(0);
  }

  .sidebar-spacer {
    flex: 1;
    min-height: 18px;
  }

  .insight-card {
    position: relative;
    margin: 0 4px 12px;
    padding: 14px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 18px;
    background:
      linear-gradient(150deg, rgba(255, 255, 255, 0.055), transparent 55%),
      rgba(255, 255, 255, 0.022);
  }

  .insight-head {
    display: flex;
    align-items: center;
    gap: 7px;
    color: rgba(255, 255, 255, 0.7);
    font-size: 10px;
    font-weight: 650;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .insight-card p {
    margin: 9px 0 12px;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.5;
  }

  .insight-line {
    height: 3px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
  }

  .insight-line span {
    display: block;
    width: 64%;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.24);
  }

  .admin-card {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 4px;
    padding: 11px 12px;
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 16px;
    text-align: left;
    background: rgba(255, 255, 255, 0.028);
    transition:
      border-color 220ms ease,
      background 220ms ease,
      transform 220ms var(--ease-out);
  }

  .admin-card:hover {
    border-color: rgba(255, 255, 255, 0.22);
    background: rgba(255, 255, 255, 0.055);
  }

  .admin-card:active {
    transform: scale(0.985);
  }

  .admin-card.elevated {
    border-color: rgba(255, 255, 255, 0.24);
    background: rgba(255, 255, 255, 0.055);
  }

  .admin-orb {
    width: 9px;
    height: 9px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.4);
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.32);
  }

  .admin-card.elevated .admin-orb {
    background: var(--accent);
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.6);
  }

  .admin-copy {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .admin-copy strong {
    font-size: 11px;
    font-weight: 600;
  }

  .admin-copy small {
    overflow: hidden;
    color: var(--muted);
    font-size: 9px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .admin-action {
    display: grid;
    width: 26px;
    height: 26px;
    margin-left: auto;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 9px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.08);
  }

  .version-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px 4px 2px;
    color: rgba(255, 255, 255, 0.26);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.24);
  }
</style>
