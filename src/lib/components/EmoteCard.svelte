<script lang="ts">
  import type { Emote } from '$lib/types/emote';
  import { isEmoteNewByDate, topHotRanks } from '$lib/stores/emotes';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  export let e: Emote;
  const dispatch = createEventDispatcher<{ select: Emote }>();

  $: isNew = isEmoteNewByDate(e.created_at);
  $: hotRank = $topHotRanks.get(e.id) ?? null;
  $: isHot = hotRank !== null;
</script>

<button
  type="button"
  class="card {isNew ? 'is-new' : ''} {isHot ? 'is-hot' : ''}"
  on:click={() => dispatch('select', e)}
  title={e.name}
>
  <div class="badges-container">
    {#if isHot}
      <span
        class="badge hot-badge"
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
      </span>
    {/if}

    {#if isNew}
      <span class="badge new-badge" title="Recién creado ({new Date(e.created_at).toLocaleString()})">
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
      </span>
    {/if}
  </div>

  <div class="img-wrapper">
    <img src={convertFileSrc(e.path)} alt={e.name} />
  </div>
  <div class="meta">{e.name}</div>
</button>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    padding: 8px;
    cursor: pointer;
    transition: transform 0.15s ease, background 0.15s ease, border-color 0.15s ease,
      box-shadow 0.15s ease;
    overflow: hidden;
    text-align: center;
    width: 100%;
    box-sizing: border-box;
  }
  .card.is-new {
    border-color: rgba(16, 185, 129, 0.5);
  }
  .card.is-hot {
    border-color: rgba(249, 115, 22, 0.55);
    box-shadow: 0 0 10px rgba(249, 115, 22, 0.16);
  }
  .card:hover {
    transform: translateY(-2px);
    background: rgba(255, 255, 255, 0.1);
    border-color: #6366f1;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.25);
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
  .badge {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
    animation: popIn 0.2s ease-out;
  }
  .new-badge {
    background: rgba(16, 185, 129, 0.92);
  }
  .hot-badge {
    width: auto;
    padding: 0 6px;
    border-radius: 999px;
    gap: 2px;
    background: linear-gradient(135deg, #f59e0b, #ef4444);
  }
  .rank-num {
    font-size: 10px;
    font-weight: 800;
    line-height: 1;
  }
  @keyframes popIn {
    from {
      transform: scale(0.5);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
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
    color: #e5e7eb;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
