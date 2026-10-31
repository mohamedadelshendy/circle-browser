<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
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
  const initialMode = (typeof localStorage !== "undefined" && (localStorage.getItem("circle_sidebar_mode") as "docked" | "auto_hide")) || "docked";
  const initialPos = (typeof localStorage !== "undefined" && (localStorage.getItem("circle_sidebar_position") as "left" | "right")) || "left";

  let sidebarMode: "docked" | "auto_hide" = initialMode;
  let sidebarPosition: "left" | "right" = initialPos;
  let isOverlayVisible = false;
  let hideTimeout: number | null = null;
  let closeAnimationTimeout: number | null = null;

  let showMenu = false;
  let showShields = false;

  // Search input & suggestions state
  let searchVal = "";
  let isSearchFocused = false;
  let selectedIndex = -1;

  interface Suggestion {
    type: "search" | "url" | "quick";
    title: string;
    subtitle: string;
    target: string;
    icon: "search" | "url" | "google" | "github" | "youtube" | "wikipedia" | "reddit";
  }

  const quickSites: Suggestion[] = [
    { type: "quick", title: "Google Search", subtitle: "google.com", target: "https://www.google.com", icon: "google" },
    { type: "quick", title: "GitHub", subtitle: "github.com · Where the world builds", target: "https://github.com", icon: "github" },
    { type: "quick", title: "YouTube", subtitle: "youtube.com · Videos and Music", target: "https://youtube.com", icon: "youtube" },
    { type: "quick", title: "Wikipedia", subtitle: "wikipedia.org · The Free Encyclopedia", target: "https://wikipedia.org", icon: "wikipedia" },
    { type: "quick", title: "Reddit", subtitle: "reddit.com · Dive into anything", target: "https://reddit.com", icon: "reddit" },
  ];

  $: suggestions = computeSuggestions(searchVal);

  function computeSuggestions(query: string): Suggestion[] {
    const q = query.trim();
    const qLower = q.toLowerCase();
    if (!q) {
      return quickSites;
    }

    const list: Suggestion[] = [];

    // 1. Google Search
    list.push({
      type: "search",
      title: `Search Google for "${q}"`,
      subtitle: "Google Search",
      target: q,
      icon: "search",
    });

    // 2. Direct URL if it looks like a domain or path
    const hasSpaces = q.includes(" ");
    const hasDot = q.includes(".");
    const isLocalhost = qLower.startsWith("localhost");
    const isHttp = qLower.startsWith("http://") || qLower.startsWith("https://");

    if (!hasSpaces && (hasDot || isLocalhost || isHttp)) {
      const formatted = isHttp ? q : `https://${q}`;
      list.push({
        type: "url",
        title: `Navigate to ${formatted}`,
        subtitle: "Direct URL",
        target: formatted,
        icon: "url",
      });
    }

    // 3. Filter matching quick sites
    quickSites.forEach((s) => {
      if (
        s.title.toLowerCase().includes(qLower) ||
        s.subtitle.toLowerCase().includes(qLower) ||
        s.target.toLowerCase().includes(qLower)
      ) {
        list.push(s);
      }
    });

    return list.slice(0, 6);
  }

  // Synchronize input with active tab url when not typing
  $: if (!isSearchFocused && activeTab) {
    if (
      !activeTab.url ||
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

  onMount(async () => {
    try {
      const savedPos = localStorage.getItem("circle_sidebar_position") as "left" | "right" | null;
      if (savedPos && (savedPos === "left" || savedPos === "right")) {
        sidebarPosition = savedPos;
        await browserBridge.setSidebarPosition(savedPos);
      }
      const savedMode = localStorage.getItem("circle_sidebar_mode") as "docked" | "auto_hide" | null;
      if (savedMode && (savedMode === "docked" || savedMode === "auto_hide")) {
        sidebarMode = savedMode;
        await browserBridge.setSidebarMode(savedMode);
      }
    } catch (err) {
      console.error("Failed to restore sidebar preferences:", err);
    }
  });

  // Toggle between Docked (side-by-side) and Auto-Hide (disappears & website fills full screen)
  async function handleToggleDockMode() {
    if (sidebarMode === "docked") {
      sidebarMode = "auto_hide";
      isOverlayVisible = false;
      localStorage.setItem("circle_sidebar_mode", "auto_hide");
      await browserBridge.setSidebarMode("auto_hide");
    } else {
      sidebarMode = "docked";
      isOverlayVisible = false;
      localStorage.setItem("circle_sidebar_mode", "docked");
      await browserBridge.setSidebarMode("docked");
    }
  }

  // Auto-hide edge trigger hover handler: expands sticky floating sidebar
  async function handleMouseEnter() {
    if (sidebarMode !== "auto_hide") return;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
    if (closeAnimationTimeout) {
      clearTimeout(closeAnimationTimeout);
      closeAnimationTimeout = null;
    }
    if (!isOverlayVisible) {
      await browserBridge.setSidebarOverlay(true);
      isOverlayVisible = true;
    }
  }

  // Auto-hide leave handler: smooth grace period (400ms) before fading out
  function handleSidebarMouseLeave() {
    if (sidebarMode !== "auto_hide") return;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
    }
    hideTimeout = window.setTimeout(() => {
      isOverlayVisible = false;
      closeAnimationTimeout = window.setTimeout(async () => {
        if (!isOverlayVisible) {
          await browserBridge.setSidebarOverlay(false);
        }
        closeAnimationTimeout = null;
      }, 340);
      hideTimeout = null;
    }, 400);
  }

  // Toggle sidebar position between LEFT and RIGHT side of the browser
  async function handleToggleSidebarPosition() {
    const newPos = await browserBridge.toggleSidebarPosition();
    sidebarPosition = newPos === "right" ? "right" : "left";
    localStorage.setItem("circle_sidebar_position", sidebarPosition);
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
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }
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
      const from = draggedIndex;
      const to = targetIndex;
      // Instant optimistic UI update
      const updated = [...tabs];
      const [moved] = updated.splice(from, 1);
      updated.splice(to, 0, moved);
      tabs = updated;
      await browserBridge.reorderTabs(from, to);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }

  async function handleMoveTab(fromIndex: number, delta: number) {
    const toIndex = fromIndex + delta;
    if (toIndex >= 0 && toIndex < tabs.length) {
      const updated = [...tabs];
      const [moved] = updated.splice(fromIndex, 1);
      updated.splice(toIndex, 0, moved);
      tabs = updated;
      await browserBridge.reorderTabs(fromIndex, toIndex);
    }
  }
