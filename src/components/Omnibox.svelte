<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";

  export let url = "";
  export let isLoading = false;

  const dispatch = createEventDispatcher<{
    navigate: string;
  }>();

  let inputVal = "";
  let isFocused = false;
  let selectedIndex = -1;

  interface Suggestion {
    type: "search" | "url" | "quick";
    title: string;
    target: string;
  }

  const quickSites: Suggestion[] = [
    { type: "quick", title: "Google Search", target: "https://www.google.com" },
    { type: "quick", title: "GitHub - Where the world builds software", target: "https://github.com" },
    { type: "quick", title: "YouTube - Videos and Music", target: "https://youtube.com" },
    { type: "quick", title: "Wikipedia - The Free Encyclopedia", target: "https://wikipedia.org" },
  ];

  $: suggestions = computeSuggestions(inputVal);

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

  $: if (!isFocused) {
    if (url === "browser://newtab" || url.includes("newtab.html") || url === "about:blank") {
      inputVal = "";
    } else {
      inputVal = url;
    }
  }

  async function handleFocus() {
    isFocused = true;
    selectedIndex = -1;
    await browserBridge.expandChrome(320);
  }

  async function handleBlur() {
    // Delay collapse to allow click events on suggestions to register
    setTimeout(async () => {
      isFocused = false;
      selectedIndex = -1;
      await browserBridge.collapseChrome();
    }, 200);
  }

  function submit(targetUrl: string) {
    if (!targetUrl.trim()) return;
    dispatch("navigate", targetUrl.trim());
    isFocused = false;
    browserBridge.collapseChrome();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
        submit(suggestions[selectedIndex].target);
      } else {
        submit(inputVal);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % suggestions.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length;
    } else if (e.key === "Escape") {
      isFocused = false;
      browserBridge.collapseChrome();
    }
  }
</script>

<div class="omnibox-container">
  <div class="omnibox-wrapper {isFocused ? 'focused' : ''}">
    <!-- Google 4-Color Icon -->
    <div class="omnibox-icon" title="Search Google">
      <svg viewBox="0 0 24 24" width="14" height="14">
        <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
        <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
        <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
        <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
      </svg>
    </div>

    <input
      type="text"
      class="omnibox-input"
      placeholder="Search Google or type a URL"
      bind:value={inputVal}
      on:focus={handleFocus}
      on:blur={handleBlur}
      on:keydown={handleKeydown}
      spellcheck="false"
      autocomplete="off"
    />

    {#if isLoading}
      <div class="progress-bar"></div>
    {/if}
  </div>

  <!-- Autocomplete Suggestions Dropdown -->
  {#if isFocused && suggestions.length > 0}
    <div class="suggestions-dropdown">
      {#each suggestions as item, idx}
        <div
          class="suggestion-item {selectedIndex === idx ? 'selected' : ''}"
          role="button"
          tabindex="0"
          on:mousedown={() => submit(item.target)}
          on:keydown={(e) => (e.key === "Enter" || e.key === " ") && submit(item.target)}
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

<style>
  .omnibox-container {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .omnibox-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    height: 28px;
    background: var(--bg-omnibox);
    border: 1px solid var(--border-omnibox);
    border-radius: 14px;
    padding: 0 10px;
    transition: border-color 0.15s, background 0.15s;
    overflow: hidden;
  }

  .omnibox-wrapper.focused {
    border-color: #818cf8;
    background: #191722;
  }

  .omnibox-icon {
    display: flex;
    align-items: center;
    margin-right: 8px;
  }

  .progress-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    height: 2px;
    width: 100%;
    background: linear-gradient(90deg, #38bdf8, #818cf8, #c084fc);
    animation: indeterminate 1.5s infinite linear;
  }

  @keyframes indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }

  .suggestions-dropdown {
    position: absolute;
    top: 32px;
    left: 0;
    width: 100%;
    background: #1e1b2b;
    border: 1px solid var(--border-omnibox);
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    overflow: hidden;
    z-index: 100;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
    transition: background 0.1s;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: #2e2a40;
    color: #ffffff;
  }

  .suggestion-icon {
    display: flex;
    align-items: center;
    margin-right: 10px;
    color: var(--text-muted);
  }

  .suggestion-item:hover .suggestion-icon,
  .suggestion-item.selected .suggestion-icon {
    color: #818cf8;
  }

  .suggestion-text {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
