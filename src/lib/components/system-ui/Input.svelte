<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { InputVariant } from '$lib/types/system-ui';

  export let value = '';
  export let label: string | undefined = undefined;
  export let type: 'text' | 'url' | 'search' = 'text';
  export let placeholder = '';
  export let readonly = false;
  export let disabled = false;
  export let required = false;
  export let active = false;
  export let variant: InputVariant = 'default';
  export let title: string | undefined = undefined;
  export let inputElement: HTMLInputElement | null = null;

  const dispatch = createEventDispatcher<{
    focus: FocusEvent;
    blur: FocusEvent;
    keydown: KeyboardEvent;
    input: Event;
  }>();

  export function focus() {
    inputElement?.focus();
  }

  export function blur() {
    inputElement?.blur();
  }
</script>

<div class="ui-input-group">
  {#if label}
    <span class="ui-input-label">{label}</span>
  {/if}

  <div
    class="ui-input-wrapper variant-{variant} {active ? 'is-active' : ''} {disabled ? 'is-disabled' : ''}"
    on:click={() => inputElement?.focus()}
    role="presentation"
  >
    {#if $$slots.leading}
      <div class="ui-input-slot leading">
        <slot name="leading" />
      </div>
    {/if}

    <input
      bind:this={inputElement}
      {type}
      bind:value
      {placeholder}
      {readonly}
      {disabled}
      {required}
      {title}
      class="ui-input-field {readonly ? 'is-readonly' : ''}"
      on:focus={(e) => dispatch('focus', e)}
      on:blur={(e) => dispatch('blur', e)}
      on:keydown={(e) => dispatch('keydown', e)}
      on:input={(e) => dispatch('input', e)}
    />

    {#if $$slots.trailing}
      <div class="ui-input-slot trailing">
        <slot name="trailing" />
      </div>
    {/if}
  </div>
</div>

<style>
  .ui-input-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
    text-align: left;
  }

  .ui-input-label {
    font-size: 13px;
    font-weight: 600;
    color: #6b7280;
  }
  @media (prefers-color-scheme: dark) {
    .ui-input-label {
      color: #9ca3af;
    }
  }

  .ui-input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    box-sizing: border-box;
    background: #ffffff;
    border: 1px solid #cbd5e1;
    border-radius: 8px;
    padding: 0 10px;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
    cursor: text;
  }

  .ui-input-wrapper.variant-pill {
    border-radius: 9999px;
    padding: 4px 14px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  }

  .ui-input-wrapper:focus-within,
  .ui-input-wrapper.is-active {
    border-color: #6366f1;
    box-shadow: 0 4px 14px rgba(99, 102, 241, 0.16);
  }

  .ui-input-wrapper.is-disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .ui-input-wrapper {
      background: #232327;
      border-color: rgba(255, 255, 255, 0.12);
    }
    .ui-input-wrapper.variant-pill {
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    }
  }

  .ui-input-field {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: 13.5px;
    padding: 8px 0;
    outline: none;
  }

  .ui-input-field::placeholder {
    color: #9ca3af;
  }

  .ui-input-field.is-readonly {
    color: #9ca3af;
    cursor: default;
  }

  .ui-input-slot {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    flex-shrink: 0;
    color: #9ca3af;
  }
</style>