</script>

<div
  class="sidebar-root-wrapper"
  role="region"
  aria-label="Circle Browser Sidebar"
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleSidebarMouseLeave}
>
  <!-- Invisible Edge Trigger Strip when in Auto-Hide Mode and sidebar is hidden -->
  {#if sidebarMode === "auto_hide" && !isOverlayVisible}
    <div
      class="edge-trigger-strip position-{sidebarPosition}"
      on:mouseenter={handleMouseEnter}
      role="presentation"
      title="Hover to reveal Circle Sidebar"
    ></div>
  {/if}

  <!-- Sidebar Panel: Either Docked or Floating Sticky Overlay -->
  <aside
    class="circle-sidebar {sidebarMode === 'docked' ? 'docked' : 'floating-overlay'} position-{sidebarPosition} {isOverlayVisible ? 'visible' : ''}"
    on:mouseenter={handleMouseEnter}
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
          class="icon-btn nav-btn"
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
          class="icon-btn nav-btn"
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
          class="icon-btn nav-btn"
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

    <!-- Omnibox / Search Box -->
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
          placeholder="Search or enter URL..."
          bind:value={searchVal}
          on:focus={(e) => {
            isSearchFocused = true;
            selectedIndex = -1;
            e.currentTarget.select();
          }}
          on:blur={() => setTimeout(() => (isSearchFocused = false), 250)}
          on:keydown={handleSearchKeydown}
          spellcheck="false"
          autocomplete="off"
        />

        {#if searchVal.trim()}
          <button
            type="button"
            class="search-clear-btn"
            title="Clear"
            on:pointerdown|preventDefault={() => {
              searchVal = "";
              selectedIndex = -1;
            }}
          >
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        {/if}
      </div>

      <!-- Professional Autocomplete Suggestions Dropdown -->
      {#if isSearchFocused && suggestions.length > 0}
        <div class="sidebar-suggestions-dropdown">
          {#each suggestions as item, idx}
            <div
              class="suggestion-item {selectedIndex === idx ? 'selected' : ''}"
              role="button"
              tabindex="0"
              on:pointerdown|preventDefault={() => handleSearchSubmit(item.target)}
              on:mouseenter={() => (selectedIndex = idx)}
            >
              <span class="suggestion-icon">
                {#if item.icon === "google"}
                  <svg viewBox="0 0 24 24" width="15" height="15">
                    <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                    <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                    <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                    <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
                  </svg>
                {:else if item.icon === "github"}
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
                  </svg>
                {:else if item.icon === "youtube"}
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="#FF0000">
                    <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                  </svg>
                {:else if item.icon === "wikipedia"}
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12.09 13.119l2.793-6.903h1.867l-3.805 9.245-2.073-5.068-2.074 5.068L4.993 6.216h1.867l2.793 6.903 1.218-3.016-1.042-2.585h1.722l.859 2.129.859-2.129h1.722l-1.042 2.585 1.218 3.016h-.097zm8.47-6.903l-3.328 8.086-1.39-3.447.886-2.197h-1.637l-.707 1.753-1.649-4.088-.86 2.13h-1.72l1.042-2.585-2.074-5.068h1.867l2.793 6.903 1.218-3.016-1.042-2.585h1.722l.859 2.129.859-2.129h1.722l-1.042 2.585 1.218 3.016 1.867-4.608h1.867l.18 4.608z"/>
                  </svg>
                {:else if item.icon === "reddit"}
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="#FF4500">
                    <circle cx="12" cy="12" r="10"/>
                    <path fill="#fff" d="M12 7.5a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm4.5 3a1.5 1.5 0 1 0-1.5 1.5c.01-.01.01-.02 0 0-.2 1.3-1.5 2.3-3 2.3s-2.8-1-3-2.3a1.5 1.5 0 1 0-1.5-1.5 1.5 1.5 0 0 0 1.2.7c0 .1 0 .2 0 .3 0 1.8 2 3.3 4.5 3.3s4.5-1.5 4.5-3.3c0-.1 0-.2 0-.3a1.5 1.5 0 0 0 1.3-.7z"/>
                  </svg>
                {:else if item.type === "url"}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#60a5fa" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="2" y1="12" x2="22" y2="12"></line>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                  </svg>
                {:else}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#a1a1aa" stroke-width="2">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                  </svg>
                {/if}
              </span>

              <div class="suggestion-content">
                <span class="suggestion-title">{item.title}</span>
                <span class="suggestion-sub">{item.subtitle}</span>
              </div>

              {#if selectedIndex === idx}
                <span class="suggestion-enter-badge">↵</span>
              {/if}
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

    <!-- Vertical Tabs List with Drag & Drop and Instant Reordering -->
    <div class="sidebar-tabs-list" role="tablist">
      {#each tabs as tab, index (tab.id)}
        <div
          class="sidebar-tab-item {tab.id === activeTabId ? 'active' : ''} {dragOverIndex === index ? 'drag-over' : ''} {draggedIndex === index ? 'dragging' : ''}"
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
            {:else if tab.url && tab.url.includes("google.com")}
              <svg viewBox="0 0 24 24" width="15" height="15">
                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
              </svg>
            {:else if tab.url && tab.url.includes("github.com")}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
              </svg>
            {:else if tab.url && tab.url.includes("youtube.com")}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="#FF0000">
                <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
              </svg>
            {:else if tab.url && tab.url.includes("wikipedia.org")}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12.09 13.119l2.793-6.903h1.867l-3.805 9.245-2.073-5.068-2.074 5.068L4.993 6.216h1.867l2.793 6.903 1.218-3.016-1.042-2.585h1.722l.859 2.129.859-2.129h1.722l-1.042 2.585 1.218 3.016h-.097zm8.47-6.903l-3.328 8.086-1.39-3.447.886-2.197h-1.637l-.707 1.753-1.649-4.088-.86 2.13h-1.72l1.042-2.585-2.074-5.068h1.867l2.793 6.903 1.218-3.016-1.042-2.585h1.722l.859 2.129.859-2.129h1.722l-1.042 2.585 1.218 3.016 1.867-4.608h1.867l.18 4.608z"/>
              </svg>
            {:else if tab.url && tab.url.includes("reddit.com")}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="#FF4500">
                <circle cx="12" cy="12" r="10"/>
                <path fill="#fff" d="M12 7.5a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm4.5 3a1.5 1.5 0 1 0-1.5 1.5c.01-.01.01-.02 0 0-.2 1.3-1.5 2.3-3 2.3s-2.8-1-3-2.3a1.5 1.5 0 1 0-1.5-1.5 1.5 1.5 0 0 0 1.2.7c0 .1 0 .2 0 .3 0 1.8 2 3.3 4.5 3.3s4.5-1.5 4.5-3.3c0-.1 0-.2 0-.3a1.5 1.5 0 0 0 1.3-.7z"/>
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

          <!-- Tab Action Controls: Reorder Up/Down + Close Button -->
          <div class="tab-actions">
            {#if index > 0}
              <button
                type="button"
                class="tab-reorder-btn"
                title="Move tab up (Ctrl+Shift+Up)"
                on:click|stopPropagation={() => handleMoveTab(index, -1)}
                aria-label="Move tab up"
              >
                ▲
              </button>
            {/if}
            {#if index < tabs.length - 1}
              <button
                type="button"
                class="tab-reorder-btn"
                title="Move tab down (Ctrl+Shift+Down)"
                on:click|stopPropagation={() => handleMoveTab(index, 1)}
                aria-label="Move tab down"
              >
                ▼
              </button>
            {/if}

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
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="11 17 6 12 11 7"/>
                <polyline points="18 17 13 12 18 7"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="13 17 18 12 13 7"/>
                <polyline points="6 17 11 12 6 7"/>
              </svg>
            {/if}
          {:else}
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

<style>
  .search-clear-btn {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 50%;
    color: #8e8e96;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
    margin-left: 4px;
    transition: background 0.15s, color 0.15s;
  }

  .search-clear-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  .sidebar-suggestions-dropdown {
    position: absolute;
    top: 42px;
    left: 8px;
    right: 8px;
    background: #1c1c1f;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 10px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.75), 0 2px 8px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    padding: 4px;
    z-index: 1000;
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    padding: 7px 10px;
    border-radius: 7px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
    user-select: none;
    -webkit-user-select: none;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: rgba(255, 255, 255, 0.08);
  }

  .suggestion-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    margin-right: 10px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .suggestion-item:hover .suggestion-icon,
  .suggestion-item.selected .suggestion-icon {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .suggestion-content {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }

  .suggestion-title {
    font-size: 12.5px;
    font-weight: 500;
    color: #f4f4f6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.25;
  }

  .suggestion-sub {
    font-size: 11px;
    color: #8e8e96;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
    margin-top: 1px;
  }

  .suggestion-enter-badge {
    font-size: 11px;
    color: #71717a;
    padding: 2px 5px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    margin-left: 6px;
    flex-shrink: 0;
  }

  .tab-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
    flex-shrink: 0;
  }

  .tab-reorder-btn {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: #71717a;
    font-size: 8px;
    cursor: pointer;
    opacity: 0;
    padding: 0;
    transition: opacity 0.15s, background 0.15s, color 0.15s;
  }

  .sidebar-tab-item:hover .tab-reorder-btn {
    opacity: 0.8;
  }

  .tab-reorder-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
    opacity: 1 !important;
  }

  .sidebar-tab-item.dragging {
    opacity: 0.45;
    transform: scale(0.98);
  }

  .sidebar-tab-item.drag-over {
    border-top: 2px solid #3b82f6;
    background: rgba(59, 130, 246, 0.15);
  }

  .nav-btn:active {
    transform: scale(0.92);
  }
</style>
