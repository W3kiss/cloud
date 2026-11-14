<script lang="ts">
  let {
    name,
    size = 18,
    stroke = 1.8,
    className = "",
  }: { name: string; size?: number; stroke?: number; className?: string } = $props();

  const paths: Record<string, string[]> = {
    logo: ["M12 2l9 5v10l-9 5-9-5V7l9-5Z", "m8.5 12 2.5 2.5 4.5-4.5"],
    star: ["M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9L12 3Z"],
    home: ["M3 11l9-8 9 8", "M5 10v10h5v-6h4v6h5V10"],
    command: ["M9 9V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V9Z"],
    upload: ["M12 16V4", "m7 9 5-5 5 5", "M4 20h16"],
    grid: ["M4 4h6v6H4z", "M14 4h6v6h-6z", "M4 14h6v6H4z", "M14 14h6v6h-6z"],
    sliders: [
      "M4 6h16",
      "M4 12h16",
      "M4 18h16",
      "M8 4v4",
      "M16 10v4",
      "M10 16v4",
    ],
    archive: ["M4 7h16v14H4z", "M2 3h20v4H2z", "M10 11h4"],
    search: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z", "m21 21-4.35-4.35"],
    shield: ["M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6l8-3Z", "m9 12 2 2 4-5"],
    window: ["M3 5h18v14H3z", "M3 9h18", "M7 7h.01", "M10 7h.01"],
    rocket: [
      "M5 15c-1 2-1 4-1 4s2 0 4-1",
      "M9 15l-4-4c3-6 7-8 13-8 0 6-2 10-8 13Z",
      "M14 7l3 3",
      "M7 17l-2 2",
    ],
    terminal: ["M4 5h16v14H4z", "m7 9 3 3-3 3", "M13 15h4"],
    check: ["m5 12 5 5L20 7"],
    x: ["M6 6l12 12", "M18 6 6 18"],
    download: ["M12 3v12", "m7 10 5 5 5-5", "M5 21h14"],
    spark: [
      "M12 3v4",
      "M12 17v4",
      "M3 12h4",
      "M17 12h4",
      "m5.6 5.6 2.8 2.8",
      "m15.6 15.6 2.8 2.8",
      "m5.6 18.4 2.8-2.8",
      "m15.6 8.4 2.8-2.8",
    ],
    alert: ["M12 3 2 21h20L12 3Z", "M12 9v5", "M12 17h.01"],
    usb: ["M12 3v14", "M9 6l3-3 3 3", "M7 11v3a3 3 0 0 0 3 3", "M17 9v4a3 3 0 0 1-3 3", "M5 9h4v4H5z", "M15 5h4v4h-4z", "M10 20a2 2 0 1 0 4 0 2 2 0 0 0-4 0Z"],
    package: ["M12 3 3 7.5v9L12 21l9-4.5v-9L12 3Z", "M3 7.5 12 12l9-4.5", "M12 12v9"],
    folder: ["M3 6h7l2 2h9v11H3z"],
    refresh: ["M21 12a9 9 0 1 1-3-6.7", "M21 3v6h-6"],
    arrow: ["M5 12h14", "m13 6 6 6-6 6"],
    lock: ["M6 10h12v11H6z", "M9 10V7a3 3 0 0 1 6 0v3"],
    external: ["M14 4h6v6", "M20 4 10 14", "M20 14v6H4V4h6"],
    gamepad: [
      "M7 12h4",
      "M9 10v4",
      "M16 11h.01",
      "M18 13h.01",
      "M6 7h12a4 4 0 0 1 4 4v2a4 4 0 0 1-7 2H9a4 4 0 0 1-7-2v-2a4 4 0 0 1 4-4Z",
    ],
    film: [
      "M3 5h18v14H3z",
      "M7 5v14",
      "M17 5v14",
      "M3 9h4",
      "M3 15h4",
      "M17 9h4",
      "M17 15h4",
    ],
    globe: [
      "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z",
      "M3 12h18",
      "M12 3c2.5 3 2.5 15 0 18",
      "M12 3c-2.5 3-2.5 15 0 18",
    ],
    cpu: [
      "M9 3v3",
      "M15 3v3",
      "M9 18v3",
      "M15 18v3",
      "M3 9h3",
      "M3 15h3",
      "M18 9h3",
      "M18 15h3",
      "M6 6h12v12H6z",
    ],
    play: ["M8 5v14l11-7L8 5Z"],
    settings: [
      "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
      "M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.6V2.8h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z",
    ],
    power: ["M12 3v9", "M6.6 6.6a8 8 0 1 0 10.8 0"],
    copy: ["M8 8h12v12H8z", "M4 16H3V3h13v1"],
    info: ["M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Z", "M12 11v6", "M12 7h.01"],
    message: ["M4 5h16v11H8l-4 4V5Z", "M8 9h8", "M8 12h5"],
    user: ["M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z", "M4 21c0-4 3.6-6 8-6s8 2 8 6"],
    coffee: ["M4 8h13v6a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5V8Z", "M17 9h2a2 2 0 1 1 0 4h-2"],
  };

  let iconPaths = $derived(paths[name] ?? paths.spark);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden="true"
>
  {#each iconPaths as path}
    <path d={path} />
  {/each}
</svg>
