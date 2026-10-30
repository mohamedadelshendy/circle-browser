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

<div class="tab-bar">
  <!-- Brand Icon matching Brave Mockup -->
  <div class="brand-icon" title="Titan Browser">
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 2.18l7 3.12v4.7c0 4.54-3.14 8.78-7 9.88-3.86-1.1-7-5.34-7-9.88V6.3l7-3.12z"/>
    </svg>
  </div>

  <!-- Draggable Tab Strip -->
  <div class="tabs-container">
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
        <!-- Favicon / Loading state -->
        <span class="tab-favicon">
          {#if tab.is_loading}
            <span class="spinner"></span>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="2" y1="12" x2="22" y2="12"></line>
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
            </svg>
          {/if}
        </span>

        <span class="tab-title" title={tab.title || "New tab"}>
          {tab.title || "New tab"}
        </span>

        <button
          class="tab-close-btn"
          title="Close tab (Ctrl+W)"
          on:click|stopPropagation={() => dispatch("closeTab", tab.id)}
          aria-label="Close tab"
        >
          &times;
        </button>
      </div>
    {/each}

    <!-- New Tab (+) button -->
    <button
      class="new-tab-btn"
      title="New tab (Ctrl+T)"
      on:click={() => dispatch("newTab")}
      aria-label="New tab"
    >
      +
    </button>
  </div>

  <!-- Frameless Window Controls -->
  <div class="window-controls">
    <button
      class="win-btn"
      title="Minimize"
      on:click={() => browserBridge.minimizeWindow()}
      aria-label="Minimize window"
    >
      &#x2500;
    </button>
    <button
      class="win-btn"
      title="Maximize"
      on:click={() => browserBridge.maximizeWindow()}
      aria-label="Maximize window"
    >
      &#x25A2;
    </button>
    <button
      class="win-btn close"
      title="Close"
      on:click={() => browserBridge.closeWindow()}
      aria-label="Close window"
    >
      &#x2715;
    </button>
  </div>
</div>

<style>
  .tab-favicon {
    margin-right: 6px;
    display: flex;
    align-items: center;
    color: var(--text-muted);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .tab.drag-over {
    border-left: 2px solid var(--accent-color);
  }
</style>
