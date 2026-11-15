type TabBehavior = 'cycle' | 'custom';

interface ModalEntry {
  root: HTMLElement;
  initial: HTMLElement;
  previous: HTMLElement | null;
  last: HTMLElement;
  tabBehavior: TabBehavior;
}

const stack: ModalEntry[] = [];
let restoring = false;

function top(): ModalEntry | undefined {
  return stack[stack.length - 1];
}

function usable(element: HTMLElement | null | undefined): element is HTMLElement {
  return !!element?.isConnected && !element.hasAttribute('disabled')
    && element.getClientRects().length > 0;
}

function focus(element: HTMLElement) {
  if (restoring) return;
  restoring = true;
  try {
    element.focus({ preventScroll: true });
  } finally {
    restoring = false;
  }
}

function focusEntry(entry: ModalEntry) {
  const target = usable(entry.last) && entry.root.contains(entry.last) ? entry.last
    : usable(entry.initial) ? entry.initial : entry.root;
  focus(target);
}

function focusFallback() {
  const panel = Array.from(document.querySelectorAll<HTMLElement>(
    '.panel.active, .current-panel, .parent-panel, .preview-panel'
  )).find(usable);
  if (panel) focus(panel);
}

function handleFocusIn(event: FocusEvent) {
  const entry = top();
  if (!entry || restoring) return;
  const target = event.target;
  if (target instanceof HTMLElement && entry.root.contains(target)) {
    entry.last = target;
  } else {
    focusEntry(entry);
  }
}

function focusable(entry: ModalEntry): HTMLElement[] {
  return Array.from(entry.root.querySelectorAll<HTMLElement>(
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter(usable);
}

function handleKeydown(event: KeyboardEvent) {
  const entry = top();
  if (!entry || event.key !== 'Tab' || entry.tabBehavior === 'custom') return;
  const items = focusable(entry);
  event.preventDefault();
  if (items.length === 0) {
    focusEntry(entry);
    return;
  }
  const index = items.indexOf(document.activeElement as HTMLElement);
  const next = event.shiftKey
    ? (index <= 0 ? items.length - 1 : index - 1)
    : (index < 0 || index === items.length - 1 ? 0 : index + 1);
  focus(items[next]);
}

export function hasActiveModal(): boolean {
  return stack.length > 0;
}

export function focusActiveModal() {
  const entry = top();
  if (entry) focusEntry(entry);
}

export function isInActiveModal(target: EventTarget | null): boolean {
  return !!top() && target instanceof Node && top()!.root.contains(target);
}

export function registerModal(root: HTMLElement, initial: HTMLElement = root, tabBehavior: TabBehavior = 'cycle'): () => void {
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const entry: ModalEntry = { root, initial, previous, last: initial, tabBehavior };
  if (stack.length === 0) {
    document.addEventListener('focusin', handleFocusIn, true);
    document.addEventListener('keydown', handleKeydown, true);
  }
  stack.push(entry);
  focusEntry(entry);
  return () => {
    const wasTop = top() === entry;
    const index = stack.indexOf(entry);
    if (index < 0) return;
    stack.splice(index, 1);
    if (stack.length === 0) {
      document.removeEventListener('focusin', handleFocusIn, true);
      document.removeEventListener('keydown', handleKeydown, true);
    }
    if (!wasTop) return;
    queueMicrotask(() => {
      if (top()) {
        focusActiveModal();
      } else if (usable(previous)) {
        focus(previous);
      } else {
        focusFallback();
      }
    });
  };
}
