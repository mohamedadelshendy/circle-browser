<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";
  import type { Tab } from "../lib/types";

  export let tabs: Tab[] = [];
  export let activeTabId: string | null = null;

  const dispatch = createEventDispatcher<{
    switchTab: string;
    closeTab: string;
    newTab: void;
  }>();

  let draggedIndex: number | null = null;
  let dragOverIndex: number | null = null;

  function handleDragStart(index: number, e: DragEvent) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", `${index}`);
    }
  }

  function handleDragOver(index: number, e: DragEvent) {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== index) {
      dragOverIndex = index;
    }
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  async function handleDrop(targetIndex: number, e: DragEvent) {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== targetIndex) {
      await browserBridge.reorderTabs(draggedIndex, targetIndex);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }
</script>

<div class="tab-bar" data-tauri-drag-region>
  <!-- Lion Brand / Tab Strip Icon -->
  <button type="button" class="brand-icon" title="Circle Browser" on:click={() => dispatch("newTab")} aria-label="Circle Browser">
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <!-- Shield / Lion Head Outline -->
      <path
        d="M12 2L4 5V11.09C4 16.14 7.41 20.85 12 22C16.59 20.85 20 16.14 20 11.09V5L12 2Z"
        fill="#fb542b"
      />
      <path
        d="M12 6L8 8V12C8 14.5 9.7 16.8 12 17.5C14.3 16.8 16 14.5 16 12V8L12 6Z"
        fill="#ffffff"
        opacity="0.9"
      />
    </svg>
  </button>

  <!-- Draggable Tab Strip -->
  <div class="tabs-container" data-tauri-drag-region>
    {#each tabs as tab, index (tab.id)}
      <div
        class="tab {tab.id === activeTabId ? 'active' : ''} {dragOverIndex === index ? 'drag-over' : ''}"
        draggable="true"
        on:dragstart={(e) => handleDragStart(index, e)}
        on:dragover={(e) => handleDragOver(index, e)}
        on:dragleave={handleDragLeave}
        on:drop={(e) => handleDrop(index, e)}
        on:dragend={handleDragEnd}
        on:click={() => dispatch("switchTab", tab.id)}
        on:keydown={(e) => (e.key === "Enter" || e.key === " ") && dispatch("switchTab", tab.id)}
        role="tab"
        aria-selected={tab.id === activeTabId}
        tabindex="0"
      >
        <!-- Favicon (16x16px) / Loading spinner -->
        <span class="tab-favicon">
          {#if tab.is_loading}
            <span class="tab-spinner"></span>
          {:else if tab.url.includes("google.com")}
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
            </svg>
          {:else}
            <!-- Brave style globe / shield icon -->
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L4 5v6.09c0 5.05 3.41 9.76 8 10.91 4.59-1.15 8-5.86 8-10.91V5l-8-3z"/>
            </svg>
          {/if}
        </span>

        <!-- Title text (truncated with ellipsis, 12-13px font, color: #f1f2f6) -->
        <span class="tab-title" title={tab.title || "New tab"}>
          {tab.title || "New tab"}
        </span>

        <!-- Circular Close button (16x16 icon inside 24x24 hover circle) -->
        <button
          class="tab-close-circle"
          title="Close tab (Ctrl+W)"
          on:click|stopPropagation={() => dispatch("closeTab", tab.id)}
          aria-label="Close tab"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    {/each}

    <!-- New Tab (+) button: 28x28px rounded circular button directly next to tabs -->
    <button
      class="new-tab-btn"
      title="New tab (Ctrl+T)"
      on:click={() => dispatch("newTab")}
      aria-label="New tab"
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <!-- Draggable Spacer before window controls -->
  <div class="header-drag-space" data-tauri-drag-region></div>

  <!-- Window control buttons on the far right (46px wide x 32px high, no-drag) -->
  <div class="window-controls">
    <button
      class="win-btn"
      title="Minimize"
      on:click={() => browserBridge.minimizeWindow()}
      aria-label="Minimize window"
    >
      <svg width="10" height="1" viewBox="0 0 10 1">
        <rect width="10" height="1" fill="currentColor"/>
      </svg>
    </button>
    <button
      class="win-btn"
      title="Maximize"
      on:click={() => browserBridge.maximizeWindow()}
      aria-label="Maximize window"
    >
      <svg width="10" height="10" viewBox="0 0 10 10">
        <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1"/>
      </svg>
    </button>
    <button
      class="win-btn close"
      title="Close"
      on:click={() => browserBridge.closeWindow()}
      aria-label="Close window"
    >
      <svg width="10" height="10" viewBox="0 0 10 10">
        <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .header-drag-space {
    flex: 0 0 12px;
    height: 100%;
    -webkit-app-region: drag;
  }

  .tab-spinner {
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-lion);
    border-radius: 50%;
    animation: tab-spin 0.7s linear infinite;
  }

  @keyframes tab-spin {
    to {
      transform: rotate(360deg);
    }
  }

  .tab.drag-over {
    border-left: 2px solid var(--accent-lion);
  }
</style>
