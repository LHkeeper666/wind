<script lang="ts">
  import type { TocHeading } from '$lib/previewers';
  import OutlineSidebar, { makeOutlineId, type OutlineNode } from './OutlineSidebar.svelte';

  let {
    headings = [],
    activeLine = -1,
    onJump = (_line: number) => {},
    onFocusChange = (_focused: boolean) => void 0,
    onExit = undefined,
  }: {
    headings: TocHeading[];
    activeLine?: number;
    onJump?: (line: number) => void;
    onFocusChange?: (focused: boolean) => void;
    onExit?: () => void;
  } = $props();

  let inner: OutlineSidebar | undefined = $state(undefined);

  function mapNodes(items: TocHeading[], parentId: string | null): OutlineNode[] {
    return items.map((heading, index) => {
      const id = makeOutlineId(parentId, index);
      return {
        id,
        label: heading.text,
        anchor: heading.line,
        children: mapNodes(heading.children, id),
      };
    });
  }

  let nodes = $derived(mapNodes(headings, null));

  let expandedIds = $derived.by(() => {
    const ids = new Set<string>();
    function walk(items: TocHeading[], parentId: string | null) {
      items.forEach((heading, index) => {
        const id = makeOutlineId(parentId, index);
        if (heading.expanded && heading.children.length > 0) ids.add(id);
        walk(heading.children, id);
      });
    }
    walk(headings, null);
    return ids;
  });

  function locateHeading(id: string): TocHeading | null {
    const parts = id.split('.').map(Number);
    let list = headings;
    let found: TocHeading | null = null;
    for (const part of parts) {
      found = list[part] ?? null;
      if (!found) return null;
      list = found.children;
    }
    return found;
  }

  function handleToggle(id: string) {
    const heading = locateHeading(id);
    if (!heading) return;
    heading.expanded = !heading.expanded;
    headings = [...headings];
  }

  export function getSelectedIndex(): number {
    return inner?.getSelectedIndex() ?? -1;
  }

  export function setSelectedTocIndex(index: number): void {
    inner?.setSelectedIndex(index);
  }

  export function getSelectedId(): string | null {
    return inner?.getSelectedId() ?? null;
  }

  export function setSelectedId(id: string): void {
    inner?.setSelectedId(id);
  }

  export function focus(): boolean {
    return inner?.focus() ?? false;
  }
</script>

<OutlineSidebar
  bind:this={inner}
  {nodes}
  {expandedIds}
  activeAnchor={activeLine}
  onJump={(node) => onJump(node.anchor)}
  onToggle={handleToggle}
  {onFocusChange}
  {onExit}
  title="OUTLINE"
  searchPlaceholder="Search headings..."
  emptyText="No headings"
/>
