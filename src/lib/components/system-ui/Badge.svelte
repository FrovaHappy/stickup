<script lang="ts">
  import type { BadgeShape, BadgeVariant } from '$lib/types/system-ui';

  export let variant: BadgeVariant = 'gray';
  export let shape: BadgeShape = 'rounded';
  export let size: 'sm' | 'md' = 'sm';
  export let title: string | undefined = undefined;
  export let animate = false;
</script>

<span
  class="ui-badge variant-{variant} shape-{shape} size-{size} {animate ? 'animate-pop' : ''}"
  {title}
>
  <slot />
</span>

<style>
  .ui-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    font-weight: 700;
    line-height: 1.3;
    white-space: nowrap;
    box-sizing: border-box;
  }

  /* Sizes */
  .size-sm {
    font-size: 11px;
    padding: 2px 6px;
  }
  .size-md {
    font-size: 12px;
    padding: 3px 9px;
  }

  /* Shapes */
  .shape-rounded {
    border-radius: 5px;
  }
  .shape-pill {
    border-radius: 999px;
  }
  .shape-circle {
    width: 22px;
    height: 22px;
    padding: 0;
    border-radius: 50%;
  }

  /* Variants */
  .variant-gray {
    background: rgba(156, 163, 175, 0.15);
    color: #9ca3af;
  }

  .variant-blue {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }

  .variant-green {
    background: rgba(16, 185, 129, 0.2);
    color: #059669;
  }
  @media (prefers-color-scheme: dark) {
    .variant-green {
      color: #34d399;
    }
  }

  .variant-amber {
    background: rgba(245, 158, 11, 0.2);
    color: #d97706;
  }
  @media (prefers-color-scheme: dark) {
    .variant-amber {
      color: #fbbf24;
    }
  }

  .variant-hot {
    background: linear-gradient(135deg, #f59e0b, #ef4444);
    color: #ffffff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
  }

  .variant-new {
    background: rgba(16, 185, 129, 0.92);
    color: #ffffff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
  }

  .animate-pop {
    animation: badgePopIn 0.2s ease-out;
  }

  @keyframes badgePopIn {
    from {
      transform: scale(0.5);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
  }
</style>

