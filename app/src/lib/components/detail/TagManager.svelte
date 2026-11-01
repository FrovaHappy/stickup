<script lang="ts">
  import { allTags } from '$lib/stores/emotes';

  export let tagList: string[] = [];
  export let onChange: (newTags: string[]) => void;

  let tagInputText = '';
  let showSuggestions = false;
  let inputEl: HTMLInputElement | null = null;

  $: suggestedTags = ($allTags || [])
    .filter((t) => !tagList.some((existing) => existing.toLowerCase() === t.toLowerCase()))
    .filter((t) => {
      if (!tagInputText.trim()) return true;
      return t.toLowerCase().includes(tagInputText.trim().toLowerCase());
    });

  function addTag(tagName: string) {
    const clean = tagName.trim();
    if (!clean) return;
    if (tagList.some((t) => t.toLowerCase() === clean.toLowerCase())) {
      tagInputText = '';
      return;
    }
    onChange([...tagList, clean]);
    tagInputText = '';
  }

  function removeTag(tagToRemove: string) {
    onChange(tagList.filter((t) => t !== tagToRemove));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (tagInputText.trim()) {
        addTag(tagInputText);
      }
    } else if (e.key === 'Backspace' && !tagInputText && tagList.length > 0) {
      removeTag(tagList[tagList.length - 1]);
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }
</script>

<div class="tags-section">
  <div class="section-header-row">
    <h3 class="section-title">TAGS</h3>
    <span class="tags-counter">{tagList.length} {tagList.length === 1 ? 'tag' : 'tags'}</span>
  </div>

  <div class="tag-manager-wrapper">
    <div
      class="tag-box-container"
      on:click={() => inputEl?.focus()}
      role="presentation"
    >
      {#each tagList as t}
        <span class="tag-badge">
          <span class="tag-badge-text">{t}</span>
          <button
            type="button"
            class="tag-badge-remove"
            title="Eliminar tag"
            on:click|stopPropagation={() => removeTag(t)}
          >
            ✕
          </button>
        </span>
      {/each}

      <input
        bind:this={inputEl}
        type="text"
        class="tag-inline-input"
        placeholder={tagList.length === 0 ? 'Escribe o busca un tag...' : 'Agregar tag...'}
        bind:value={tagInputText}
        on:focus={() => (showSuggestions = true)}
        on:keydown={handleKeydown}
      />
    </div>

    {#if showSuggestions && suggestedTags.length > 0}
      <div
        class="dropdown-backdrop"
        on:click|stopPropagation={() => (showSuggestions = false)}
        role="presentation"
      ></div>

      <div class="tag-dropdown">
        <div class="tag-dropdown-header">
          <span class="dropdown-header-title">🏷️ Tags globales existentes</span>
          <span class="dropdown-header-count">{suggestedTags.length} disponibles</span>
        </div>
        <div class="tag-dropdown-list">
          {#each suggestedTags as st}
            <button
              type="button"
              class="tag-dropdown-item"
              on:click|stopPropagation={() => {
                addTag(st);
                inputEl?.focus();
              }}
            >
              <span class="tag-item-name">{st}</span>
              <span class="tag-item-action">+ Agregar</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .tags-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 12px;
    position: relative;
  }
  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-title {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.5px;
    color: #9ca3af;
    margin: 0;
  }
  .tags-counter {
    font-size: 11px;
    font-weight: 600;
    color: #6b7280;
  }
  .tag-manager-wrapper {
    position: relative;
    width: 100%;
  }
  .tag-box-container {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    background: #2b2b2f;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 6px 10px;
    min-height: 42px;
    cursor: text;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }
  .tag-box-container:focus-within {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
  }
  .tag-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(99, 102, 241, 0.18);
    border: 1px solid rgba(99, 102, 241, 0.4);
    color: #c7d2fe;
    font-size: 12px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 6px;
    animation: tagAppear 0.12s ease-out;
  }
  @keyframes tagAppear {
    from {
      opacity: 0;
      transform: scale(0.92);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  .tag-badge-text {
    line-height: 1.2;
    white-space: nowrap;
  }
  .tag-badge-remove {
    background: transparent;
    border: none;
    color: #a5b4fc;
    font-size: 11px;
    line-height: 1;
    padding: 0 1px;
    cursor: pointer;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }
  .tag-badge-remove:hover {
    background: rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }
  .tag-inline-input {
    flex: 1;
    min-width: 100px;
    background: transparent;
    border: none;
    color: #f3f4f6;
    font-size: 13px;
    outline: none;
    padding: 4px 2px;
  }
  .tag-inline-input::placeholder {
    color: #6b7280;
  }
  .dropdown-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1050;
  }
  .tag-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    background: #232326;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.05);
    z-index: 1100;
    max-height: 200px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .tag-dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    font-size: 11px;
    color: #9ca3af;
    font-weight: 600;
  }
  .dropdown-header-count {
    color: #6b7280;
    font-size: 10px;
  }
  .tag-dropdown-list {
    overflow-y: auto;
    max-height: 160px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tag-dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #e5e7eb;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s ease;
  }
  .tag-dropdown-item:hover {
    background: rgba(99, 102, 241, 0.18);
    color: #ffffff;
  }
  .tag-item-name {
    font-weight: 500;
  }
  .tag-item-action {
    font-size: 11px;
    color: #818cf8;
    font-weight: 600;
  }
</style>

