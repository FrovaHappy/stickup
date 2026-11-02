<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isDragging = false;
  export let disabled = false;
  export let title = 'Arrastra y suelta tus archivos o carpetas aquí, o usa los botones de abajo.';
  export let draggingTitle = '¡Suelta tus archivos o carpetas aquí para importarlos!';
  export let subtitle =
    'Soporta 1 o varios archivos (.png, .jpg, .webp, .gif), carpetas completas, portapapeles y URL';
  export let showDivider = true;
  export let dividerText = 'OR';
  export let ariaLabel = 'Zona para arrastrar y soltar archivos';

  const dispatch = createEventDispatcher<{
    drop: DragEvent;
    dragenter: DragEvent;
    dragleave: DragEvent;
  }>();

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (disabled) return;
    isDragging = true;
    dispatch('dragenter', e);
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    dispatch('dragleave', e);
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    if (disabled) return;
    dispatch('drop', e);
  }
</script>

<div
  class="ui-dropzone {isDragging ? 'is-dragging' : ''} {disabled ? 'is-disabled' : ''}"
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="region"
  aria-label={ariaLabel}
>
  <div class="ui-dropzone-icon">
    {#if $$slots.icon}
      <slot name="icon" />
    {:else}
      <svg
        width="68"
        height="58"
        viewBox="0 0 68 58"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M6 10C6 6.68629 8.68629 4 12 4H25.5147C27.106 4 28.6321 4.63214 29.7574 5.75736L34 10H54C57.3137 10 60 12.6863 60 16V48C60 51.3137 57.3137 54 54 54H12C8.68629 54 6 51.3137 6 48V10Z"
          fill="#3B5BFF"
        />
        <path d="M9 14H56V46H9V14Z" fill="#0F172A" />
        <path
          d="M6 22C6 19.2386 8.23858 17 11 17H61.5C64.5376 17 66.8548 19.7263 66.3556 22.7219L61.6889 50.7219C61.2909 53.1098 59.2248 54.8571 56.8037 54.8571H11C8.23858 54.8571 6 52.6186 6 49.8571V22Z"
          fill="#4C6EF5"
        />
        <rect x="44" y="25" width="16" height="4" rx="2" fill="#1E3A8A" />
        <rect x="42" y="33" width="16" height="4" rx="2" fill="#1E3A8A" />
      </svg>
    {/if}
  </div>

  <p class="ui-dropzone-title">
    {isDragging ? draggingTitle : title}
  </p>

  {#if subtitle}
    <span class="ui-dropzone-subtitle">{subtitle}</span>
  {/if}

  {#if showDivider && $$slots.default}
    <div class="ui-dropzone-divider">
      <span class="ui-dropzone-line"></span>
      <span class="ui-dropzone-divider-text">{dividerText}</span>
      <span class="ui-dropzone-line"></span>
    </div>
  {/if}

  <slot />
</div>

<style>
  .ui-dropzone {
    background: #f4f7fb;
    border: 1.5px dashed #cbd5e1;
    border-radius: 16px;
    padding: 32px 24px 26px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
    transition:
      border-color 0.15s ease,
      background 0.15s ease,
      transform 0.15s ease;
  }

  .ui-dropzone.is-dragging {
    border-color: #3b5bff;
    background: rgba(59, 91, 255, 0.08);
    transform: scale(1.01);
  }

  .ui-dropzone.is-disabled {
    opacity: 0.7;
  }

  @media (prefers-color-scheme: dark) {
    .ui-dropzone {
      background: #262830;
      border-color: rgba(255, 255, 255, 0.16);
    }
    .ui-dropzone.is-dragging {
      border-color: #4c6ef5;
      background: rgba(76, 110, 245, 0.14);
    }
  }

  .ui-dropzone-icon {
    margin-bottom: 4px;
    filter: drop-shadow(0 6px 12px rgba(59, 91, 255, 0.22));
  }

  .ui-dropzone-title {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    line-height: 1.45;
    color: #334155;
    max-width: 290px;
  }
  @media (prefers-color-scheme: dark) {
    .ui-dropzone-title {
      color: #f1f5f9;
    }
  }

  .ui-dropzone-subtitle {
    font-size: 11.5px;
    color: #64748b;
    max-width: 320px;
    line-height: 1.35;
  }
  @media (prefers-color-scheme: dark) {
    .ui-dropzone-subtitle {
      color: #94a3b8;
    }
  }

  .ui-dropzone-divider {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 220px;
    margin: 4px 0;
  }

  .ui-dropzone-line {
    flex: 1;
    height: 1px;
    background: #cbd5e1;
  }
  @media (prefers-color-scheme: dark) {
    .ui-dropzone-line {
      background: rgba(255, 255, 255, 0.15);
    }
  }

  .ui-dropzone-divider-text {
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    letter-spacing: 0.5px;
  }
</style>

