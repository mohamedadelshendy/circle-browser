export interface Tab {
  id: string;
  title: string;
  url: string;
  favicon: string | null;
  is_loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
}

export interface TabStateChangedPayload {
  tabs: Tab[];
  active_tab_id: string | null;
}

export interface NavigationEventPayload {
  tab_id: string;
  url: string;
}

export interface TabLoadingPayload {
  tab_id: string;
  url: string;
  is_loading: boolean;
}

export interface ShieldStatsPayload {
  tab_id: string;
  blocked_trackers_count: number;
  blocked_ads_count: number;
  https_upgrades_count: number;
  shields_enabled: boolean;
}
