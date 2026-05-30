<script lang="ts">
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

  function onClaimKeydown(event: KeyboardEvent, id: string) {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onSelect(id);
  }
</script>

<p class="ct">
  {#each segments as segment, index (index)}
    {#if segment.kind === 'plain'}
      <span class="plain">{segment.text}</span>
    {:else}
      <span
        class="claim kind-{segment.claim.kind}"
        class:selected={segment.claim.id === selectedId}
        data-id={segment.claim.id}
        role="button"
        tabindex="0"
        onclick={() => onSelect(segment.claim.id)}
        onkeydown={(event) => onClaimKeydown(event, segment.claim.id)}
      >
        {segment.claim.text}
      </span>
    {/if}
  {/each}
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
    border-radius: var(--radius-sm);
    padding: 1px 3px;
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

  .kind-fact {
    background: var(--ok-soft);
  }

  .kind-inference {
    background: var(--warn-soft);
  }

  .kind-opinion {
    background: var(--neutral-soft);
  }

  .kind-contradiction {
    background: var(--bad-soft);
  }
</style>
