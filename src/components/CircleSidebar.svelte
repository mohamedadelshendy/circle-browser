<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";
  import type { Tab } from "../lib/types";
  import ShieldsPopup from "./ShieldsPopup.svelte";
  import MenuPopup from "./MenuPopup.svelte";

  export let tabs: Tab[] = [];
  export let activeTabId: string | null = null;
  export let activeTab: Tab | null = null;

  const dispatch = createEventDispatcher<{
    newTab: void;
    switchTab: string;
    closeTab: string;
    navigate: string;
    back: void;
    forward: void;
    reload: void;
  }>();

  // Sidebar modes & position: left vs right, docked vs auto_hide
  let sidebarMode: "docked" | "auto_hide" = "docked";
  let sidebarPosition: "left" | "right" = "left";
  let isOverlayVisible = false;
  let hideTimeout: number | null = null;

  let showMenu = false;
  let showShields = false;

  // Search input & suggestions state
  let searchVal = "";
  let isSearchFocused = false;
  let selectedIndex = -1;

  interface Suggestion {
    type: "search" | "url" | "quick";
    title: string;
    target: string;
  }

  const quickSites: Suggestion[] = [
    { type: "quick", title: "Google Search", target: "https://www.google.com" },
    { type: "quick", title: "GitHub - Where the world builds", target: "https://github.com" },
    { type: "quick", title: "YouTube - Videos and Music", target: "https://youtube.com" },
    { type: "quick", title: "Wikipedia - The Free Encyclopedia", target: "https://wikipedia.org" },
  ];

  $: suggestions = computeSuggestions(searchVal);

  function computeSuggestions(query: string): Suggestion[] {
    const q = query.trim().toLowerCase();
    if (!q) {
      return quickSites;
    }

    const list: Suggestion[] = [
      {
        type: "search",
        title: `Search Google for "${query}"`,
        target: query,
      },
    ];

    if (!q.includes(" ") && (q.includes(".") || q.startsWith("localhost"))) {
      const formatted = q.startsWith("http") ? q : `https://${q}`;
      list.push({
        type: "url",
        title: `Navigate to ${formatted}`,
        target: formatted,
      });
    }

    quickSites.forEach((s) => {
      if (s.title.toLowerCase().includes(q) || s.target.toLowerCase().includes(q)) {
        list.push(s);
      }
    });

    return list.slice(0, 5);
  }

  // Synchronize input with active tab url when not typing
  $: if (!isSearchFocused && activeTab) {
    if (
      activeTab.url === "browser://newtab" ||
      activeTab.url.includes("newtab.html") ||
      activeTab.url === "about:blank"
    ) {
      searchVal = "";
    } else {
      searchVal = activeTab.url;
    }
  }

  function handleSearchSubmit(targetUrl: string) {
    if (!targetUrl.trim()) return;
    dispatch("navigate", targetUrl.trim());
    isSearchFocused = false;
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
        handleSearchSubmit(suggestions[selectedIndex].target);
      } else {
        handleSearchSubmit(searchVal);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % suggestions.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length;
    } else if (e.key === "Escape") {
      isSearchFocused = false;
    }
  }

  // Toggle between Docked (side-by-side) and Auto-Hide (disappears & website fills full screen)
  async function handleToggleDockMode() {
    if (sidebarMode === "docked") {
      sidebarMode = "auto_hide";
      isOverlayVisible = false;
      await browserBridge.setSidebarMode("auto_hide");
    } else {
      sidebarMode = "docked";
      isOverlayVisible = false;
      await browserBridge.setSidebarMode("docked");
    }
  }

  // Auto-hide edge trigger hover handler: expands sticky floating sidebar
  async function handleEdgeMouseEnter() {
    if (sidebarMode !== "auto_hide") return;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
    await browserBridge.setSidebarOverlay(true);
    isOverlayVisible = true;
  }

  // Auto-hide leave handler: smoothly tucks floating sidebar off-screen
  function handleSidebarMouseLeave() {
    if (sidebarMode !== "auto_hide") return;
    isOverlayVisible = false;
    hideTimeout = window.setTimeout(async () => {
      await browserBridge.setSidebarOverlay(false);
      hideTimeout = null;
    }, 240);
  }

  // Toggle sidebar position between LEFT and RIGHT side of the browser
  async function handleToggleSidebarPosition() {
    const newPos = await browserBridge.toggleSidebarPosition();
    sidebarPosition = newPos === "right" ? "right" : "left";
  }

  // Drag and drop tab reordering
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

