<script lang="ts">
  import { onMount } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";

  let isHovered = false;
  let hideTimer: number | null = null;
  let collapseTimer: number | null = null;

  async function handleMouseEnter() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
    if (collapseTimer) {
      clearTimeout(collapseTimer);
      collapseTimer = null;
    }
    if (!isHovered) {
      await browserBridge.setTopbarOverlay(true);
      isHovered = true;
    }
  }

  function handleMouseLeave() {
    if (hideTimer) {
      clearTimeout(hideTimer);
    }
    // 250ms grace period before auto-hiding when cursor leaves pill area
    hideTimer = window.setTimeout(() => {
      isHovered = false;
      // Wait for smooth 220ms CSS slide-up transition before collapsing webview height in Rust
      collapseTimer = window.setTimeout(async () => {
        if (!isHovered) {
          await browserBridge.setTopbarOverlay(false);
        }
        collapseTimer = null;
      }, 240);
      hideTimer = null;
    }, 250);
  }

  function handleMouseMove(e: MouseEvent) {
    // If cursor moves downwards past the pill card (height 32px + margin 6px = 38px, with 6px buffer = 44px), trigger leave
    if (e.clientY > 44) {
      handleMouseLeave();
    }
  }

  onMount(() => {
    window.addEventListener("mouseleave", handleMouseLeave);
    window.addEventListener("blur", handleMouseLeave);

    return () => {
      window.removeEventListener("mouseleave", handleMouseLeave);
      window.removeEventListener("blur", handleMouseLeave);
    };
  });
</script>

<div
  class="topbar-root"
  role="region"
  aria-label="Window Controls"
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:mousemove={handleMouseMove}
>
  <!-- Invisible trigger area at top edge of the corner when collapsed -->
  {#if !isHovered}
    <div
      class="topbar-edge-trigger"
      on:mouseenter={handleMouseEnter}
      role="presentation"
    ></div>
  {/if}

  <!-- Floating Window Controls Rectangle with smooth border and 3 buttons -->
  <div class="controls-card {isHovered ? 'visible' : ''}">
    <!-- Minimize -->
    <button
      type="button"
      class="win-btn"
      title="Minimize"
      aria-label="Minimize window"
      on:click={() => browserBridge.minimizeWindow()}
    >
      <svg width="10" height="1" viewBox="0 0 10 1">
        <rect width="10" height="1" fill="currentColor"/>
      </svg>
    </button>

    <!-- Maximize / Restore -->
    <button
      type="button"
      class="win-btn"
      title="Maximize / Restore"
      aria-label="Maximize window"
      on:click={() => browserBridge.maximizeWindow()}
    >
      <svg width="10" height="10" viewBox="0 0 10 10">
        <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1"/>
      </svg>
    </button>

    <!-- Cancel / Close -->
    <button
      type="button"
      class="win-btn close-btn"
      title="Close"
      aria-label="Close window"
      on:click={() => browserBridge.closeWindow()}
    >
      <svg width="10" height="10" viewBox="0 0 10 10">
        <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent !important;
    background-color: transparent !important;
  }

  .topbar-root {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: transparent !important;
    user-select: none;
    -webkit-user-select: none;
    display: flex;
    justify-content: center;
  }

  .topbar-edge-trigger {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 8px;
    background: transparent !important;
    cursor: default;
    z-index: 10;
  }

  .controls-card {
    width: calc(100% - 12px);
    max-width: 126px;
    height: 32px;
    margin-top: 6px;
    display: flex;
    align-items: center;
    background-color: #2b2b2b;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45), 0 1px 3px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    transform: translateY(-44px);
    opacity: 0;
    pointer-events: none;
    transition: transform 220ms cubic-bezier(0.16, 1, 0.3, 1), opacity 180ms ease;
    will-change: transform, opacity;
    z-index: 20;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .controls-card.visible {
    transform: translateY(0);
    opacity: 1;
    pointer-events: auto;
  }

  .win-btn {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #cfcfd6;
    cursor: pointer;
    outline: none;
    padding: 0;
    transition: background-color 0.15s ease, color 0.15s ease;
    -webkit-app-region: no-drag;
  }

  .win-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .win-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
  }

  .win-btn.close-btn:hover {
    background-color: #e81123 !important;
    color: #ffffff !important;
  }

  .win-btn.close-btn:active {
    background-color: #c4101e !important;
    color: #ffffff !important;
  }
</style>
