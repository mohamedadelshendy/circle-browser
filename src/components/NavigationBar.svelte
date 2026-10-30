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
  <!-- Left Navigation Controls (Back, Forward, Reload, Bookmarks) -->
  <div class="nav-controls-group">
    <!-- Back Button (32x32px circular, 16px icon) -->
    <button
      class="nav-circle-btn"
      title="Click to go back"
      on:click={() => dispatch("back")}
      aria-label="Back"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M15 18l-6-6 6-6"/>
      </svg>
    </button>

    <!-- Forward Button (32x32px circular, 16px icon) -->
    <button
      class="nav-circle-btn"
      title="Click to go forward"
      on:click={() => dispatch("forward")}
      aria-label="Forward"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 18l6-6-6-6"/>
      </svg>
    </button>

    <!-- Reload Button (32x32px circular, 16px icon) -->
    <button
      class="nav-circle-btn"
      title={activeTab?.is_loading ? "Stop loading" : "Reload this page (Ctrl+R)"}
      on:click={() => dispatch("reload")}
      aria-label="Reload"
    >
      {#if activeTab?.is_loading}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M2.5 22v-6h6"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L21.5 8M2.5 16l3.14 2.36A9 9 0 0 0 20.49 15"/>
        </svg>
      {/if}
    </button>

    <!-- Bookmarks Button (32x32px circular, 16px icon) -->
    <button
      class="nav-circle-btn bookmark {isBookmarked ? 'active' : ''}"
      title={isBookmarked ? "Remove bookmark" : "Bookmark this tab (Ctrl+D)"}
      on:click={toggleBookmark}
      aria-label="Bookmark"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill={isBookmarked ? "#fbbf24" : "none"} stroke="currentColor" stroke-width="2" stroke-linejoin="round">
        <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
      </svg>
    </button>
  </div>

  <!-- Center Omnibox / URL Bar -->
  <Omnibox
    url={activeTab?.url || ""}
    isLoading={activeTab?.is_loading || false}
    on:navigate={(e) => dispatch("navigate", e.detail)}
    on:toggleShields={toggleShields}
  />

  <!-- Right Controls: Media/Audio, Extensions puzzle piece, and 3-dots Menu button (each 32x32px) -->
  <div class="right-controls-group">
    <!-- Media / Audio Control Button -->
    <button
      class="tool-circle-btn"
      title="Control your music, videos, and more"
      aria-label="Media"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3"/>
        <circle cx="18" cy="16" r="3"/>
      </svg>
    </button>

    <!-- Extensions Puzzle Piece Button -->
    <button
      class="tool-circle-btn"
      title="Extensions"
      aria-label="Extensions"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M20 14.5a2 2 0 0 0-2-2H16V10a2 2 0 0 0-2-2h-1.5a1.5 1.5 0 0 0 0-3H11a2 2 0 0 0-2 2v2.5H6.5a2 2 0 0 0-2 2v1.5a1.5 1.5 0 0 0 3 0V16a2 2 0 0 0 2 2h2.5v1.5a1.5 1.5 0 0 0 3 0V18a2 2 0 0 0 2-2h1.5a2 2 0 0 0 2-2v-1.5z"/>
      </svg>
    </button>

    <!-- 3-dots Menu Button -->
    <button
      class="tool-circle-btn menu-btn {showMenu ? 'active' : ''}"
      title="Customize and control Circle"
      on:click={toggleMenu}
      aria-label="Menu"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="12" cy="5" r="2"/>
        <circle cx="12" cy="12" r="2"/>
        <circle cx="12" cy="19" r="2"/>
      </svg>
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