<div class="sidebar-root-wrapper">
  <!-- Invisible 12px Edge Trigger Strip when in Auto-Hide Mode -->
  {#if sidebarMode === "auto_hide"}
    <div
      class="edge-trigger-strip position-{sidebarPosition}"
      on:mouseenter={handleEdgeMouseEnter}
      role="presentation"
      title="Hover to reveal Circle Sidebar"
    ></div>
  {/if}

  <!-- Sidebar Panel: Either Docked or Floating Sticky Overlay (Zen Image 2) -->
  <aside
    class="circle-sidebar {sidebarMode === 'docked' ? 'docked' : 'floating-overlay'} position-{sidebarPosition} {isOverlayVisible ? 'visible' : ''}"
    on:mouseleave={handleSidebarMouseLeave}
    on:mouseenter={handleEdgeMouseEnter}
  >
    <!-- Top Bar / Window Drag & Navigation Controls (Height: 42px) -->
    <header class="sidebar-top-bar" data-tauri-drag-region>
      <!-- Left Actions: Menu, Position Toggle (Left/Right), Dock/Pin, Maximize -->
      <div class="top-bar-group">
        <!-- 3-dots Menu -->
        <button
          type="button"
          class="icon-btn"
          title="Menu (Alt+F)"
          on:click={() => (showMenu = !showMenu)}
          aria-label="Menu"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
            <circle cx="5" cy="12" r="2"/>
            <circle cx="12" cy="12" r="2"/>
            <circle cx="19" cy="12" r="2"/>
          </svg>
        </button>

        <!-- Sidebar Position Left/Right Toggle ([|]) -->
        <button
          type="button"
          class="icon-btn"
          title="Switch sidebar to {sidebarPosition === 'left' ? 'Right' : 'Left'} side"
          on:click={handleToggleSidebarPosition}
          aria-label="Toggle Sidebar Position"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            {#if sidebarPosition === 'left'}
              <line x1="9" y1="3" x2="9" y2="21"/>
            {:else}
              <line x1="15" y1="3" x2="15" y2="21"/>
            {/if}
          </svg>
        </button>

        <!-- Pin / Dock Toggle Button -->
        <button
          type="button"
          class="icon-btn {sidebarMode === 'docked' ? 'active-pinned' : ''}"
          title={sidebarMode === 'docked' ? "Shrink sidebar (Auto-Hide, full screen website)" : "Dock sidebar side-by-side"}
          on:click={handleToggleDockMode}
          aria-label="Toggle Dock Mode"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="17" x2="12" y2="22"/>
            <path d="M5 17h14v-2l-3-3V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v7l-3 3v2z"/>
          </svg>
        </button>

        <!-- Maximize / Fullscreen Window -->
        <button
          type="button"
          class="icon-btn"
          title="Maximize / Restore window"
          on:click={() => browserBridge.maximizeWindow()}
          aria-label="Maximize"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9"/>
            <polyline points="9 21 3 21 3 15"/>
            <line x1="21" y1="3" x2="14" y2="10"/>
            <line x1="3" y1="21" x2="10" y2="14"/>
          </svg>
        </button>
      </div>

      <!-- Right Navigation: Back, Forward, Reload -->
      <div class="top-bar-group">
        <!-- Back -->
        <button
          type="button"
          class="icon-btn"
          title="Back (Alt+Left)"
          on:click={() => dispatch("back")}
          aria-label="Back"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M15 18l-6-6 6-6"/>
          </svg>
        </button>

        <!-- Forward -->
        <button
          type="button"
          class="icon-btn"
          title="Forward (Alt+Right)"
          on:click={() => dispatch("forward")}
          aria-label="Forward"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 18l6-6-6-6"/>
          </svg>
        </button>

        <!-- Reload / Stop -->
        <button
          type="button"
          class="icon-btn"
          title={activeTab?.is_loading ? "Stop loading" : "Reload (Ctrl+R)"}
          on:click={() => dispatch("reload")}
          aria-label="Reload"
        >
          {#if activeTab?.is_loading}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21.5 2v6h-6M2.5 22v-6h6"/>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L21.5 8M2.5 16l3.14 2.36A9 9 0 0 0 20.49 15"/>
            </svg>
          {/if}
        </button>
      </div>
    </header>

    <!-- Omnibox / Search Pill: [🔍 Search...] -->
    <div class="sidebar-search-container">
      <div class="search-box {isSearchFocused ? 'focused' : ''}">
        <span class="search-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </span>

        <input
          type="text"
          class="search-input"
          placeholder="Search..."
          bind:value={searchVal}
          on:focus={() => (isSearchFocused = true)}
          on:blur={() => setTimeout(() => (isSearchFocused = false), 200)}
          on:keydown={handleSearchKeydown}
          spellcheck="false"
          autocomplete="off"
        />
      </div>

      <!-- Autocomplete Suggestions Dropdown -->
      {#if isSearchFocused && suggestions.length > 0}
        <div class="sidebar-suggestions-dropdown">
          {#each suggestions as item, idx}
            <div
              class="suggestion-item {selectedIndex === idx ? 'selected' : ''}"
              role="button"
              tabindex="0"
              on:mousedown={() => handleSearchSubmit(item.target)}
              on:keydown={(e) => (e.key === "Enter" || e.key === " ") && handleSearchSubmit(item.target)}
            >
              <span class="suggestion-icon">
                {#if item.type === "search"}
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                  </svg>
                {:else if item.type === "url"}
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="2" y1="12" x2="22" y2="12"></line>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                  </svg>
                {:else}
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                {/if}
              </span>
              <span class="suggestion-text">{item.title}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Workspace Space Header -->
    <div class="space-header">
      <span class="space-label">Space</span>
    </div>

    <!-- + New Tab Action Button -->
    <button
      type="button"
      class="new-tab-action-btn"
      title="New tab (Ctrl+T)"
      on:click={() => dispatch("newTab")}
      aria-label="New tab"
    >
      <span class="new-tab-plus-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </span>
      <span class="new-tab-text">New Tab</span>
    </button>

    <!-- Vertical Tabs List -->
    <div class="sidebar-tabs-list" role="tablist">
      {#each tabs as tab, index (tab.id)}
        <div
          class="sidebar-tab-item {tab.id === activeTabId ? 'active' : ''} {dragOverIndex === index ? 'drag-over' : ''}"
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
          title={tab.title || "New tab"}
        >
          <!-- Favicon / Loading state -->
          <span class="tab-favicon">
            {#if tab.is_loading}
              <span class="tab-spinner"></span>
            {:else if tab.url.includes("google.com")}
              <svg viewBox="0 0 24 24" width="15" height="15">
                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="2" y1="12" x2="22" y2="12"></line>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
              </svg>
            {/if}
          </span>

          <!-- Title -->
          <span class="tab-title">
            {tab.title || "New tab"}
          </span>

          <!-- Close Tab (×) -->
          <button
            type="button"
            class="tab-close-btn"
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
    </div>

    <!-- Sidebar Bottom Toolbar / Footer (Height: 42px) -->
    <footer class="sidebar-footer">
      <!-- Left Footer Controls: Downloads / Package icon, Shields -->
      <div class="footer-group">
        <!-- Package / Downloads icon -->
        <button
          type="button"
          class="icon-btn"
          title="Downloads (Ctrl+J)"
          aria-label="Downloads"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
            <line x1="12" y1="22.08" x2="12" y2="12"/>
          </svg>
        </button>

        <!-- Shields Privacy Icon -->
        <button
          type="button"
          class="icon-btn"
          title="Circle Shields"
          on:click={() => (showShields = !showShields)}
          aria-label="Circle Shields"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          </svg>
        </button>
      </div>

      <!-- Right Footer Controls: + New Space, Shrink/Collapse to Auto-Hide -->
      <div class="footer-group">
        <!-- New Space (+) Button -->
        <button
          type="button"
          class="icon-btn"
          title="New Space"
          on:click={() => dispatch("newTab")}
          aria-label="New Space"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>

        <!-- Shrink / Collapse Button: Hides sidebar & makes website full screen -->
        <button
          type="button"
          class="icon-btn"
          title={sidebarMode === 'docked' ? "Shrink sidebar (Website full screen)" : "Pin sidebar"}
          on:click={handleToggleDockMode}
          aria-label="Toggle Shrink Mode"
        >
          {#if sidebarMode === 'docked'}
            {#if sidebarPosition === 'left'}
              <!-- Point left to collapse -->
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="11 17 6 12 11 7"/>
                <polyline points="18 17 13 12 18 7"/>
              </svg>
            {:else}
              <!-- Point right to collapse -->
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="13 17 18 12 13 7"/>
                <polyline points="6 17 11 12 6 7"/>
              </svg>
            {/if}
          {:else}
            <!-- Pin / Expand icon -->
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="17" x2="12" y2="22"/>
              <path d="M5 17h14v-2l-3-3V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v7l-3 3v2z"/>
            </svg>
          {/if}
        </button>
      </div>
    </footer>

    <!-- Popups -->
    {#if showShields}
      <ShieldsPopup
        activeTabId={activeTab?.id || null}
        siteUrl={activeTab?.url || ""}
        on:close={() => (showShields = false)}
      />
    {/if}

    {#if showMenu}
      <MenuPopup
        on:close={() => (showMenu = false)}
      />
    {/if}
  </aside>
</div>
