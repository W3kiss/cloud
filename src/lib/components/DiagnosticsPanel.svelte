<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import { api, humanizeError, type DiagCheck, type ToastTone } from "$lib/api";

  let {
    onNotify,
  }: {
    onNotify: (tone: ToastTone, title: string, message?: string) => void;
  } = $props();

  let checks = $state<DiagCheck[]>([]);
  let loading = $state(false);
  let fixing = $state("");

  const problems = $derived(checks.filter((c) => c.status !== "ok").length);

  async function run() {
    loading = true;
    try {
      checks = await api.runDiagnostics();
    } catch (caught) {
      onNotify("error", "Диагностика не выполнена", humanizeError(caught));
    } finally {
      loading = false;
    }
  }

  async function fix(check: DiagCheck) {
    fixing = check.id;
    try {
      const message = await api.fixDiagnostic(check.id);
      onNotify("success", check.title, message);
      await run();
    } catch (caught) {
      onNotify("error", `Не удалось исправить: ${check.title}`, humanizeError(caught));
    } finally {
      fixing = "";
    }
  }

  async function exportReport() {
    try {
      const path = await api.exportDiagnostics();
      onNotify("success", "Отчёт сохранён", path);
    } catch (caught) {
      onNotify("error", "Не удалось создать отчёт", humanizeError(caught));
    }
  }

  onMount(run);
</script>

<div class="diag-page">
  <header class="page-head">
    <div>
      <p class="eyebrow"><span class="live-dot"></span>Self-diagnostics</p>
      <h1>Диагностика <span>системы</span></h1>
      <p class="lead">
        Проверка winget, сети, прав, места на диске и состояния установщика. Ошибки приложения пишутся в лог.
      </p>
    </div>
    <div class="head-actions">
      <button type="button" class="btn" disabled={loading} onclick={run}>
        <Icon name="refresh" size={16} />{loading ? "Проверка…" : "Проверить снова"}
      </button>
      <button type="button" class="btn" onclick={exportReport}>
        <Icon name="download" size={16} />Отчёт для отладки
      </button>
    </div>
    <img class="hero-art" src="/art/hero-diagnostics.svg" alt="" />
  </header>

  {#if checks.length}
    <p class="summary" class:bad={problems > 0}>
      {problems === 0 ? "Всё в порядке." : `Найдено проблем: ${problems}`}
    </p>
  {/if}

  <div class="list">
    {#each checks as check, i (check.id)}
      <article class="item {check.status}" style="--i:{i};">
        <span class="dot"></span>
        <div class="body">
          <strong>{check.title}</strong>
          <span class="detail">{check.detail}</span>
        </div>
        {#if check.code}<code>{check.code}</code>{/if}
        {#if check.fixable && check.status !== "ok"}
          <button type="button" class="btn" disabled={fixing === check.id} onclick={() => fix(check)}>
            {fixing === check.id ? "Исправляем…" : "Исправить"}
          </button>
        {/if}
      </article>
    {/each}
    {#if loading && checks.length === 0}
      <p class="summary">Выполняется проверка…</p>
    {/if}
  </div>
</div>

<style>
  .diag-page { padding-bottom: 40px; }
  .hero-art { width: 220px; max-width: 30%; border-radius: var(--radius-lg); opacity: 0.9; }
  .page-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 14px 4px 0; animation: fade-up 500ms var(--ease-out) both; }
  .page-head > div:first-child { max-width: 640px; }
  .eyebrow { display: flex; align-items: center; gap: 8px; margin: 0 0 8px; color: var(--muted); font-size: 12px; letter-spacing: 0.12em; text-transform: uppercase; }
  .live-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--text); }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.03em; }
  h1 span { color: var(--muted); }
  .lead { color: var(--muted); margin: 8px 0 0; }
  .head-actions { display: flex; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
  .btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; font-size: 13px; background: var(--surface-strong); border: 1px solid var(--border-strong); border-radius: var(--radius-md); }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .summary { margin: 22px 4px 12px; color: var(--muted-strong); }
  .summary.bad { color: var(--danger); }
  .list { display: grid; gap: 10px; margin-top: 8px; }
  .item { display: flex; align-items: center; gap: 14px; padding: 14px 16px; border-radius: var(--radius-lg); border: 1px solid var(--border); background: var(--surface); animation: fade-up 450ms var(--ease-out) both; animation-delay: calc(var(--i) * 40ms); }
  .dot { width: 10px; height: 10px; border-radius: 50%; background: #4ade80; flex: none; }
  .item.warn .dot { background: #fbbf24; }
  .item.fail .dot { background: var(--danger); }
  .body { flex: 1; min-width: 0; display: grid; gap: 3px; }
  .detail { color: var(--muted); font-size: 12px; white-space: pre-wrap; word-break: break-word; }
  code { font-size: 11px; padding: 3px 8px; border-radius: 99px; border: 1px solid var(--border-strong); color: var(--muted-strong); }
</style>
