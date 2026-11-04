<script lang="ts">
  import type { Emote } from '$lib/types/emote';
  import { isEmoteNewByDate, topHotRanks } from '$lib/stores/emotes';
  import { Badge, Card } from '$lib/components/system-ui';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  export let e: Emote;
  const dispatch = createEventDispatcher<{
    select: Emote;
    hover: Emote;
    unhover: Emote;
    contextmenu: { emote: Emote; event: MouseEvent };
  }>();

  $: isNew = isEmoteNewByDate(e.created_at);
  $: hotRank = $topHotRanks.get(e.id) ?? null;
  $: isHot = hotRank !== null;
  $: cardHighlight = (isHot ? 'hot' : isNew ? 'new' : 'none') as 'hot' | 'new' | 'none';
</script>

<div
  class="emote-card-wrapper"
  on:mouseenter={() => dispatch('hover', e)}
  on:mouseleave={() => dispatch('unhover', e)}
  on:focusin={() => dispatch('hover', e)}
  on:focusout={() => dispatch('unhover', e)}
  on:contextmenu={(evt) => {
    evt.preventDefault();
    evt.stopPropagation();
    dispatch('contextmenu', { emote: e, event: evt });
  }}
  role="presentation"
>
  <Card
    interactive
    highlight={cardHighlight}
    padding="sm"
    title={`${e.name} (Ctrl+C para copiar)`}
    on:click={() => dispatch('select', e)}
  >
  <div class="badges-container">
    {#if isHot}
      <Badge
        variant="hot"
        shape="pill"
        animate
        title="Top #{hotRank} más usado ({e.copy_count ?? 0} copias, {e.interaction_count ?? 0} interacciones)"
      >
        <svg
          width="11"
          height="11"
          viewBox="0 0 24 24"
          fill="currentColor"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"
          ></path>
        </svg>
        <span class="rank-num">#{hotRank}</span>
      </Badge>
    {/if}

    {#if isNew}
      <Badge
        variant="new"
        shape="circle"
        animate
        title="Recién creado ({new Date(e.created_at).toLocaleString()})"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="currentColor"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"
          ></path>
          <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"></path>
        </svg>
      </Badge>
    {/if}
  </div>

  <div class="img-wrapper">
    <img src={convertFileSrc(e.path)} alt={e.name} />
  </div>
  <div class="meta">{e.name}</div>
</Card>
</div>

<style>
  .emote-card-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
  }
  .badges-container {
    position: absolute;
    top: 6px;
    right: 6px;
    display: flex;
    gap: 4px;
    z-index: 2;
    pointer-events: none;
  }
  .rank-num {
    font-size: 10px;
    font-weight: 800;
    line-height: 1;
  }
  .img-wrapper {
    width: 100%;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    margin-bottom: 6px;
    overflow: hidden;
  }
  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }
  .meta {
    width: 100%;
    font-size: 11px;
    font-weight: 500;
    color: inherit;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
