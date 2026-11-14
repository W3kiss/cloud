<script lang="ts">
  import { settings } from "$lib/settings.svelte";

  interface Star {
    x: number;
    y: number;
    z: number; // depth 0.3 .. 1 (bigger = closer)
    r: number;
    ox: number;
    oy: number;
    vx: number;
    vy: number;
    phase: number;
    speed: number;
  }

  let canvas = $state<HTMLCanvasElement>();
  const active = $derived(settings.theme === "star");
  const animate = $derived(settings.animations);

  $effect(() => {
    if (!active || !canvas) return;
    const el = canvas;
    const ctx = el.getContext("2d");
    if (!ctx) return;
    const context: CanvasRenderingContext2D = ctx;
    const moving = animate;

    let width = 0;
    let height = 0;
    let dpr = 1;
    let stars: Star[] = [];
    let raf = 0;
    let running = true;
    const mouse = { x: -9999, y: -9999, active: false };
    let shooting: { x: number; y: number; vx: number; vy: number; life: number } | null = null;
    let nextShot = performance.now() + 4000;

    const RADIUS = 150;

    function build() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = window.innerWidth;
      height = window.innerHeight;
      el.width = Math.floor(width * dpr);
      el.height = Math.floor(height * dpr);
      context.setTransform(dpr, 0, 0, dpr, 0, 0);
      const count = Math.max(90, Math.min(280, Math.floor((width * height) / 9000)));
      stars = Array.from({ length: count }, () => {
        const z = 0.3 + Math.random() * 0.7;
        return {
          x: Math.random() * width,
          y: Math.random() * height,
          z,
          r: 0.4 + z * 1.15,
          ox: 0,
          oy: 0,
          vx: 0,
          vy: 0,
          phase: Math.random() * Math.PI * 2,
          speed: 0.6 + Math.random() * 1.6,
        };
      });
    }

    function frame(now: number) {
      context.clearRect(0, 0, width, height);
      const t = now / 1000;
      const px = mouse.active ? (mouse.x - width / 2) / width : 0;
      const py = mouse.active ? (mouse.y - height / 2) / height : 0;
      const near: { x: number; y: number; a: number }[] = [];

      for (const s of stars) {
        if (moving) {
          // slow upward drift, wrapping around the screen
          s.y -= 0.05 * s.z;
          if (s.y < -4) s.y = height + 4;

          // push away from the cursor, spring back to the home position
          const bx = s.x + s.ox - (mouse.x - 0);
          const by = s.y + s.oy - (mouse.y - 0);
          const d = Math.hypot(bx, by);
          if (mouse.active && d < RADIUS && d > 0.01) {
            const force = ((1 - d / RADIUS) ** 2) * 2.4 * s.z;
            s.vx += (bx / d) * force;
            s.vy += (by / d) * force;
          }
          s.vx += -s.ox * 0.035;
          s.vy += -s.oy * 0.035;
          s.vx *= 0.88;
          s.vy *= 0.88;
          s.ox += s.vx;
          s.oy += s.vy;
        }

        const x = s.x + s.ox - px * 26 * s.z;
        const y = s.y + s.oy - py * 26 * s.z;
        const twinkle = moving ? 0.55 + 0.45 * Math.sin(t * s.speed + s.phase) : 0.8;
        let alpha = (0.28 + 0.55 * s.z) * twinkle;
        let radius = s.r;

        if (mouse.active) {
          const d = Math.hypot(x - mouse.x, y - mouse.y);
          if (d < RADIUS * 1.25) {
            const k = 1 - d / (RADIUS * 1.25);
            alpha = Math.min(1, alpha + k * 0.7);
            radius += k * 1.1;
            if (d < 120) near.push({ x, y, a: 1 - d / 120 });
          }
        }

        context.globalAlpha = alpha;
        context.fillStyle = "#ffffff";
        context.beginPath();
        context.arc(x, y, radius, 0, Math.PI * 2);
        context.fill();
      }

      // thin "constellation" lines from the cursor to nearby stars
      if (mouse.active && moving) {
        context.lineWidth = 0.7;
        context.strokeStyle = "#ffffff";
        for (const n of near) {
          context.globalAlpha = n.a * 0.4;
          context.beginPath();
          context.moveTo(mouse.x, mouse.y);
          context.lineTo(n.x, n.y);
          context.stroke();
        }
      }

      // an occasional shooting star
      if (moving) {
        if (!shooting && now > nextShot) {
          shooting = {
            x: Math.random() * width * 0.8,
            y: Math.random() * height * 0.4,
            vx: 7 + Math.random() * 4,
            vy: 3 + Math.random() * 2.5,
            life: 1,
          };
          nextShot = now + 6000 + Math.random() * 9000;
        }
        if (shooting) {
          const sh = shooting;
          const gradient = context.createLinearGradient(sh.x, sh.y, sh.x - sh.vx * 9, sh.y - sh.vy * 9);
          gradient.addColorStop(0, "rgba(255,255,255,0.9)");
          gradient.addColorStop(1, "rgba(255,255,255,0)");
          context.globalAlpha = sh.life;
          context.strokeStyle = gradient;
          context.lineWidth = 1.4;
          context.beginPath();
          context.moveTo(sh.x, sh.y);
          context.lineTo(sh.x - sh.vx * 9, sh.y - sh.vy * 9);
          context.stroke();
          sh.x += sh.vx;
          sh.y += sh.vy;
          sh.life -= 0.012;
          if (sh.life <= 0 || sh.x > width + 80 || sh.y > height + 80) shooting = null;
        }
      }

      context.globalAlpha = 1;
      if (moving && running) raf = requestAnimationFrame(frame);
    }

    const onMove = (event: PointerEvent) => {
      mouse.x = event.clientX;
      mouse.y = event.clientY;
      mouse.active = true;
    };
    const onLeave = () => {
      mouse.active = false;
      mouse.x = mouse.y = -9999;
    };
    // A click sends a shock wave through the nearby stars.
    const onDown = (event: PointerEvent) => {
      for (const s of stars) {
        const dx = s.x + s.ox - event.clientX;
        const dy = s.y + s.oy - event.clientY;
        const d = Math.hypot(dx, dy);
        if (d < 260 && d > 0.01) {
          const force = (1 - d / 260) * 14 * s.z;
          s.vx += (dx / d) * force;
          s.vy += (dy / d) * force;
        }
      }
    };
    const onResize = () => {
      build();
      if (!moving) frame(performance.now());
    };
    const onVisibility = () => {
      running = !document.hidden;
      if (running && moving) raf = requestAnimationFrame(frame);
    };

    build();
    if (moving) {
      raf = requestAnimationFrame(frame);
    } else {
      frame(performance.now());
    }
    window.addEventListener("pointermove", onMove, { passive: true });
    window.addEventListener("pointerdown", onDown, { passive: true });
    window.addEventListener("blur", onLeave);
    document.documentElement.addEventListener("pointerleave", onLeave);
    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      running = false;
      cancelAnimationFrame(raf);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerdown", onDown);
      window.removeEventListener("blur", onLeave);
      document.documentElement.removeEventListener("pointerleave", onLeave);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  });
</script>

{#if active}
  <canvas bind:this={canvas} class="starfield" aria-hidden="true"></canvas>
{/if}

<style>
  .starfield {
    position: fixed;
    inset: 0;
    z-index: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
