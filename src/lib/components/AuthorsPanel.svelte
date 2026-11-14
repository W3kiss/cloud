<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n.svelte";

  let avatarFailed = $state(false);
  let letter = $state(false);
  function onAvatarError() {
    avatarFailed = true;
  }

  const LINKS = $derived([
    { label: t("authors.github"), url: "https://github.com/W3kiss", icon: "user", primary: true },
    { label: t("authors.source"), url: "https://github.com/W3kiss/setup_hub", icon: "terminal", primary: false },
    { label: t("authors.releases"), url: "https://github.com/W3kiss/setup_hub/releases", icon: "download", primary: false },
  ]);

  const STATS = $derived([
    { value: "12", label: t("authors.s1") },
    { value: "40", label: t("authors.s2") },
    { value: "5", label: t("authors.s3") },
    { value: "1.3", label: t("authors.s4") },
  ]);

  const STACK = ["Tauri 2", "Rust", "Svelte 5", "TypeScript", "winget", "PowerShell"];

  const PRINCIPLES = $derived([
    { icon: "shield", title: t("authors.p1"), text: t("authors.p1.t") },
    { icon: "rocket", title: t("authors.p2"), text: t("authors.p2.t") },
    { icon: "sliders", title: t("authors.p3"), text: t("authors.p3.t") },
  ]);

  async function go(url: string) {
    try {
      await openUrl(url);
    } catch {
      // Opening a browser can fail in restricted environments; nothing else to do.
    }
  }
</script>

