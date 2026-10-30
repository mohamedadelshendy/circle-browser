<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Omnibox from "./Omnibox.svelte";
  import ShieldsPopup from "./ShieldsPopup.svelte";
  import MenuPopup from "./MenuPopup.svelte";
  import { browserBridge } from "../lib/tauri-bridge";
  import type { Tab } from "../lib/types";

  export let activeTab: Tab | null = null;

  const dispatch = createEventDispatcher<{
    back: void;
    forward: void;
    reload: void;
    navigate: string;
  }>();

  let isBookmarked = false;
  let showShields = false;
  let showMenu = false;

  async function toggleShields() {
    showShields = !showShields;
    showMenu = false;
    if (showShields) {
      await browserBridge.expandChrome(340);
    } else {
      await browserBridge.collapseChrome();
    }
  }

  async function toggleMenu() {
    showMenu = !showMenu;
    showShields = false;
    if (showMenu) {
      await browserBridge.expandChrome(320);
    } else {
      await browserBridge.collapseChrome();
    }
  }

  function handleClosePopups() {
    showShields = false;
    showMenu = false;
    browserBridge.collapseChrome();
  }

  function toggleBookmark() {
    isBookmarked = !isBookmarked;
  }
</script>

<div class="nav-bar">
  <!-- Back Button -->
  <button
    class="nav-btn"
    title="Click to go back, hold to see history"
    on:click={() => dispatch("back")}
    aria-label="Back"
  >
    &lsaquo;
  </button>

  <!-- Forward Button -->
  <button
    class="nav-btn"
    title="Click to go forward, hold to see history"
    on:click={() => dispatch("forward")}
    aria-label="Forward"
  >
    &rsaquo;
  </button>

  <!-- Reload / Stop Button -->
  <button
    class="nav-btn"
    title={activeTab?.is_loading ? "Stop loading" : "Reload this page (Ctrl+R)"}
    on:click={() => dispatch("reload")}
    aria-label="Reload"
  >
    {#if activeTab?.is_loading}
      &times;
    {:else}
      &#x21bb;
    {/if}
  </button>

  <!-- Bookmark Ribbon -->
  <button
    class="nav-btn bookmark {isBookmarked ? 'active' : ''}"
    title={isBookmarked ? "Remove bookmark" : "Bookmark this tab (Ctrl+D)"}
    on:click={toggleBookmark}
    aria-label="Bookmark"
  >
    {isBookmarked ? "★" : "☆"}
  </button>

  <!-- Centered Omnibox -->
  <Omnibox
    url={activeTab?.url || ""}
    isLoading={activeTab?.is_loading || false}
    on:navigate={(e) => dispatch("navigate", e.detail)}
  />

  <!-- Right Actions matching mockup -->
  <div class="action-controls">
    <!-- Shields Protection Toggle -->
    <button
      class="action-btn shield-btn {showShields ? 'active' : ''}"
      title="Circle Shields: trackers & ads blocked"
      on:click={toggleShields}
      aria-label="Circle Shields"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
        <circle cx="12" cy="12" r="4" fill="currentColor"/>
      </svg>
    </button>

    <!-- Media / Audio Controls -->
    <button class="action-btn" title="Control your music, videos, and more" aria-label="Media">
      &#x266A;
    </button>

    <!-- Hamburger Menu -->
    <button
      class="action-btn menu-btn {showMenu ? 'active' : ''}"
      title="Customize and control Circle"
      on:click={toggleMenu}
      aria-label="Menu"
    >
      &#x22EE;
    </button>
  </div>

  <!-- Popups -->
  {#if showShields}
    <ShieldsPopup
      activeTabId={activeTab?.id || null}
      siteUrl={activeTab?.url || ""}
      on:close={handleClosePopups}
    />
  {/if}

  {#if showMenu}
    <MenuPopup
      on:close={handleClosePopups}
    />
  {/if}
</div>

<style>
  .nav-btn.bookmark.active {
    color: #fbbf24;
  }

  .action-btn.active {
    background: var(--btn-hover);
    color: var(--accent-color);
  }

  .shield-btn {
    color: var(--accent-color);
  }
</style>

