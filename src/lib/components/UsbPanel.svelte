<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Icon from "./Icon.svelte";
  import { api, humanizeError, type ToastTone, type UsbDisk } from "$lib/api";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  interface Variant {
    id: string;
    name: string;
    badge: string;
    risk: "safe" | "mid" | "high";
    text: string;
    art: string;
  }

  const VARIANTS: Variant[] = [
    {
      id: "original",
      name: "Оригинальная",
      badge: "Без изменений",
      risk: "safe",
      text: "Чистый официальный образ Microsoft. Ничего не добавляется и не удаляется.",
      art: "/art/win-original.svg",
    },
    {
      id: "bypass",
      name: "Без проверок",
      badge: "Старые ПК",
      risk: "safe",
      text: "Оригинал + обход проверок TPM, Secure Boot, RAM и CPU при установке Windows 11.",
      art: "/art/win-bypass.svg",
    },
    {
      id: "lite",
      name: "Lite",
      badge: "Меньше мусора",
      risk: "mid",
      text: "После установки отключает телеметрию и рекламу, удаляет лишние предустановленные приложения.",
      art: "/art/win-lite.svg",
    },
    {
      id: "gaming",
      name: "Gaming",
      badge: "FPS",
      risk: "mid",
      text: "Lite + игровые твики: максимальная производительность, без Game DVR и троттлинга.",
      art: "/art/win-gaming.svg",
    },
    {
      id: "minimal",
      name: "Minimal",
      badge: "Бета · рискованно",
      risk: "high",
      text: "Gaming + отключение поиска, эффектов, OneDrive и резервирования места. Самая лёгкая система.",
      art: "/art/win-minimal.svg",
    },
  ];

  const PRODUCTS = [
    { id: "win11", label: "Windows 11" },
    { id: "win10", label: "Windows 10" },
  ];

  const LANGS = [
    { id: "ru", label: "Русский", fido: "Russian" },
    { id: "en", label: "English", fido: "English (United States)" },
    { id: "uk", label: "Українська", fido: "Ukrainian" },
    { id: "de", label: "Deutsch", fido: "German" },
  ];

  const MS_PAGES: Record<string, string> = {
    win11: "https://www.microsoft.com/software-download/windows11",
    win10: "https://www.microsoft.com/software-download/windows10",
  };

  const RISK_LABEL = { safe: "Безопасно", mid: "Умеренный риск", high: "Высокий риск" };

  let variant = $state("original");
  let source = $state<"auto" | "file">("auto");
  let product = $state("win11");
  let langId = $state("ru");
  let keepIso = $state(false);
  let scheme = $state<"gpt" | "mbr">("gpt");
  let isoPath = $state("");
  let disks = $state<UsbDisk[]>([]);
  let diskNumber = $state<number | null>(null);
  let loadingDisks = $state(false);
  let isAdmin = $state(true);
  let confirming = $state(false);
  let running = $state(false);
  let stage = $state<"" | "download" | "write">("");
  let percent = $state(0);
  let phase = $state("");
  let detail = $state("");
  let done = $state("");
  let failure = $state("");

  const selectedDisk = $derived(disks.find((d) => d.number === diskNumber) ?? null);
  const selectedVariant = $derived(VARIANTS.find((v) => v.id === variant) ?? VARIANTS[0]);
  const fidoLang = $derived(LANGS.find((l) => l.id === langId)?.fido ?? "Russian");
  const ready = $derived(
    (source === "auto" || isoPath !== "") && selectedDisk !== null && !running,
  );

  async function refreshDisks() {
    loadingDisks = true;
    try {
      disks = await api.listUsbDisks();
      if (!disks.some((d) => d.number === diskNumber)) {
        diskNumber = disks.length === 1 ? disks[0].number : null;
      }
    } catch (caught) {
      disks = [];
      onNotify("error", "Не удалось получить список флешек", humanizeError(caught));
    } finally {
      loadingDisks = false;
    }
  }

  async function pickIso() {
    try {
      const selected = await open({
        multiple: false,
        title: "Выберите ISO-образ Windows",
        filters: [{ name: "ISO", extensions: ["iso"] }],
      });
      if (typeof selected === "string") {
        isoPath = selected;
      }
    } catch (caught) {
      onNotify("error", "Не удалось выбрать файл", String(caught));
    }
  }

  async function start() {
    if (!ready || diskNumber === null) return;
    confirming = false;
    running = true;
    percent = 0;
    phase = "Запуск";
    detail = "";
    done = "";
    failure = "";
    let downloaded = "";
    try {
      let path = isoPath;
      if (source === "auto") {
        stage = "download";
        phase = "Запрос ссылки у Microsoft";
        path = await api.downloadWindowsIso(product, fidoLang);
        downloaded = path;
      }
      stage = "write";
      percent = 0;
      detail = "";
      phase = "Подготовка флешки";
      done = await api.createUsbInstaller(path, diskNumber, variant, scheme);
      percent = 100;
      if (downloaded && !keepIso) {
        await api.deleteCachedIso(downloaded).catch(() => {});
      }
      onNotify("success", "Флешка готова", "Можно загружаться с неё.");
      await refreshDisks();
    } catch (caught) {
      failure = humanizeError(caught);
      onNotify("error", "Не удалось записать флешку", failure);
    } finally {
      running = false;
      stage = "";
    }
  }

  async function stop() {
    await api.cancelIsoDownload().catch(() => {});
  }

  async function openMsPage() {
    try {
      await openUrl(MS_PAGES[product]);
    } catch (caught) {
      onNotify("error", "Не удалось открыть ссылку", String(caught));
    }
  }

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;
    const track = (promise: Promise<UnlistenFn>) =>
      void promise.then((fn) => (disposed ? fn() : unlisteners.push(fn)));

    track(
      listen<{ percent: number; phase: string }>("usb-progress", (event) => {
        if (stage !== "write") return;
        percent = event.payload.percent;
        phase = event.payload.phase;
      }),
    );
    track(
      listen<{ percent: number; phase: string; receivedMb: number; totalMb: number; speedMbs: number }>(
        "iso-progress",
        (event) => {
          if (stage !== "download") return;
          percent = event.payload.percent;
          phase = event.payload.phase;
          const p = event.payload;
          detail = p.totalMb
            ? `${p.receivedMb} / ${p.totalMb} МБ${p.speedMbs ? ` · ${p.speedMbs} МБ/с` : ""}`
            : "";
        },
      ),
    );
    void api.adminStatus().then((value) => (isAdmin = value)).catch(() => {});
    void refreshDisks();
    return () => {
      disposed = true;
      unlisteners.forEach((fn) => fn());
    };
  });
