<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let open = false;
  export let title: string | undefined = undefined;
  export let subtitle: string | undefined = undefined;
  export let maxHeight = '320px';

  const dispatch = createEventDispatcher<{ close: void }>();

  function closeDropdown() {
    open = false;
    dispatch('close');
  }
</script>

{#if open}
  <div
    class="ui-dropdown-backdrop"
    on:click|stopPropagation={closeDropdown}
    role="presentation"
  ></div>

  <div class="ui-dropdown-menu" style:max-height={maxHeight}>
    {#if title || subtitle || $$slots.header}
      <div class="ui-dropdown-header">
        {#if $$slots.header}
          <slot name="header" />
        {:else}
          {#if title}
            <span class="ui-dropdown-title">{title}</span>
          {/if}
          {#if subtitle}
            <span class="ui-dropdown-subtitle">{subtitle}</span>
          {/if}
        {/if}
      </div>
    {/if}

    <div class="ui-dropdown-content">
      <slot />
    </div>
  </div>
{/if}

<style>
  .ui-dropdown-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }

  .ui-dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    z-index: 100;
    background: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 14px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    text-align: left;
    animation: dropdownSlideIn 0.14s ease-out;
  }

  @media (prefers-color-scheme: dark) {
    .ui-dropdown-menu {
      background: #1f1f23;
      border-color: rgba(255, 255, 255, 0.12);
      box-shadow:
        0 20px 40px rgba(0, 0, 0, 0.55),
        0 0 0 1px rgba(255, 255, 255, 0.05);
    }
  }

  @keyframes dropdownSlideIn {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .ui-dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: rgba(148, 163, 184, 0.06);
    border-bottom: 1px solid rgba(148, 163, 184, 0.14);
    font-size: 11px;
    font-weight: 600;
    color: #9ca3af;
  }

  .ui-dropdown-title {
    font-weight: 700;
  }

  .ui-dropdown-subtitle {
    font-size: 10px;
    color: #6b7280;
  }

  .ui-dropdown-content {
    padding: 12px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>

