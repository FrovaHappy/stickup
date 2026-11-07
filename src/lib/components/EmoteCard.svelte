<script lang="ts">
  import type { Emote } from "$lib/types/emote";
  import { isEmoteNewByDate, topHotRanks } from "$lib/stores/emotes";
  import { Badge } from "$lib/components/system-ui";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

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
</script>

<div
  class="emote-card-wrapper"
  on:mouseenter={() => dispatch("hover", e)}
  on:mouseleave={() => dispatch("unhover", e)}
  on:focusin={() => dispatch("hover", e)}
  on:focusout={() => dispatch("unhover", e)}
  on:contextmenu={(evt) => {
    evt.preventDefault();
    evt.stopPropagation();
    dispatch("contextmenu", { emote: e, event: evt });
  }}
  role="presentation"
>
  <button
    type="button"
    class="emote-card"
    class:is-hot={isHot}
    class:is-new={isNew}
    title={`${e.name} (Ctrl+C para copiar)`}
    on:click={() => dispatch("select", e)}
  >
    <div class="badges-container">
      {#if isHot}
        <Badge
          variant="hot"
          shape="pill"
          animate
          title="Top #{hotRank} más usado ({e.copy_count ?? 0} copias)"
        >
          <svg
            width="10"
            height="10"
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
          title="Recién agregado ({new Date(e.created_at).toLocaleString()})"
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
              d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"
            ></path>
            <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"></path>
          </svg>
        </Badge>
      {/if}
    </div>

    <div class="img-container">
      <img src={convertFileSrc(e.path)} alt={e.name} loading="lazy" />
    </div>

    <span class="hover-label">{e.name}</span>
  </button>
</div>

<style>
  .emote-card-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
  }

  .emote-card {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
    background: #27272a;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    overflow: hidden;
    transition:
      transform 0.16s ease,
      border-color 0.16s ease,
      box-shadow 0.16s ease,
      background-color 0.16s ease;
  }

  .emote-card:hover,
  .emote-card:focus-visible {
    border-color: var(--color-orange-primary, #f97316);
    background-color: #2e2e32;
    box-shadow: 0 0 12px rgba(var(--color-orange-rgb, 249, 115, 22), 0.45);
    transform: scale(1.03);
    outline: none;
    z-index: 5;
  }

  .badges-container {
    position: absolute;
    top: 5px;
    right: 5px;
    display: flex;
    gap: 4px;
    z-index: 2;
    pointer-events: none;
  }

  .rank-num {
    font-size: 9px;
    font-weight: 800;
    line-height: 1;
  }

  .img-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12%;
    box-sizing: border-box;
  }

  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
    pointer-events: none;
    user-select: none;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
  }

  .hover-label {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.85), transparent);
    color: #f4f4f5;
    font-size: 10.5px;
    font-weight: 500;
    padding: 10px 4px 4px 4px;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0;
    transform: translateY(4px);
    transition: all 0.15s ease;
    pointer-events: none;
  }

  .emote-card:hover .hover-label,
  .emote-card:focus-visible .hover-label {
    opacity: 1;
    transform: translateY(0);
  }
</style>
