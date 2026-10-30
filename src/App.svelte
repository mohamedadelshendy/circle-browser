<script lang="ts">
  import { onMount } from "svelte";
  import TabBar from "./components/TabBar.svelte";
  import NavigationBar from "./components/NavigationBar.svelte";
  import { browserBridge } from "./lib/tauri-bridge";
  import type { Tab } from "./lib/types";

  let tabs: Tab[] = [];
  let activeTabId: string | null = null;

  $: activeTab = tabs.find((t) => t.id === activeTabId) || null;

  onMount(() => {
    // 1. Initial State Fetch
    browserBridge.getTabs().then((state) => {
      tabs = state.tabs;
      activeTabId = state.active_tab_id;
    }).catch(console.error);

    // 2. Listen to tab state synchronization events
    const unlistenState = browserBridge.onTabStateChanged((payload) => {
      tabs = payload.tabs;
      activeTabId = payload.active_tab_id;
    });

    // 3. Listen to navigation events
    const unlistenNav = browserBridge.onTabNavigating((payload) => {
      const target = tabs.find((t) => t.id === payload.tab_id);
      if (target) {
        target.url = payload.url;
        tabs = [...tabs];
      }
    });

    // 4. Listen to loading progress
    const unlistenLoading = browserBridge.onTabLoading((payload) => {
      const target = tabs.find((t) => t.id === payload.tab_id);
      if (target) {
        target.is_loading = payload.is_loading;
        target.url = payload.url;
        tabs = [...tabs];
      }
    });

    // Global keyboard shortcuts
    window.addEventListener("keydown", handleGlobalKeydown);

    return () => {
      unlistenState.then((fn) => fn());
      unlistenNav.then((fn) => fn());
      unlistenLoading.then((fn) => fn());
      window.removeEventListener("keydown", handleGlobalKeydown);
    };
  });

  async function handleGlobalKeydown(e: KeyboardEvent) {
    // Ctrl+T: New Tab
    if (e.ctrlKey && e.key.toLowerCase() === "t") {
      e.preventDefault();
      await handleNewTab();
    }
    // Ctrl+W: Close Tab
    else if (e.ctrlKey && e.key.toLowerCase() === "w") {
      e.preventDefault();
      if (activeTabId) await handleCloseTab(activeTabId);
    }
    // Ctrl+R / F5: Reload
    else if ((e.ctrlKey && e.key.toLowerCase() === "r") || e.key === "F5") {
      e.preventDefault();
      await handleReload();
    }
    // Ctrl+Tab: Switch to Next Tab
    else if (e.ctrlKey && e.key === "Tab") {
      e.preventDefault();
      cycleTabs(e.shiftKey ? -1 : 1);
    }
  }

  function cycleTabs(offset: number) {
    if (tabs.length <= 1 || !activeTabId) return;
    const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
    if (currentIndex === -1) return;
    const nextIndex = (currentIndex + offset + tabs.length) % tabs.length;
    handleSwitchTab(tabs[nextIndex].id);
  }

  async function handleNewTab() {
    await browserBridge.openTab();
  }

  async function handleSwitchTab(tabId: string) {
    await browserBridge.switchTab(tabId);
  }

  async function handleCloseTab(tabId: string) {
    await browserBridge.closeTab(tabId);
  }

  async function handleNavigate(targetUrl: string) {
    if (!activeTabId) return;
    await browserBridge.navigate(activeTabId, targetUrl);
  }

  async function handleBack() {
    if (activeTabId) await browserBridge.goBack(activeTabId);
  }

  async function handleForward() {
    if (activeTabId) await browserBridge.goForward(activeTabId);
  }

  async function handleReload() {
    if (activeTabId) await browserBridge.reload(activeTabId);
  }
</script>

<div class="browser-chrome">
  <!-- Row 1: Draggable Tabs, Brand Logo & Window Controls -->
  <TabBar
    {tabs}
    {activeTabId}
    on:newTab={handleNewTab}
    on:switchTab={(e) => handleSwitchTab(e.detail)}
    on:closeTab={(e) => handleCloseTab(e.detail)}
  />

  <!-- Row 2: Nav Controls, Omnibox, Shields & Actions -->
  <NavigationBar
    {activeTab}
    on:back={handleBack}
    on:forward={handleForward}
    on:reload={handleReload}
    on:navigate={(e) => handleNavigate(e.detail)}
  />
</div>
