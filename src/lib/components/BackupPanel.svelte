<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, humanizeError, type ToastTone } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  let source = $state("");
  let folderName = $state("");
  let scope = $state<"roaming" | "local">("roaming");
  let busy = $state(false);
  let picking = $state(false);
  let fieldError = $state("");

  async function pickFolder() {
    picking = true;
    fieldError = "";
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Выберите папку с резервной копией",
      });
      if (typeof selected === "string") {
        source = selected;
        if (!folderName) {
          const parts = selected.split(/[\\/]/).filter(Boolean);
          folderName = parts[parts.length - 1] ?? "";
        }
      }
    } catch (caught) {
      onNotify("error", "Не удалось открыть проводник", humanizeError(caught));
    } finally {
      picking = false;
    }
  }

  function validFolderName(name: string) {
    return (
      /^[A-Za-z0-9][A-Za-z0-9 ._-]{0,63}$/.test(name) &&
      !name.endsWith(".") &&
      !name.endsWith(" ")
    );
  }

  async function restore() {
    fieldError = "";

    if (!source) {
      fieldError = "Выберите папку-источник.";
      return;
    }
    if (!validFolderName(folderName)) {
      fieldError =
        "Имя папки: латинские буквы, цифры, пробелы, точки и дефисы. До 64 символов.";
      return;
    }

    busy = true;
    try {
      const message = await api.restoreBackup(source, scope, folderName);
      onNotify("success", "Папка восстановлена", message);
    } catch (caught) {
      onNotify("error", "Восстановление не удалось", humanizeError(caught));
    } finally {
      busy = false;
    }
  }
</script>