</script>

<div class="usb-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>Windows to USB</p>
      <h1>Windows <span>на флешку</span></h1>
      <p class="lead">
        Выберите вариант Windows, ISO-образ и флешку — приложение само сделает её загрузочной.
      </p>
    </div>
    <img class="hero-art" src="/art/hero-usb.svg" alt="" />
  </header>

  {#if !isAdmin}
    <div class="notice warn">
      <Icon name="alert" size={18} />
      Для записи флешки нужны права администратора. Перезапустите приложение от имени администратора.
    </div>
  {/if}

  <h2 class="section-title">1. Вариант Windows</h2>
  <div class="variant-grid">
    {#each VARIANTS as v, i (v.id)}
      <button
        type="button"
        class="variant"
        class:active={variant === v.id}
        style="--i:{i};"
        disabled={running}
        onclick={() => (variant = v.id)}
      >
        <div class="variant-banner"><img src={v.art} alt="" /></div>
        <div class="variant-body">
          <div class="variant-top">
            <strong>{v.name}</strong>
            {#if variant === v.id}<Icon name="check" size={16} />{/if}
          </div>
          <span class="pill {v.risk}">{v.badge}</span>
          <p>{v.text}</p>
          <small>{RISK_LABEL[v.risk]}</small>
        </div>
      </button>
    {/each}
  </div>
  {#if variant !== "original"}
    <p class="hint">
      Образ Windows не изменяется: твики применяются при первом входе в систему через автоответный файл и скрипт.
    </p>
  {/if}

  <h2 class="section-title">2. Образ Windows</h2>
  <div class="card">
    <div class="tabs">
      <button type="button" class:active={source === "auto"} disabled={running} onclick={() => (source = "auto")}>Скачать автоматически</button>
      <button type="button" class:active={source === "file"} disabled={running} onclick={() => (source = "file")}>У меня есть ISO</button>
    </div>

    {#if source === "auto"}
      <div class="row wrap">
        <select bind:value={product} disabled={running}>
          {#each PRODUCTS as p}<option value={p.id}>{p.label}</option>{/each}
        </select>
        <select bind:value={langId} disabled={running}>
          {#each LANGS as l}<option value={l.id}>{l.label}</option>{/each}
        </select>
        <span class="muted">Официальный образ x64 с серверов Microsoft, около 6–7 ГБ.</span>
      </div>
      <label class="check"><input type="checkbox" bind:checked={keepIso} disabled={running} />Оставить скачанный ISO после записи (для следующей флешки)</label>
      <small class="muted">Приложение само получает ссылку Microsoft и качает выбранную версию. Выбранный вариант (Lite, Gaming…) применяется к этому образу при установке; прерванная загрузка продолжится с того же места.</small>
    {:else}
      <div class="row">
        <div class="grow">
          <strong>{isoPath ? isoPath.split(/[\\/]/).pop() : "Файл не выбран"}</strong>
          <span class="muted">{isoPath || "Выберите ISO-образ Windows 10 или 11."}</span>
        </div>
        <button type="button" class="btn" disabled={running} onclick={pickIso}>
          <Icon name="folder" size={16} />Выбрать ISO
        </button>
      </div>
    {/if}
  </div>

  <h2 class="section-title">3. Флешка</h2>
  <div class="card">
    <div class="row">
      <div class="grow"><strong>Съёмные USB-накопители</strong>
        <span class="muted">Системные и внутренние диски не показываются.</span></div>
      <button type="button" class="btn" disabled={running || loadingDisks} onclick={refreshDisks}>
        <Icon name="refresh" size={16} />{loadingDisks ? "Поиск…" : "Обновить"}
      </button>
    </div>
    {#if disks.length === 0}
      <p class="empty">{loadingDisks ? "Ищем флешки…" : "USB-накопители не найдены. Вставьте флешку и нажмите «Обновить»."}</p>
    {:else}
      <div class="disk-list">
        {#each disks as d (d.number)}
          <button type="button" class="disk" class:active={diskNumber === d.number} disabled={running}
            onclick={() => (diskNumber = d.number)}>
            <img src="/art/hero-usb.svg" alt="" />
            <span><strong>{d.name || "USB-накопитель"}</strong>
              <small>Диск {d.number} · {d.sizeGb} ГБ · {d.style}{d.letters ? ` · ${d.letters}:` : ""}</small></span>
          </button>
        {/each}
      </div>
    {/if}
    <div class="scheme">
      <span class="muted">Схема разделов:</span>
      <button type="button" class="chip" class:active={scheme === "gpt"} disabled={running} onclick={() => (scheme = "gpt")}>GPT · UEFI</button>
      <button type="button" class="chip" class:active={scheme === "mbr"} disabled={running} onclick={() => (scheme = "mbr")}>MBR · BIOS</button>
    </div>
  </div>

  {#if running || done || failure}
    <div class="card progress">
      {#if source === "auto"}
        <div class="steps">
          <span class:on={stage === "download"} class:ok={stage === "write" || (done && !failure)}>1. Скачивание</span>
          <span class:on={stage === "write"} class:ok={done && !failure}>2. Запись на флешку</span>
        </div>
      {/if}
      <div class="row"><strong>{running ? phase : failure ? "Ошибка" : "Готово"}</strong><span class="muted">{Math.round(percent)}%</span></div>
      <div class="bar"><div class="fill" style="width:{percent}%"></div></div>
      {#if running && detail}<small class="muted">{detail}</small>{/if}
      {#if running && stage === "download"}
        <button type="button" class="btn" onclick={stop}>Остановить загрузку</button>
      {/if}
      {#if done}<p class="ok">{done}</p>{/if}
      {#if failure}
        <p class="err">{failure}</p>
        {#if source === "auto"}
          <div class="row wrap">
            <button type="button" class="btn" onclick={openMsPage}><Icon name="external" size={14} />Открыть страницу Microsoft</button>
            <button type="button" class="btn" onclick={() => (source = "file")}>Выбрать ISO вручную</button>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <div class="actions">
    {#if confirming && selectedDisk}
      <div class="confirm">
        <Icon name="alert" size={18} />
        <span>Все данные на «{selectedDisk.name}» (диск {selectedDisk.number}, {selectedDisk.sizeGb} ГБ) будут <b>безвозвратно удалены</b>. Вариант: {selectedVariant.name}.</span>
        <button type="button" class="btn danger" onclick={start}>Да, стереть и записать</button>
        <button type="button" class="btn" onclick={() => (confirming = false)}>Отмена</button>
      </div>
    {:else}
      <button type="button" class="btn primary" disabled={!ready || !isAdmin} onclick={() => (confirming = true)}>
        <Icon name="usb" size={18} />Записать Windows на флешку
      </button>
    {/if}
  </div>
</div>

<style>
  .usb-page { padding-bottom: 40px; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 680px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .section-title { margin: 28px 4px 12px; font-size: 15px; color: var(--muted-strong); }
  .notice { display: flex; gap: 10px; align-items: center; margin: 18px 4px 0; padding: 12px 16px; border-radius: var(--radius-md); border: 1px solid var(--border-strong); background: var(--surface); }
  .notice.warn { color: var(--danger); border-color: color-mix(in srgb, var(--danger) 40%, transparent); }
  .variant-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 14px; }
  .variant { text-align: left; padding: 0; overflow: hidden; color: var(--text); background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-lg); cursor: pointer; transition: transform 200ms var(--ease-out), border-color 200ms; animation: fade-up 500ms var(--ease-out) both; animation-delay: calc(var(--i) * 60ms); }
  .variant:hover:not(:disabled) { transform: translateY(-3px); border-color: var(--border-strong); }
  .variant.active { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent), 0 10px 40px -18px var(--accent); }
  .variant-banner { height: 96px; background: var(--bg-soft); overflow: hidden; }
  .variant-banner img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .variant-body { padding: 14px 16px 16px; display: grid; gap: 8px; }
  .variant-top { display: flex; justify-content: space-between; align-items: center; color: var(--accent); }
  .variant-top strong { color: var(--text); font-size: 16px; }
  .variant-body p { margin: 0; color: var(--muted); font-size: 13px; line-height: 1.45; }
  .variant-body small { color: var(--muted); font-size: 11px; }
  .pill { justify-self: start; padding: 3px 9px; font-size: 11px; border-radius: 99px; border: 1px solid var(--border-strong); color: var(--muted-strong); }
  .pill.high { color: var(--danger); border-color: color-mix(in srgb, var(--danger) 50%, transparent); }
  .hint { margin: 10px 4px 0; color: var(--muted); font-size: 12px; }
  .card { padding: 16px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 12px; }
  .row { display: flex; align-items: center; gap: 14px; }
  .grow { flex: 1; min-width: 0; display: grid; gap: 3px; }
  .grow strong, .grow span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .muted { color: var(--muted); font-size: 12px; }
  .links { display: flex; flex-wrap: wrap; gap: 8px 16px; margin: 10px 4px 0; }
  .link { display: inline-flex; gap: 6px; align-items: center; background: none; border: 0; padding: 0; color: var(--muted-strong); font-size: 12px; cursor: pointer; text-decoration: underline; text-underline-offset: 3px; }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font: inherit; font-size: 13px; color: var(--text); background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); cursor: pointer; }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.primary { background: var(--text); color: var(--bg); font-weight: 600; padding: 13px 22px; }
  .btn.danger { background: var(--danger); color: #111; border-color: var(--danger); font-weight: 600; }
  .empty { margin: 0; color: var(--muted); font-size: 13px; }
  .disk-list { display: grid; gap: 8px; }
  .disk { display: flex; gap: 12px; align-items: center; text-align: left; padding: 8px 12px 8px 8px; font: inherit; color: var(--text); background: var(--bg-soft); border: 1px solid var(--border); border-radius: var(--radius-md); cursor: pointer; }
  .disk.active { border-color: var(--text); }
  .disk img { width: 56px; height: 40px; object-fit: cover; border-radius: 8px; }
  .disk span { display: grid; gap: 2px; }
  .disk small { color: var(--muted); }
  .scheme { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .chip { padding: 6px 12px; font: inherit; font-size: 12px; color: var(--muted-strong); background: none; border: 1px solid var(--border-strong); border-radius: 99px; cursor: pointer; }
  .chip.active { background: var(--text); color: var(--bg); }
  .progress { margin-top: 20px; }
  .tabs { display: flex; gap: 8px; flex-wrap: wrap; }
  .tabs button { padding: 8px 14px; font-size: 13px; color: var(--muted-strong); border: 1px solid var(--border); border-radius: 99px; }
  .tabs button.active { background: var(--text); color: var(--bg); border-color: var(--text); }
  .row.wrap { flex-wrap: wrap; }
  select { padding: 9px 12px; background: var(--bg-soft); border: 1px solid var(--border-strong); border-radius: 10px; color: var(--text); }
  .check { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: var(--muted-strong); }
  .steps { display: flex; gap: 18px; font-size: 12px; color: var(--muted); }
  .steps span.on { color: var(--text); font-weight: 600; }
  .steps span.ok { color: var(--success); }
  .bar { height: 8px; border-radius: 99px; background: var(--surface-strong); overflow: hidden; }
  .fill { height: 100%; background: var(--text); transition: width 400ms var(--ease-out); }
  .ok { margin: 0; color: var(--success); }
  .err { margin: 0; color: var(--danger); white-space: pre-wrap; }
  .actions { margin-top: 22px; }
  .confirm { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; padding: 14px 16px; border-radius: var(--radius-md); border: 1px solid color-mix(in srgb, var(--danger) 50%, transparent); color: var(--danger); }
  .confirm span { flex: 1; min-width: 240px; color: var(--text); }
</style>
