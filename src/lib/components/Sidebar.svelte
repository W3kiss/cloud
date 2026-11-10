<script lang="ts">
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n.svelte";

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
    <span class="brand-status" title={t("nav.local")}></span>
  </div>

  <div class="nav-caption">{t("nav.caption")}</div>

  <nav class="nav" style={`--index:${activeIndex}`} aria-label={t("nav.aria")}>
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

  <button
    type="button"
    class="admin-card"
    class:elevated={admin}
    onclick={onElevate}
    title={admin ? t("admin.title.on") : t("admin.title.off")}
  >
    <span class="admin-orb"></span>
    <span class="admin-copy">
      <strong>{admin ? t("admin.on") : t("admin.off")}</strong>
      <small>{admin ? t("admin.on.hint") : t("admin.off.hint")}</small>
    </span>
    {#if !admin}
      <span class="admin-action"><Icon name="power" size={14} /></span>
    {/if}
  </button>

  <div class="version-row">
    <span>v1.0.0</span>
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
      linear-gradient(160deg, rgb(var(--fg) / 0.055), transparent 38%),
      rgb(var(--panel) / 0.72);
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.06),
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
    background: rgb(var(--fg) / 0.045);
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
    border: 1px solid rgb(var(--fg) / 0.68);
    border-radius: 15px;
    color: #0a0a0c;
    background:
      radial-gradient(circle at 30% 15%, #ffffff, transparent 40%),
      linear-gradient(145deg, #f4f4f5, #cfcfd4);
    box-shadow:
      0 0 28px rgb(var(--fg) / 0.14),
      inset 0 1px 0 rgb(var(--fg) / 0.8);
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
    box-shadow: 0 0 12px rgb(var(--fg) / 0.7);
    animation: pulse-dot 2.4s ease-in-out infinite;
  }

  .nav-caption {
    padding: 0 12px 10px;
    color: rgb(var(--fg) / 0.32);
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
    border: 1px solid rgb(var(--fg) / 0.04);
    border-radius: 20px;
    background: rgba(0, 0, 0, 0.14);
    --item-h: 58px;
    flex: 0 1 auto;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
  }

  .glider {
    position: absolute;
    top: calc(6px + var(--index) * (var(--item-h) + 6px));
    right: 6px;
    left: 6px;
    height: var(--item-h);
    border: 1px solid rgb(var(--fg) / 0.16);
    border-radius: 16px;
    background:
      linear-gradient(100deg, rgb(var(--fg) / 0.1), rgb(var(--fg) / 0.035) 62%, transparent),
      rgb(var(--fg) / 0.025);
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.1),
      0 0 26px rgb(var(--fg) / 0.035);
    transition: top 380ms var(--ease-out);
    pointer-events: none;
  }

  .nav-item {
    position: relative;
    z-index: 1;
    display: flex;
    height: var(--item-h);
    flex-shrink: 0;
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
    background: rgb(var(--fg) / 0.035);
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
    border: 1px solid rgb(var(--fg) / 0.06);
    border-radius: 11px;
    color: var(--muted-strong);
    background: rgb(var(--fg) / 0.035);
    transition:
      color 220ms ease,
      border-color 220ms ease,
      background 220ms ease;
  }

  .nav-item.active .nav-icon {
    border-color: rgb(var(--fg) / 0.26);
    color: var(--accent);
    background: rgb(var(--fg) / 0.08);
    box-shadow: 0 0 18px rgb(var(--fg) / 0.06);
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
    color: rgb(var(--fg) / 0.34);
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color 220ms ease;
  }

  .nav-item.active .nav-copy small {
    color: rgb(var(--fg) / 0.54);
  }

  .nav-arrow {
    display: flex;
    margin-left: auto;
    color: rgb(var(--fg) / 0.2);
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
    flex: 1 0 0;
    min-height: 8px;
  }

  .admin-card,
  .version-row {
    flex-shrink: 0;
  }

  @media (max-height: 820px) {
    .sidebar { padding: 14px 12px 12px; }
    .brand { padding-bottom: 12px; }
    .nav { --item-h: 46px; }
    .nav-item small { display: none; }
  }

  @media (max-height: 640px) {
    .nav { --item-h: 40px; }
    .nav-caption { display: none; }
  }

  .insight-card {
    position: relative;
    margin: 0 4px 12px;
    padding: 14px;
    overflow: hidden;
    border: 1px solid rgb(var(--fg) / 0.06);
    border-radius: 18px;
    background:
      linear-gradient(150deg, rgb(var(--fg) / 0.055), transparent 55%),
      rgb(var(--fg) / 0.022);
  }

  .insight-head {
    display: flex;
    align-items: center;
    gap: 7px;
    color: rgb(var(--fg) / 0.7);
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
    background: rgb(var(--fg) / 0.07);
  }

  .insight-line span {
    display: block;
    width: 64%;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    box-shadow: 0 0 10px rgb(var(--fg) / 0.24);
  }

  .admin-card {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 4px;
    padding: 11px 12px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 16px;
    text-align: left;
    background: rgb(var(--fg) / 0.028);
    transition:
      border-color 220ms ease,
      background 220ms ease,
      transform 220ms var(--ease-out);
  }

  .admin-card:hover {
    border-color: rgb(var(--fg) / 0.22);
    background: rgb(var(--fg) / 0.055);
  }

  .admin-card:active {
    transform: scale(0.985);
  }

  .admin-card.elevated {
    border-color: rgb(var(--fg) / 0.24);
    background: rgb(var(--fg) / 0.055);
  }

  .admin-orb {
    width: 9px;
    height: 9px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: rgb(var(--fg) / 0.4);
    box-shadow: 0 0 12px rgb(var(--fg) / 0.32);
  }

  .admin-card.elevated .admin-orb {
    background: var(--accent);
    box-shadow: 0 0 12px rgb(var(--fg) / 0.6);
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
    background: rgb(var(--fg) / 0.08);
  }

  .version-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px 4px 2px;
    color: rgb(var(--fg) / 0.26);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgb(var(--fg) / 0.24);
  }
</style>
