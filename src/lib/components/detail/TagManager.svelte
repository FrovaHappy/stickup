<script lang="ts">
  import { Plus, Tag } from '@lucide/svelte';
  import { allTags } from '$lib/stores/emotes';
  import { Chip, Dropdown } from '$lib/components/system-ui';
  import { toPascalCase, formatSearchQueryInput } from '$lib/utils/format';

  export let tagList: string[] = [];
  export let onChange: (newTags: string[]) => void;

  let tagInputText = '';
  let showSuggestions = false;
  let inputEl: HTMLInputElement | null = null;

  $: suggestedTags = ($allTags || [])
    .filter((t) => !tagList.some((existing) => existing.toLowerCase() === t.toLowerCase()))
    .filter((t) => {
      const q = tagInputText.trim().toLowerCase();
      if (!q) return true;
      const pascalQ = toPascalCase(tagInputText).toLowerCase();
      return t.toLowerCase().includes(q) || (pascalQ ? t.toLowerCase().includes(pascalQ) : false);
    });

  function addTag(tagName: string) {
    const clean = toPascalCase(tagName);
    if (!clean) return;
    if (tagList.some((t) => t.toLowerCase() === clean.toLowerCase())) {
      tagInputText = '';
      return;
    }
    onChange([...tagList, clean]);
    tagInputText = '';
  }

  function handleTagInput(e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    tagInputText = formatSearchQueryInput(target.value);
    target.value = tagInputText;
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
    <div class="section-title-wrap">
      <Tag size={13} class="tag-header-icon" />
      <h3 class="section-title">TAGS</h3>
    </div>
    <span class="tags-counter">{tagList.length} {tagList.length === 1 ? 'tag' : 'tags'}</span>
  </div>

  <div class="tag-manager-wrapper">
    <div
      class="tag-box-container"
      on:click={() => inputEl?.focus()}
      role="presentation"
    >
      {#each tagList as t}
        <Chip
          variant="amber"
          removable
          removeTitle="Eliminar tag"
          on:remove={() => removeTag(t)}
        >
          {t}
        </Chip>
      {/each}

      <input
        bind:this={inputEl}
        type="text"
        class="tag-inline-input"
        placeholder={tagList.length === 0 ? 'Escribe o busca un tag...' : 'Agregar tag...'}
        value={tagInputText}
        on:input={handleTagInput}
        on:focus={() => (showSuggestions = true)}
        on:keydown={handleKeydown}
      />
    </div>

    <Dropdown
      open={showSuggestions && suggestedTags.length > 0}
      title="Tags globales existentes"
      subtitle="{suggestedTags.length} disponibles"
      maxHeight="200px"
      on:close={() => (showSuggestions = false)}
    >
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
            <span class="tag-item-action">
              <Plus size={12} />
              <span>Agregar</span>
            </span>
          </button>
        {/each}
      </div>
    </Dropdown>
  </div>
</div>

<style>
  .tags-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
    border-top: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    padding-top: 12px;
    position: relative;
  }
  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-title-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  :global(.tag-header-icon) {
    color: var(--color-orange-primary, #f97316);
  }
  .section-title {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.5px;
    color: var(--color-text-muted, #9ca3af);
    margin: 0;
  }
  .tags-counter {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-dim, #71717a);
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
    background: var(--color-bg-input, rgba(0, 0, 0, 0.25));
    border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 10px;
    padding: 6px 10px;
    min-height: 42px;
    cursor: text;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }
  .tag-box-container:focus-within {
    border-color: var(--color-orange-primary, #f97316);
    box-shadow: 0 0 0 2px var(--color-orange-focus-ring, rgba(249, 115, 22, 0.25));
  }
  .tag-inline-input {
    flex: 1;
    min-width: 100px;
    background: transparent;
    border: none;
    color: var(--color-text-main, #f3f4f6);
    font-size: 13px;
    outline: none;
    padding: 4px 2px;
  }
  .tag-inline-input::placeholder {
    color: var(--color-text-dim, #71717a);
  }
  .tag-dropdown-list {
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
    color: inherit;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s ease;
  }
  .tag-dropdown-item:hover {
    background: rgba(var(--color-orange-rgb, 249, 115, 22), 0.15);
  }
  .tag-item-name {
    font-weight: 500;
  }
  .tag-item-action {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--color-orange-primary, #f97316);
    font-weight: 600;
  }
</style>
