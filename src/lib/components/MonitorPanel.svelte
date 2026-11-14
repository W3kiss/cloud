<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    api,
    humanizeError,
    type DiskResult,
    type HwStats,
    type StressResult,
    type ToastTone,
  } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  const HISTORY = 60;
  let stats = $state<HwStats | null>(null);
  let cpuHist = $state<number[]>([]);
  let ramHist = $state<number[]>([]);
  let gpuHist = $state<number[]>([]);
  let tempHist = $state<number[]>([]);

  const push = (list: number[], value: number | null | undefined): number[] =>
    value == null ? list : [...list, value].slice(-HISTORY);

  function spark(values: number[], max = 100): string {
    if (values.length < 2) return "";
    const step = 100 / (HISTORY - 1);
    const offset = (HISTORY - values.length) * step;
    return values
      .map((v, i) => `${(offset + i * step).toFixed(1)},${(30 - Math.min(v, max) / max * 28 - 1).toFixed(1)}`)
      .join(" ");
  }

  const temp = $derived(stats?.cpuTemp ?? stats?.gpuTemp ?? null);

  // ---- benchmark state
  let drives = $state<string[]>([]);
  let drive = $state("C:");
  let sizeMb = $state(512);
  let duration = $state(30);
  let running = $state<"" | "disk" | "cpu">("");
  let phase = $state("");
  let percent = $state(0);
  let disk = $state<DiskResult | null>(null);
  let stress = $state<StressResult | null>(null);
  let confirmStress = $state(false);

  async function runDisk() {
    running = "disk";
    percent = 0;
    disk = null;
    try {
      disk = await api.runDiskBench(drive, sizeMb);
    } catch (caught) {
      onNotify("error", "Тест диска не выполнен", humanizeError(caught));
    } finally {
      running = "";
    }
  }

  async function runStress() {
    confirmStress = false;
    running = "cpu";
    percent = 0;
    stress = null;
    try {
      stress = await api.runCpuStress(duration);
    } catch (caught) {
      onNotify("error", "Стресс-тест остановлен", humanizeError(caught));
    } finally {
      running = "";
    }
  }

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;
    const track = (promise: Promise<UnlistenFn>) =>
      void promise.then((fn) => (disposed ? fn() : unlisteners.push(fn)));

    track(
      listen<HwStats>("hw-stats", (event) => {
        stats = event.payload;
        cpuHist = push(cpuHist, event.payload.cpu);
        ramHist = push(ramHist, event.payload.ramPercent);
        gpuHist = push(gpuHist, event.payload.gpu);
        tempHist = push(tempHist, event.payload.cpuTemp ?? event.payload.gpuTemp);
      }),
    );
    track(
      listen<{ phase: string; percent: number }>("bench-progress", (event) => {
        phase = event.payload.phase;
        percent = event.payload.percent;
      }),
    );
    void api.startMonitor().catch(() => {});
    void api.listDrives().then((list) => {
      drives = list;
      if (list.length && !list.includes(drive)) drive = list[0];
    }).catch(() => {});

    return () => {
      disposed = true;
      unlisteners.forEach((fn) => fn());
      void api.stopMonitor().catch(() => {});
    };
  });
</script>

