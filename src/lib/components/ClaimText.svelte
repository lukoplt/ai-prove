<script lang="ts">
  import { t } from '$lib/stores/i18n.svelte';
  import type { Claim } from '$lib/types';

  let {
    input,
    claims,
    selectedId,
    onSelect = () => {},
  }: {
    input: string;
    claims: Claim[];
    selectedId: string | null;
    onSelect?: (id: string) => void;
  } = $props();

  type Segment = { kind: 'plain'; text: string } | { kind: 'claim'; claim: Claim };

  const segments = $derived(buildSegments(input, claims));
  const order = $derived(
    segments.filter(
      (segment): segment is { kind: 'claim'; claim: Claim } => segment.kind === 'claim',
    ),
  );

  function buildSegments(text: string, list: Claim[]): Segment[] {
    if (list.length === 0) return [{ kind: 'plain', text }];

    const sorted = [...list].sort((a, b) => a.start - b.start);
    const out: Segment[] = [];
    let cursor = 0;

    for (const claim of sorted) {
      if (claim.start < cursor) continue;
      if (claim.start > cursor) out.push({ kind: 'plain', text: text.slice(cursor, claim.start) });
      out.push({ kind: 'claim', claim });
      cursor = claim.end;
    }

    if (cursor < text.length) out.push({ kind: 'plain', text: text.slice(cursor) });
    return out;
  }

  /**
   * The colour alone carries the classification, which fails WCAG 1.4.1. This
   * repeats it as text for screen readers, along with the verification verdict
   * once it arrives.
   */
  function claimLabel(claim: Claim): string {
    const parts = [t(`sidepanel.kind_${claim.kind}`)];
    if (claim.verification) parts.push(t(`status.${claim.verification.status}`));
    return parts.join(', ');
  }

  function isFocusable(claim: Claim, index: number): boolean {
    if (selectedId !== null) return claim.id === selectedId;
    return order[0]?.claim.id === claim.id && index >= 0;
  }

  function onClaimKeydown(event: KeyboardEvent, id: string) {
    const index = order.findIndex((segment) => segment.claim.id === id);
    if (index < 0 || order.length === 0) return;

    let step = 0;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') step = 1;
    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') step = -1;
    if (step === 0) return;

    event.preventDefault();
    onSelect(order[(index + step + order.length) % order.length].claim.id);
  }
</script>

<p class="ct" role="group" aria-label={t('a11y.claims_group')}>
  {#each segments as segment, index (index)}{#if segment.kind === 'plain'}<span class="plain"
        >{segment.text}</span
      >{:else}<button
        type="button"
        class="claim kind-{segment.claim.kind}"
        class:selected={segment.claim.id === selectedId}
        data-id={segment.claim.id}
        aria-pressed={segment.claim.id === selectedId}
        tabindex={isFocusable(segment.claim, index) ? 0 : -1}
        onclick={() => onSelect(segment.claim.id)}
        onkeydown={(event) => onClaimKeydown(event, segment.claim.id)}
        >{segment.claim.text}<span class="sr-only"> ({claimLabel(segment.claim)})</span></button
      >{/if}{/each}
</p>

<style>
  .ct {
    margin: 0;
    line-height: 1.75;
    font-size: 15px;
    white-space: pre-wrap;
    color: var(--text);
  }

  .claim {
    display: inline;
    margin: 0;
    padding: 1px 3px;
    border: 0;
    border-radius: var(--radius-sm);
    color: inherit;
    font: inherit;
    text-align: left;
    white-space: pre-wrap;
    cursor: pointer;
    outline: 2px solid transparent;
    transition:
      outline-color var(--dur-fast) var(--ease),
      background var(--dur-fast) var(--ease);
  }

  .claim:hover {
    background: var(--accent-soft);
  }

  .claim.selected {
    outline-color: var(--accent);
  }

  /* Redundant, non-colour cue so the classification survives colour-blindness,
     greyscale printing, and forced-colors mode. */
  .kind-fact {
    background: var(--ok-soft);
    box-shadow: inset 0 -2px 0 var(--ok);
  }

  .kind-inference {
    background: var(--warn-soft);
    box-shadow: inset 0 -2px 0 var(--warn);
  }

  .kind-opinion {
    background: var(--neutral-soft);
    box-shadow: inset 0 -2px 0 var(--neutral);
  }

  .kind-contradiction {
    background: var(--bad-soft);
    box-shadow: inset 0 -2px 0 var(--bad);
  }
</style>
