<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";

  const dispatch = createEventDispatcher<{
    close: void;
  }>();

  async function handleNewTab() {
    dispatch("close");
    await browserBridge.openTab();
  }

  function handleClose() {
    dispatch("close");
  }

  function handleExit() {
    dispatch("close");
    browserBridge.closeWindow();
  }
</script>

<div
  class="menu-overlay"
  role="presentation"
  on:click|self={handleClose}
  on:keydown={(e) => e.key === "Escape" && handleClose()}
>
  <div class="menu-card">
    <button class="menu-item" on:click={handleNewTab}>
      <span>New tab</span>
      <span class="shortcut">Ctrl+T</span>
    </button>
    <button class="menu-item" on:click={handleClose}>
      <span>New private window</span>
      <span class="shortcut">Ctrl+Shift+N</span>
    </button>
    <div class="divider"></div>
    <button class="menu-item" on:click={handleClose}>
      <span>History</span>
      <span class="shortcut">Ctrl+H</span>
    </button>
    <button class="menu-item" on:click={handleClose}>
      <span>Bookmarks</span>
      <span class="shortcut">Ctrl+B</span>
    </button>
    <button class="menu-item" on:click={handleClose}>
      <span>Downloads</span>
      <span class="shortcut">Ctrl+J</span>
    </button>
    <div class="divider"></div>
    <button class="menu-item" on:click={handleClose}>
      <span>Settings</span>
      <span class="shortcut">Ctrl+,</span>
    </button>
    <button class="menu-item exit" on:click={handleExit}>
      <span>Exit Titan</span>
      <span class="shortcut">Alt+F4</span>
    </button>
  </div>
</div>

<style>
  .menu-overlay {
    position: absolute;
    top: 40px;
    right: 8px;
    z-index: 200;
  }

  .menu-card {
    width: 220px;
    background: #1e1b2b;
    border: 1px solid #3b374e;
    border-radius: 10px;
    padding: 6px 0;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
  }

  .menu-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 14px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }

  .menu-item:hover {
    background: #2e2a40;
    color: #ffffff;
  }

  .menu-item.exit:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .shortcut {
    font-size: 11px;
    color: var(--text-muted);
  }

  .divider {
    height: 1px;
    background: #2e2a3f;
    margin: 4px 0;
  }
</style>
