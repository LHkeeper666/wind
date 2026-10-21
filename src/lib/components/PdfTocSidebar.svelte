<script lang="ts">
  import type { PdfOutlineItem } from '$lib/utils/pdf-shared';
  import OutlineSidebar, { makeOutlineId, type OutlineNode } from './OutlineSidebar.svelte';

  let {
    outline = [],
    currentPage = -1,
    expandedIds = new Set<string>(),
    onToggleExpanded = (_id: string) => {},
    onJump = (_page: number, _y: number) => {},
    onFocusChange = (_focused: boolean) => void 0,
    onExit = () => void 0,
  }: {
    outline: PdfOutlineItem[];
    currentPage?: number;
    pageDimensions?: { width: number; height: number }[];
    expandedIds?: Set<string>;
    onToggleExpanded?: (id: string) => void;
    onJump?: (page: number, y: number) => void;
    onFocusChange?: (focused: boolean) => void;
    onExit?: () => void;
  } = $props();

  let inner: OutlineSidebar | undefined = $state(undefined);

  let mapped = $derived.by(() => {
    const byId = new Map<string, PdfOutlineItem>();
    function walk(items: PdfOutlineItem[], parentId: string | null): OutlineNode[] {
      return items.map((item, index) => {
        const id = makeOutlineId(parentId, index);
        byId.set(id, item);
        return {
          id,
          label: item.title,
          anchor: item.page,
          children: walk(item.children, id),
        };
      });
    }
    const nodes = walk(outline, null);
    return { nodes, byId };
  });

  let nodes = $derived(mapped.nodes);

  function handleJump(node: OutlineNode) {
    const item = mapped.byId.get(node.id);
    if (item) onJump(item.page, item.y);
  }

  export function getSelectedIndex(): number {
    return inner?.getSelectedIndex() ?? -1;
  }

  export function setSelectedIndex(index: number): void {
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
  activeAnchor={currentPage}
  onJump={handleJump}
  onToggle={onToggleExpanded}
  {onFocusChange}
  {onExit}
  title="OUTLINE"
  searchPlaceholder="Search outline..."
  emptyText="No outline"
  showAnchor
  passThroughCtrlW
/>