<div class="mon-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>Hardware & benchmark</p>
      <h1>Монитор <span>и тесты</span></h1>
      <p class="lead">Загрузка процессора, памяти и видеокарты в реальном времени, быстрый тест диска и проверка стабильности.</p>
    </div>
    <img class="hero-art" src="/art/hero-monitor.svg" alt="" />
  </header>

  <div class="widgets">
    <article class="widget">
      <header><span>CPU</span><b>{stats ? `${stats.cpu.toFixed(0)}%` : "—"}</b></header>
      <svg viewBox="0 0 100 30" preserveAspectRatio="none"><polyline points={spark(cpuHist)} /></svg>
      <small>{stats?.cpuName || "Процессор"}</small>
    </article>
    <article class="widget">
      <header><span>RAM</span><b>{stats ? `${stats.ramPercent.toFixed(0)}%` : "—"}</b></header>
      <svg viewBox="0 0 100 30" preserveAspectRatio="none"><polyline points={spark(ramHist)} /></svg>
      <small>{stats ? `${stats.ramUsedGb} / ${stats.ramTotalGb} ГБ` : "Память"}</small>
    </article>
    <article class="widget">
      <header><span>GPU</span><b>{stats?.gpu != null ? `${stats.gpu.toFixed(0)}%` : "н/д"}</b></header>
      <svg viewBox="0 0 100 30" preserveAspectRatio="none"><polyline points={spark(gpuHist)} /></svg>
      <small>{stats?.gpuName ?? "Видеокарта (3D-нагрузка)"}</small>
    </article>
    <article class="widget">
      <header><span>Температура</span><b>{temp != null ? `${temp.toFixed(0)}°C` : "н/д"}</b></header>
      <svg viewBox="0 0 100 30" preserveAspectRatio="none"><polyline points={spark(tempHist, 110)} /></svg>
      <small>{temp != null ? (stats?.cpuTemp != null ? "Процессор (ACPI)" : "Видеокарта") : "Датчик недоступен в Windows без драйвера"}</small>
    </article>
  </div>

  <h2 class="section-title">Тест диска</h2>
  <div class="card">
    <div class="row">
      <select bind:value={drive} disabled={running !== ""}>
        {#each drives as d}<option value={d}>{d}</option>{/each}
      </select>
      <select bind:value={sizeMb} disabled={running !== ""}>
        <option value={256}>256 МБ</option>
        <option value={512}>512 МБ</option>
        <option value={1024}>1 ГБ</option>
      </select>
      {#if running === "disk"}
        <button type="button" class="btn" onclick={() => api.cancelBench()}>Остановить</button>
      {:else}
        <button type="button" class="btn primary" disabled={running !== ""} onclick={runDisk}><Icon name="rocket" size={15} />Запустить тест</button>
      {/if}
    </div>
    {#if running === "disk"}
      <div class="bar"><div class="fill" style="width:{percent}%"></div></div>
      <small class="muted">{phase}</small>
    {/if}
    {#if disk}
      <div class="results">
        <div><b>{disk.seqReadMbs}</b><small>МБ/с чтение</small></div>
        <div><b>{disk.seqWriteMbs}</b><small>МБ/с запись</small></div>
        <div><b>{disk.randReadIops}</b><small>IOPS 4K</small></div>
        <div><b>{disk.randReadMbs}</b><small>МБ/с 4K</small></div>
      </div>
    {/if}
    <small class="muted">Файл создаётся в папке SetupHubBench на выбранном диске и удаляется после теста. Для точного результата закройте тяжёлые программы.</small>
  </div>

  <h2 class="section-title">Стресс-тест процессора</h2>
  <div class="card">
    <div class="row">
      <select bind:value={duration} disabled={running !== ""}>
        <option value={15}>15 секунд</option>
        <option value={30}>30 секунд</option>
        <option value={60}>60 секунд</option>
      </select>
      {#if running === "cpu"}
        <button type="button" class="btn" onclick={() => api.cancelBench()}>Остановить</button>
      {:else if confirmStress}
        <span class="warn"><Icon name="alert" size={16} />Процессор будет загружен на 100%, возможен нагрев и шум вентиляторов.</span>
        <button type="button" class="btn danger" onclick={runStress}>Начать</button>
        <button type="button" class="btn" onclick={() => (confirmStress = false)}>Отмена</button>
      {:else}
        <button type="button" class="btn primary" disabled={running !== ""} onclick={() => (confirmStress = true)}><Icon name="rocket" size={15} />Запустить стресс-тест</button>
      {/if}
    </div>
    {#if running === "cpu"}
      <div class="bar"><div class="fill" style="width:{percent}%"></div></div>
      <small class="muted">{phase} · следите за температурой выше</small>
    {/if}
    {#if stress}
      <div class="results">
        <div><b>{stress.score}</b><small>баллов (млн оп/с)</small></div>
        <div><b>{stress.threads}</b><small>потоков</small></div>
        <div><b class:bad={stress.throttlePercent > 10}>{stress.throttlePercent}%</b><small>падение к концу теста</small></div>
      </div>
      <small class="muted">{stress.throttlePercent > 10 ? "Заметное падение производительности: вероятен перегрев (троттлинг) или ограничение питания." : "Производительность стабильна на протяжении всего теста."}</small>
    {/if}
  </div>
</div>

<style>
  .mon-page { padding-bottom: 40px; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 640px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .widgets { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-top: 22px; }
  .widget { padding: 14px 16px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 8px; }
  .widget header { display: flex; justify-content: space-between; align-items: baseline; color: var(--muted); font-size: 12px; letter-spacing: 0.08em; text-transform: uppercase; }
  .widget b { font-size: 26px; color: var(--text); letter-spacing: -0.02em; }
  .widget svg { width: 100%; height: 44px; }
  .widget polyline { fill: none; stroke: var(--accent); stroke-width: 1.6; vector-effect: non-scaling-stroke; stroke-linejoin: round; }
  .widget small { color: var(--muted); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .section-title { margin: 28px 4px 12px; font-size: 15px; color: var(--muted-strong); }
  .card { padding: 16px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); display: grid; gap: 12px; }
  .row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  select { padding: 9px 12px; background: var(--bg-soft); border: 1px solid var(--border-strong); border-radius: 10px; color: var(--text); }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font-size: 13px; background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.primary { background: var(--text); color: var(--bg); font-weight: 600; }
  .btn.danger { background: var(--danger); color: #111; border-color: var(--danger); font-weight: 600; }
  .warn { display: inline-flex; align-items: center; gap: 8px; color: var(--danger); font-size: 12px; flex: 1; min-width: 220px; }
  .bar { height: 8px; border-radius: 99px; background: var(--surface-strong); overflow: hidden; }
  .fill { height: 100%; background: var(--text); transition: width 300ms var(--ease-out); }
  .results { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 10px; }
  .results div { padding: 12px; border-radius: var(--radius-md); background: var(--bg-soft); border: 1px solid var(--border); display: grid; gap: 2px; }
  .results b { font-size: 22px; }
  .results b.bad { color: var(--danger); }
  small, .muted { color: var(--muted); font-size: 12px; }
</style>
