import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Tab,
  TabStateChangedPayload,
  NavigationEventPayload,
  TabLoadingPayload,
  ShieldStatsPayload,
} from "./types";

export const browserBridge = {
  // Tab Lifecycle
  openTab: async (url?: string): Promise<Tab> => {
    return await invoke("open_tab", { url });
  },

  closeTab: async (tabId: string): Promise<string | null> => {
    return await invoke("close_tab", { tabId });
  },

  switchTab: async (tabId: string): Promise<void> => {
    return await invoke("switch_tab", { tabId });
  },

  reorderTabs: async (fromIndex: number, toIndex: number): Promise<void> => {
    return await invoke("reorder_tabs", { fromIndex, toIndex });
  },

  // Navigation
  navigate: async (tabId: string, url: string): Promise<void> => {
    return await invoke("navigate", { tabId, url });
  },

  reload: async (tabId: string): Promise<void> => {
    return await invoke("reload", { tabId });
  },

  goBack: async (tabId: string): Promise<void> => {
    return await invoke("go_back", { tabId });
  },

  goForward: async (tabId: string): Promise<void> => {
    return await invoke("go_forward", { tabId });
  },

  getTabs: async (): Promise<TabStateChangedPayload> => {
    return await invoke("get_tabs");
  },

  // Privacy & Shields
  getShieldStats: async (tabId: string): Promise<ShieldStatsPayload> => {
    return await invoke("get_shield_stats", { tabId });
  },

  toggleShields: async (tabId: string, enabled: boolean): Promise<ShieldStatsPayload> => {
    return await invoke("toggle_shields", { tabId, enabled });
  },

  // Dynamic Chrome / Sidebar Control
  setSidebarWidth: async (width: number): Promise<void> => {
    return await invoke("set_sidebar_width", { width });
  },

  expandChrome: async (height: number = 320): Promise<void> => {
    return await invoke("expand_chrome", { height });
  },

  collapseChrome: async (): Promise<void> => {
    return await invoke("collapse_chrome");
  },

  // Window Controls
  minimizeWindow: async (): Promise<void> => {
    return await invoke("minimize_window");
  },

  maximizeWindow: async (): Promise<void> => {
    return await invoke("maximize_window");
  },

  closeWindow: async (): Promise<void> => {
    return await invoke("close_window");
  },

  // Event Subscriptions
  onTabStateChanged: (cb: (payload: TabStateChangedPayload) => void): Promise<UnlistenFn> => {
    return listen<TabStateChangedPayload>("tab-state-changed", (event) => cb(event.payload));
  },

  onTabNavigating: (cb: (payload: NavigationEventPayload) => void): Promise<UnlistenFn> => {
    return listen<NavigationEventPayload>("tab-navigating", (event) => cb(event.payload));
  },

  onTabLoading: (cb: (payload: TabLoadingPayload) => void): Promise<UnlistenFn> => {
    return listen<TabLoadingPayload>("tab-loading", (event) => cb(event.payload));
  },

  onShieldStatsUpdated: (cb: (payload: ShieldStatsPayload) => void): Promise<UnlistenFn> => {
    return listen<ShieldStatsPayload>("shield-stats-updated", (event) => cb(event.payload));
  },
};
