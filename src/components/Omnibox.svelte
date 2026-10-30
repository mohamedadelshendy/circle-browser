<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { browserBridge } from "../lib/tauri-bridge";

  export let url = "";
  export let isLoading = false;

  const dispatch = createEventDispatcher<{
    navigate: string;
    toggleShields: void;
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
    <!-- Inside Left: Brave Shields Lion Icon (18x18px, accent color #fb542b) with 10px left margin -->
    <button
      type="button"
      class="omnibox-left-icon"
      title="Brave Shields (Trackers & Ads Blocked)"
      on:click={() => dispatch("toggleShields")}
      aria-label="Brave Shields"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <path
          d="M12 2L4 5V11.09C4 16.14 7.41 20.85 12 22C16.59 20.85 20 16.14 20 11.09V5L12 2Z"
          fill="#fb542b"
        />
        <path
          d="M12 6.5L8.5 8.2V11.5C8.5 13.8 10 15.8 12 16.5C14 15.8 15.5 13.8 15.5 11.5V8.2L12 6.5Z"
          fill="#ffffff"
          opacity="0.95"
        />
      </svg>
    </button>

    <!-- Omnibox Text Input -->
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

    <!-- Inside Right: Security / Rewards icons (16x16px) -->
    <div class="omnibox-right-actions">
      <!-- Security / Connection Lock Icon -->
      <button class="omnibox-action-btn" title="View site information">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </button>

      <!-- Brave Rewards Triangle Icon -->
      <button class="omnibox-action-btn" title="Brave Rewards">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <polygon points="12,2 22,20 2,20" stroke="#fb542b" stroke-width="2" fill="rgba(251, 84, 43, 0.15)"/>
        </svg>
      </button>
    </div>

    <!-- Active Loading Progress Indicator -->
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
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
              </svg>
            {:else if item.type === "url"}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="2" y1="12" x2="22" y2="12"></line>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
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
  .progress-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    height: 2px;
    width: 100%;
    background: linear-gradient(90deg, #fb542b, #818cf8, #fb542b);
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
</style>
