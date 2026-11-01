<script lang="ts">
  import type { Emote } from '$lib/types/emote';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  export let e: Emote;
  const dispatch = createEventDispatcher<{ select: Emote }>();
</script>

<button type="button" class="card" on:click={() => dispatch('select', e)} title={e.name}>
  <div class="img-wrapper">
    <img src={convertFileSrc(e.path)} alt={e.name} />
  </div>
  <div class="meta">{e.name}</div>
</button>

<style>
  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    padding: 8px;
    cursor: pointer;
    transition: transform 0.15s ease, background 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    overflow: hidden;
    text-align: center;
    width: 100%;
    box-sizing: border-box;
  }
  .card:hover {
    transform: translateY(-2px);
    background: rgba(255, 255, 255, 0.1);
    border-color: #6366f1;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.25);
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