<div class="backup-page">
  <header class="page-head">
    <div>
      <p class="eyebrow">
        <span class="live-dot"></span>
        Recovery / AppData Restore
      </p>
      <h1>Верните <span>рабочие настройки</span></h1>
      <p class="lead">
        Скопируйте папку профиля приложения из резервной копии в Roaming или
        Local AppData. Исходные файлы не изменяются.
      </p>
    </div>
    <div class="head-badge">
      <Icon name="archive" size={16} />
      <span>Локальное восстановление</span>
    </div>
    <img class="hero-art" src="/art/hero-backup.svg" alt="" />
  </header>

  <div class="backup-layout">
    <section class="form-card">
      <div class="form-head">
        <span class="form-icon"><Icon name="folder" size={19} /></span>
        <div>
          <h2>Параметры восстановления</h2>
          <p>Выберите папку и укажите имя профиля назначения.</p>
        </div>
      </div>

      <label class="field">
        <span class="field-label">Папка-источник</span>
        <button
          type="button"
          class="folder-picker"
          class:filled={Boolean(source)}
          disabled={picking}
          onclick={pickFolder}
        >
          <Icon name={picking ? "refresh" : "folder"} size={16} />
          <span>{source || "Выбрать папку резервной копии..."}</span>
          <Icon name="arrow" size={14} />
        </button>
      </label>

      <label class="field">
        <span class="field-label">Имя папки в AppData</span>
        <input
          type="text"
          bind:value={folderName}
          spellcheck="false"
          placeholder="Например, Discord"
          maxlength="64"
        />
      </label>

      <div class="field">
        <span class="field-label">Область назначения</span>
        <div class="scope-switch" role="group" aria-label="Область AppData">
          <button
            type="button"
            class:active={scope === "roaming"}
            onclick={() => (scope = "roaming")}
          >
            <strong>Roaming</strong>
            <small>%APPDATA%</small>
          </button>
          <button
            type="button"
            class:active={scope === "local"}
            onclick={() => (scope = "local")}
          >
            <strong>Local</strong>
            <small>%LOCALAPPDATA%</small>
          </button>
        </div>
      </div>

      {#if fieldError}
        <p class="field-error">
          <Icon name="alert" size={13} />
          {fieldError}
        </p>
      {/if}

      <button type="button" class="restore" disabled={busy} onclick={restore}>
        {#if busy}
          <span class="spinner"></span>
          Копирование...
        {:else}
          <Icon name="refresh" size={16} />
          Восстановить папку
        {/if}
      </button>
    </section>

    <aside class="info-card">
      <div class="info-head">
        <Icon name="info" size={15} />
        <span>Как это работает</span>
      </div>
      <ol>
        <li>
          <span>01</span>
          <p>Выберите папку с сохранённым профилем приложения.</p>
        </li>
        <li>
          <span>02</span>
          <p>Укажите имя папки, которая будет создана в AppData.</p>
        </li>
        <li>
          <span>03</span>
          <p>Приложение рекурсивно скопирует файлы и покажет результат.</p>
        </li>
      </ol>
      <div class="safety-note">
        <Icon name="shield" size={14} />
        <span>
          Системная папка Windows и пересекающиеся пути блокируются на уровне
          Rust-команды.
        </span>
      </div>
    </aside>
  </div>
</div>

<style>
  .backup-page {
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

  .head-badge {
    display: inline-flex;
    height: 40px;
    flex: 0 0 auto;
    align-items: center;
    gap: 7px;
    padding: 0 13px;
    border: 1px solid rgb(var(--fg) / 0.07);
    border-radius: 12px;
    color: var(--muted);
    background: rgb(var(--fg) / 0.025);
    font-size: 9.5px;
    white-space: nowrap;
  }

  .backup-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.5fr) minmax(280px, 0.8fr);
    gap: 16px;
    margin-top: 30px;
  }

  .form-card,
  .info-card {
    padding: 22px;
    border: 1px solid rgb(var(--fg) / 0.075);
    border-radius: 23px;
    background:
      linear-gradient(155deg, rgb(var(--fg) / 0.04), transparent 40%),
      linear-gradient(180deg, rgb(var(--panel) / 0.86), rgb(var(--panel) / 0.92));
    box-shadow:
      inset 0 1px 0 rgb(var(--fg) / 0.05),
      0 16px 38px rgba(0, 0, 0, 0.24);
    animation: fade-up 520ms var(--ease-out) both;
    animation-delay: 60ms;
  }

  .form-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding-bottom: 20px;
    border-bottom: 1px solid rgb(var(--fg) / 0.06);
  }

  .form-icon {
    display: grid;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid rgb(var(--fg) / 0.14);
    border-radius: 13px;
    color: var(--text);
    background: rgb(var(--fg) / 0.045);
  }

  .form-head h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 620;
    letter-spacing: -0.02em;
  }

  .form-head p {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin-top: 17px;
  }

  .field-label {
    color: rgb(var(--fg) / 0.42);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .folder-picker,
  .field input {
    display: flex;
    width: 100%;
    height: 44px;
    align-items: center;
    gap: 9px;
    padding: 0 12px;
    border: 1px solid rgb(var(--fg) / 0.09);
    border-radius: 12px;
    color: var(--muted);
    background: rgb(var(--panel) / 0.62);
    font-size: 11px;
    text-align: left;
    transition:
      border-color 200ms ease,
      box-shadow 200ms ease,
      background 200ms ease;
  }

  .field input {
    outline: 0;
  }

  .folder-picker:hover,
  .field input:focus {
    border-color: rgb(var(--fg) / 0.36);
    background: rgb(var(--panel) / 0.84);
    box-shadow: 0 0 0 3px rgb(var(--fg) / 0.045);
  }

  .folder-picker.filled {
    color: var(--text);
  }

  .folder-picker span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-picker :global(svg:last-child) {
    margin-left: auto;
    flex: 0 0 auto;
    opacity: 0.4;
  }

  .scope-switch {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .scope-switch button {
    display: flex;
    height: 56px;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    padding: 0 13px;
    border: 1px solid rgb(var(--fg) / 0.07);
    border-radius: 12px;
    text-align: left;
    background: rgb(var(--fg) / 0.02);
    transition:
      border-color 200ms ease,
      background 200ms ease,
      box-shadow 200ms ease;
  }

  .scope-switch button.active {
    border-color: rgb(var(--fg) / 0.38);
    background: rgb(var(--fg) / 0.06);
    box-shadow: 0 0 22px rgb(var(--fg) / 0.045);
  }

  .scope-switch strong {
    font-size: 11px;
    font-weight: 620;
  }

  .scope-switch small {
    color: var(--muted);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 8.5px;
  }

  .field-error {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 13px 0 0;
    color: #ff9d9d;
    font-size: 10px;
    line-height: 1.45;
  }

  .restore {
    display: inline-flex;
    width: 100%;
    height: 46px;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-top: 22px;
    border: 1px solid rgb(var(--fg) / 0.78);
    border-radius: 13px;
    color: #09090b;
    background: linear-gradient(145deg, #ffffff, #dcdce0);
    font-size: 11.5px;
    font-weight: 650;
    box-shadow:
      0 0 22px rgb(var(--fg) / 0.11),
      inset 0 1px 0 rgb(var(--fg) / 0.7);
    transition:
      transform 200ms var(--ease-out),
      box-shadow 200ms ease;
  }

  .restore:hover:not(:disabled) {
    box-shadow:
      0 0 32px rgb(var(--fg) / 0.2),
      inset 0 1px 0 rgb(var(--fg) / 0.85);
    transform: translateY(-1px);
  }

  .restore:disabled {
    opacity: 0.65;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 1.5px solid rgb(var(--panel) / 0.28);
    border-top-color: #09090b;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .info-card {
    align-self: start;
    animation-delay: 120ms;
  }

  .info-head {
    display: flex;
    align-items: center;
    gap: 7px;
    color: rgb(var(--fg) / 0.66);
    font-size: 9.5px;
    font-weight: 650;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .info-card ol {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin: 20px 0 0;
    padding: 0;
    list-style: none;
  }

  .info-card li {
    display: flex;
    gap: 11px;
  }

  .info-card li span {
    flex: 0 0 auto;
    color: rgb(var(--fg) / 0.46);
    font-family: "Cascadia Code", Consolas, monospace;
    font-size: 9px;
    font-weight: 600;
  }

  .info-card li p {
    margin: 0;
    color: var(--muted);
    font-size: 10.5px;
    line-height: 1.55;
  }

  .safety-note {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-top: 22px;
    padding: 12px;
    border: 1px solid rgb(var(--fg) / 0.06);
    border-radius: 12px;
    color: rgb(var(--fg) / 0.4);
    background: rgb(var(--fg) / 0.02);
    font-size: 9px;
    line-height: 1.5;
  }

  .safety-note :global(svg) {
    flex: 0 0 auto;
    margin-top: 1px;
    color: rgb(var(--fg) / 0.58);
  }

  @media (max-width: 980px) {
    .backup-layout {
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
</style>
