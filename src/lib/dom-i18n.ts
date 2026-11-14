import { EN_PHRASES } from "./i18n-en";

const CYR = /[А-Яа-яЁё]/;
const ATTRS = ["placeholder", "title", "aria-label", "alt"];

type Rule = { re: RegExp; to: string };

let rules: Rule[] | null = null;
const cache = new Map<string, string>();

function buildRules(): Rule[] {
  return Object.keys(EN_PHRASES)
    .sort((a, b) => b.length - a.length)
    .map((key) => {
      const norm = key.replace(/\s+/g, " ").trim();
      const body = norm.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/ /g, "\\s+");
      const pre = /^[А-Яа-яЁё]/.test(norm) ? "(?<![А-Яа-яЁё])" : "";
      const post = /[А-Яа-яЁё]$/.test(norm) ? "(?![А-Яа-яЁё])" : "";
      return { re: new RegExp(pre + body + post, "g"), to: EN_PHRASES[key] };
    });
}

export function translateText(text: string): string {
  if (!CYR.test(text)) return text;
  const hit = cache.get(text);
  if (hit !== undefined) return hit;
  rules ??= buildRules();
  let out = text;
  for (const rule of rules) {
    out = out.replace(rule.re, () => rule.to);
    if (!CYR.test(out)) break;
  }
  if (cache.size > 4000) cache.clear();
  cache.set(text, out);
  return out;
}

const textOriginal = new WeakMap<Node, string>();
const textShown = new WeakMap<Node, string>();
const attrOriginal = new WeakMap<Element, Record<string, string>>();
const attrShown = new WeakMap<Element, Record<string, string>>();

let translating = false;

function handleText(node: Node) {
  const current = node.nodeValue ?? "";
  const shown = textShown.get(node);
  const known = shown !== undefined && shown === current;
  if (translating) {
    const original = known ? (textOriginal.get(node) as string) : current;
    const result = translateText(original);
    if (known && result === shown) return;
    if (result !== original || known) {
      textOriginal.set(node, original);
      textShown.set(node, result);
      if (current !== result) node.nodeValue = result;
    } else {
      textShown.delete(node);
    }
  } else if (known) {
    node.nodeValue = textOriginal.get(node) as string;
    textShown.delete(node);
  }
}

function handleAttrs(el: Element) {
  for (const name of ATTRS) {
    const current = el.getAttribute(name);
    if (current === null) continue;
    const shownMap = attrShown.get(el) ?? {};
    const origMap = attrOriginal.get(el) ?? {};
    const known = shownMap[name] !== undefined && shownMap[name] === current;
    if (translating) {
      const original = known ? origMap[name] : current;
      const result = translateText(original);
      if (result === original && !known) continue;
      origMap[name] = original;
      shownMap[name] = result;
      attrOriginal.set(el, origMap);
      attrShown.set(el, shownMap);
      if (current !== result) el.setAttribute(name, result);
    } else if (known) {
      el.setAttribute(name, origMap[name]);
      delete shownMap[name];
    }
  }
}

function walk(root: Node) {
  if (root.nodeType === Node.TEXT_NODE) {
    handleText(root);
    return;
  }
  if (root.nodeType !== Node.ELEMENT_NODE && root.nodeType !== Node.DOCUMENT_NODE) return;
  if (root.nodeType === Node.ELEMENT_NODE) {
    const tag = (root as Element).tagName;
    if (tag === "SCRIPT" || tag === "STYLE") return;
    handleAttrs(root as Element);
  }
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
  let node = walker.nextNode();
  while (node) {
    if (node.nodeType === Node.TEXT_NODE) {
      const parent = node.parentElement?.tagName;
      if (parent !== "SCRIPT" && parent !== "STYLE") handleText(node);
    } else {
      handleAttrs(node as Element);
    }
    node = walker.nextNode();
  }
}

let observer: MutationObserver | null = null;
let queued = false;
const pending = new Set<Node>();

function flush() {
  queued = false;
  const nodes = [...pending];
  pending.clear();
  observer?.disconnect();
  for (const node of nodes) {
    if (node.isConnected) walk(node);
  }
  observe();
}

function observe() {
  observer?.observe(document.body, {
    childList: true,
    subtree: true,
    characterData: true,
    attributes: true,
    attributeFilter: ATTRS,
  });
}

function schedule(node: Node) {
  pending.add(node);
  if (!queued) {
    queued = true;
    queueMicrotask(flush);
  }
}

/** Translate the whole document when `enabled`, or restore the Russian originals. */
export function setDomTranslation(enabled: boolean): void {
  if (typeof document === "undefined") return;
  translating = enabled;
  if (!observer) {
    observer = new MutationObserver((records) => {
      if (!translating) return;
      for (const record of records) {
        if (record.type === "childList") record.addedNodes.forEach(schedule);
        else if (record.type === "characterData") schedule(record.target);
        else schedule(record.target);
      }
    });
  }
  observer.disconnect();
  walk(document.body);
  observe();
}