<div class="au">
  <section class="hero">
    <div class="avatar-wrap">
      <div class="halo" aria-hidden="true">
        <i></i><i></i><i></i>
      </div>

      <div class="avatar">
        {#if !avatarFailed}
          <img
            src="https://github.com/W3kiss.png?size=256"
            alt="w3kiss"
            draggable="false"
            onerror={onAvatarError}
          />
        {:else}
          <img src="/avatar.jpg" alt="w3kiss" draggable="false" onerror={() => (letter = true)} />
        {/if}
        {#if letter}<span>W</span>{/if}
      </div>
    </div>

    <p class="role"><span class="dash"></span>{t("authors.role")}<span class="dash"></span></p>
    <h1>w3kiss</h1>
    <p class="tagline">
      {t("authors.tagline")}
    </p>

    <div class="links">
      {#each LINKS as link (link.url)}
        <button type="button" class="link" class:primary={link.primary} onclick={() => go(link.url)}>
          <Icon name={link.icon} size={16} />
          {link.label}
          <Icon name="external" size={13} />
        </button>
      {/each}
    </div>
  </section>

  <section class="stats" aria-label={t("authors.stats")}>
    {#each STATS as stat, i (stat.label)}
      <div class="stat" style="--i:{i};">
        <strong>{stat.value}</strong>
        <span>{stat.label}</span>
      </div>
    {/each}
  </section>

  <section class="principles">
    {#each PRINCIPLES as item, i (item.title)}
      <article class="principle" style="--i:{i};">
        <span class="p-icon"><Icon name={item.icon} size={20} /></span>
        <h3>{item.title}</h3>
        <p>{item.text}</p>
      </article>
    {/each}
  </section>

  <section class="stack">
    <p class="stack-title">{t("authors.tech")}</p>
    <div class="chips">
      {#each STACK as tech (tech)}<span>{tech}</span>{/each}
    </div>
  </section>

  <p class="foot">{t("authors.thanks")}</p>
</div>

<style>
  .au {
    display: grid;
    gap: 26px;
    padding: 6px 4px 48px;
    justify-items: center;
  }

  .hero {
    position: relative;
    width: 100%;
    max-width: 860px;
    padding: 56px 24px 44px;
    display: grid;
    justify-items: center;
    text-align: center;
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    background:
      radial-gradient(circle at 50% 0%, rgb(var(--fg) / 0.07), transparent 60%),
      var(--surface);
    overflow: hidden;
    animation: fade-up 600ms var(--ease-out) both;
  }

  .avatar-wrap {
    position: relative;
    display: grid;
    place-items: center;
    width: 104px;
    height: 104px;
    animation: float 6s ease-in-out infinite;
  }

  .halo {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 0;
    height: 0;
    pointer-events: none;
  }

  .halo i {
    position: absolute;
    border-radius: 50%;
    border: 1px solid rgb(var(--fg) / 0.12);
    transform: translate(-50%, -50%);
    animation: pulse-ring 5.5s ease-out infinite;
  }

  .halo i:nth-child(1) { width: 150px; height: 150px; }
  .halo i:nth-child(2) { width: 220px; height: 220px; animation-delay: 1.8s; }
  .halo i:nth-child(3) { width: 300px; height: 300px; animation-delay: 3.6s; }

  @keyframes pulse-ring {
    0% { opacity: 0; transform: translate(-50%, -50%) scale(0.8); }
    25% { opacity: 1; }
    100% { opacity: 0; transform: translate(-50%, -50%) scale(1.15); }
  }

  .avatar {
    position: relative;
    z-index: 1;
    display: grid;
    place-items: center;
    width: 104px;
    height: 104px;
    border-radius: 50%;
    background: linear-gradient(145deg, rgb(var(--fg) / 0.16), rgb(var(--fg) / 0.03));
    overflow: hidden;
    border: 1px solid var(--border-strong);
    box-shadow:
      0 0 0 8px rgb(var(--fg) / 0.04),
      0 18px 50px -12px rgb(0 0 0 / 0.55);
  }

  .avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar span {
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.04em;
    color: var(--text);
  }

  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
  }

  .role {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 26px 0 8px;
    font-size: 11px;
    letter-spacing: 0.26em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .dash {
    width: 26px;
    height: 1px;
    background: var(--border-strong);
  }

  h1 {
    margin: 0;
    font-size: clamp(40px, 6vw, 64px);
    font-weight: 700;
    letter-spacing: -0.05em;
    line-height: 1;
  }

  .tagline {
    max-width: 520px;
    margin: 16px 0 0;
    color: var(--muted-strong);
    line-height: 1.6;
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    margin-top: 28px;
  }

  .link {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    padding: 11px 18px;
    font-size: 13px;
    border-radius: 99px;
    border: 1px solid var(--border-strong);
    background: var(--surface-strong);
    transition: transform 180ms var(--ease-out), background 180ms, border-color 180ms;
  }

  .link:hover {
    transform: translateY(-2px);
    border-color: var(--text);
  }

  .link.primary {
    background: var(--text);
    color: var(--bg);
    border-color: var(--text);
    font-weight: 600;
  }

  .stats {
    width: 100%;
    max-width: 860px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .stat {
    display: grid;
    justify-items: center;
    gap: 4px;
    padding: 20px 12px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    background: var(--surface);
    animation: fade-up 500ms var(--ease-out) both;
    animation-delay: calc(120ms + var(--i) * 70ms);
  }

  .stat strong {
    font-size: 34px;
    letter-spacing: -0.04em;
  }

  .stat span {
    font-size: 12px;
    color: var(--muted);
  }

  .principles {
    width: 100%;
    max-width: 860px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
    gap: 12px;
  }

  .principle {
    padding: 22px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    background: var(--surface);
    animation: fade-up 500ms var(--ease-out) both;
    animation-delay: calc(300ms + var(--i) * 80ms);
  }

  .p-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin-bottom: 14px;
    border-radius: 12px;
    border: 1px solid var(--border-strong);
    background: var(--surface-strong);
  }

  .principle h3 {
    margin: 0 0 6px;
    font-size: 15px;
  }

  .principle p {
    margin: 0;
    font-size: 13px;
    line-height: 1.55;
    color: var(--muted);
  }

  .stack {
    display: grid;
    justify-items: center;
    gap: 12px;
  }

  .stack-title {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0.26em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    max-width: 640px;
  }

  .chips span {
    padding: 7px 14px;
    font-size: 12px;
    color: var(--muted-strong);
    border: 1px solid var(--border);
    border-radius: 99px;
    background: var(--surface);
  }

  .foot {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--muted);
  }
</style>
