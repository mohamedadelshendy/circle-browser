<script lang="ts">
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
    // 400ms grace period so moving near buttons never flickers
    hideTimer = window.setTimeout(() => {
      isHovered = false;
      // Wait for smooth 240ms CSS slide-up transition before collapsing webview height in Rust
      collapseTimer = window.setTimeout(async () => {
        if (!isHovered) {
          await browserBridge.setTopbarOverlay(false);
        }
        collapseTimer = null;
      }, 260);
      hideTimer = null;
    }, 400);
  }
</script>

<div
  class="topbar-root"
  role="region"
  aria-label="Window Controls Top Bar"
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
>
  <!-- Invisible 5px trigger strip at the very top edge of the screen when bar is collapsed -->
  {#if !isHovered}
    <div
      class="topbar-edge-trigger"
      on:mouseenter={handleMouseEnter}
      role="presentation"
      title="Hover to reveal Window Controls"
    ></div>
  {/if}

  <!-- Floating Window Controls Bar (#323232) -->
  <header
    class="topbar-bar {isHovered ? 'visible' : ''}"
    data-tauri-drag-region
  >
    <!-- Left & Middle Draggable Window Area -->
    <div class="topbar-drag-area" data-tauri-drag-region></div>

    <!-- Right Window Control Buttons (Minimize, Maximize/Restore, Close) -->
    <div class="topbar-window-controls" data-tauri-drag-region="false">
      <!-- Minimize -->
      <button
        type="button"
        class="topbar-btn"
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
        class="topbar-btn"
        title="Maximize / Restore"
        aria-label="Maximize window"
        on:click={() => browserBridge.maximizeWindow()}
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1"/>
        </svg>
      </button>

      <!-- Close -->
      <button
        type="button"
        class="topbar-btn close-btn"
        title="Close"
        aria-label="Close window"
        on:click={() => browserBridge.closeWindow()}
      >
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </header>
</div>

<style>
  .topbar-root {
    width: 100%;
    height: 100vh;
    position: relative;
    overflow: hidden;
    background: transparent !important;
    user-select: none;
    -webkit-user-select: none;
  }

  .topbar-edge-trigger {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 5px;
    background: transparent !important;
    cursor: default;
    z-index: 10;
  }

  .topbar-bar {
    width: 100%;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: #323232;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
    transition: transform 240ms cubic-bezier(0.16, 1, 0.3, 1), opacity 200ms ease;
    will-change: transform, opacity;
    -webkit-app-region: drag;
    z-index: 20;
  }

  .topbar-bar.visible {
    transform: translateY(0);
    opacity: 1;
    pointer-events: auto;
  }

  .topbar-drag-area {
    flex: 1;
    height: 100%;
    -webkit-app-region: drag;
  }

  .topbar-window-controls {
    display: flex;
    align-items: center;
    height: 100%;
    -webkit-app-region: no-drag;
    flex-shrink: 0;
  }

  .topbar-btn {
    width: 46px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #c0c0c6;
    cursor: pointer;
    outline: none;
    transition: background 0.15s, color 0.15s;
    -webkit-app-region: no-drag;
  }

  .topbar-btn:hover {
    background: rgba(255, 255, 255, 0.09);
    color: #ffffff;
  }

  .topbar-btn.close-btn:hover {
    background: #e81123 !important;
    color: #ffffff !important;
  }
</style>

