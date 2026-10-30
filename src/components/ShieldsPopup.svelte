<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";

  export let activeTabId: string | null = null;
  export let siteUrl = "";

  const dispatch = createEventDispatcher<{
    close: void;
  }>();

  let shieldsEnabled = true;
  let blockedCount = 0;
  let httpsUpgraded = true;
  let scriptsBlocked = false;

  $: hostname = extractHostname(siteUrl);

  function extractHostname(url: string): string {
    try {
      if (!url || url.startsWith("about:") || url.startsWith("browser:")) {
        return "New Tab";
      }
      return new URL(url).hostname;
    } catch {
      return url || "Current Site";
    }
  }

  onMount(() => {
    if (activeTabId) {
      browserBridge.getShieldStats(activeTabId).then((stats) => {
        shieldsEnabled = stats.shields_enabled;
        blockedCount = stats.blocked_trackers_count;
      }).catch(console.error);
    }

    const unlisten = browserBridge.onShieldStatsUpdated((payload) => {
      if (payload.tab_id === activeTabId) {
        shieldsEnabled = payload.shields_enabled;
        blockedCount = payload.blocked_trackers_count;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function handleToggleShields(e: Event) {
    const input = e.target as HTMLInputElement;
    shieldsEnabled = input.checked;
    if (activeTabId) {
      const stats = await browserBridge.toggleShields(activeTabId, shieldsEnabled);
      blockedCount = stats.blocked_trackers_count;
    }
  }
</script>

<div
  class="shields-overlay"
  role="presentation"
  on:click|self={() => dispatch("close")}
  on:keydown={(e) => e.key === "Escape" && dispatch("close")}
>
  <div class="shields-card">
    <div class="shields-header">
      <div class="brand">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="url(#shieldGrad)" stroke-width="2.5"/>
          <circle cx="12" cy="12" r="5" fill="#818cf8"/>
          <defs>
            <linearGradient id="shieldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#38bdf8"/>
              <stop offset="100%" stop-color="#c084fc"/>
            </linearGradient>
          </defs>
        </svg>
        <span class="title">Circle Shields</span>
      </div>
      <button class="close-btn" on:click={() => dispatch("close")} aria-label="Close">&times;</button>
    </div>

    <div class="site-info">
      <span class="site-name">{hostname}</span>
      <span class="shield-status {shieldsEnabled ? 'up' : 'down'}">
        {shieldsEnabled ? "Shields UP" : "Shields DOWN"}
      </span>
    </div>

    <div class="toggle-row">
      <span>Trackers & ads blocking</span>
      <label class="switch">
        <input
          type="checkbox"
          checked={shieldsEnabled}
          on:change={handleToggleShields}
        />
        <span class="slider"></span>
      </label>
    </div>

    <div class="stat-box">
      <div class="stat-number">{shieldsEnabled ? blockedCount : 0}</div>
      <div class="stat-label">Cross-site trackers & ads blocked</div>
    </div>

    <div class="options-list">
      <div class="opt-row">
        <span>Upgrade connections to HTTPS</span>
        <span class="opt-val">{httpsUpgraded ? "Active" : "Standard"}</span>
      </div>
      <div class="opt-row">
        <span>Block aggressive JavaScript</span>
        <label class="switch small">
          <input type="checkbox" bind:checked={scriptsBlocked} />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </div>
</div>

<style>
  .shields-overlay {
    position: absolute;
    top: 40px;
    right: 36px;
    z-index: 200;
  }

  .shields-card {
    width: 280px;
    background: #1e1b2b;
    border: 1px solid #3b374e;
    border-radius: 12px;
    padding: 14px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    color: var(--text-primary);
    font-size: 13px;
  }

  .shields-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    cursor: pointer;
  }

  .site-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #2e2a3f;
    margin-bottom: 10px;
  }

  .site-name {
    font-weight: 500;
    color: #ffffff;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shield-status.up {
    color: #4ade80;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .shield-status.down {
    color: #ef4444;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .stat-box {
    background: #14121d;
    border-radius: 8px;
    padding: 10px;
    text-align: center;
    margin-bottom: 12px;
  }

  .stat-number {
    font-size: 22px;
    font-weight: 700;
    color: var(--accent-color);
  }

  .stat-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 12px;
  }

  .opt-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--text-muted);
  }

  .opt-val {
    color: #a78bfa;
    font-weight: 500;
  }

  /* Switch */
  .switch {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
  }

  .switch.small {
    width: 30px;
    height: 16px;
  }

  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #3b374e;
    transition: 0.2s;
    border-radius: 20px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.2s;
    border-radius: 50%;
  }

  .switch.small .slider:before {
    height: 10px;
    width: 10px;
    left: 3px;
    bottom: 3px;
  }

  input:checked + .slider {
    background-color: var(--accent-color);
  }

  input:checked + .slider:before {
    transform: translateX(16px);
  }

  .switch.small input:checked + .slider:before {
    transform: translateX(14px);
  }
</style>
