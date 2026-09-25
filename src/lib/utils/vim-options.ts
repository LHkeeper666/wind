import { invoke } from '@tauri-apps/api/core';
import { Vim } from '@replit/codemirror-vim';

interface OptionMeta {
  name: string;
  shortName?: string;
  type: 'boolean' | 'number' | 'string';
  defaultValue: unknown;
  persist: boolean;
}

interface OptionSnapshot {
  name: string;
  value: unknown;
  defaultValue: unknown;
  modified: boolean;
}

type ChangeListener = (value: unknown) => void;

interface OptionEntry extends OptionMeta {
  value: unknown;
  listeners: Set<ChangeListener>;
}

class VimOptionStore {
  private options = new Map<string, OptionEntry>();
  private loaded = false;
  private setCommandRegistered = false;

  register(meta: OptionMeta, onChange?: ChangeListener): void {
    if (this.options.has(meta.name)) return;
    if (meta.shortName && this.options.has(meta.shortName)) return;

    this.options.set(meta.name, {
      ...meta,
      value: meta.defaultValue,
      listeners: new Set(),
    });

    if (meta.shortName) {
      this.options.set(meta.shortName, this.options.get(meta.name)!);
    }

    if (onChange) {
      this.onChange(meta.name, onChange);
    }
  }

  get<T>(name: string): T {
    const entry = this.options.get(name);
    if (!entry) throw new Error(`Unknown option: ${name}`);
    return entry.value as T;
  }

  set(name: string, value: unknown): void {
    const entry = this.options.get(name);
    if (!entry) throw new Error(`Unknown option: ${name}`);
    const oldValue = entry.value;
    entry.value = value;
    if (value !== oldValue) {
      this.notify(name, value);
    }
  }

  toggle(name: string): void {
    const entry = this.options.get(name);
    if (!entry) throw new Error(`Unknown option: ${name}`);
    if (entry.type !== 'boolean') return;
    this.set(name, !entry.value);
  }

  onChange(name: string, fn: ChangeListener): () => void {
    const entry = this.options.get(name);
    if (!entry) throw new Error(`Unknown option: ${name}`);
    entry.listeners.add(fn);
    return () => entry.listeners.delete(fn);
  }

  list(all?: boolean): OptionSnapshot[] {
    const seen = new Set<string>();
    const result: OptionSnapshot[] = [];
    for (const [name, entry] of this.options) {
      // Skip aliases (shortName points to the main entry)
      if (entry.shortName && entry.name !== name) continue;
      if (seen.has(name)) continue;
      seen.add(name);
      if (all || entry.value !== entry.defaultValue) {
        result.push({
          name: entry.shortName ?? name,
          value: entry.value,
          defaultValue: entry.defaultValue,
          modified: entry.value !== entry.defaultValue,
        });
      }
    }
    return result;
  }

  private notify(name: string, value: unknown): void {
    const entry = this.options.get(name);
    if (!entry) return;
    for (const fn of entry.listeners) {
      try { fn(value); } catch (e) { console.error(`Option change listener error [${name}]:`, e); }
    }
    // Also notify shortName listeners
    if (entry.shortName) {
      const aliasEntry = this.options.get(entry.shortName);
      if (aliasEntry && aliasEntry !== entry) {
        for (const fn of aliasEntry.listeners) {
          try { fn(value); } catch (e) { console.error(`Option change listener error [${name}]:`, e); }
        }
      }
    }
    this.saveDebounced();
  }

  private saveTimer: ReturnType<typeof setTimeout> | null = null;
  private saveDebounced(): void {
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => this.save(), 200);
  }

  async save(): Promise<void> {
    const persisted: Record<string, unknown> = {};
    for (const [key, entry] of this.options) {
      if (!entry.persist) continue;
      if (key !== entry.name) continue;
      if (entry.value !== entry.defaultValue) {
        persisted[entry.name] = entry.value;
      }
    }
    try {
      await invoke('write_config', { options: persisted });
    } catch {
      // Config write failure is non-fatal
    }
  }

  async load(): Promise<void> {
    if (this.loaded) return;
    this.loaded = true;
    try {
      const data = await invoke<Record<string, unknown>>('read_config');
      if (!data) return;
      for (const [name, value] of Object.entries(data)) {
        const entry = this.options.get(name);
        if (entry && value !== undefined) {
          entry.value = value;
        }
      }
    } catch {
      // Config read failure is non-fatal — use defaults
    }
  }

  // Pre-load config at app startup. Once loaded, returns immediately.
  preload(): void {
    if (this.loaded) return;
    this.load();
  }

  isLoaded(): boolean {
    return this.loaded;
  }

  setupSetCommand(onOutput: (text: string) => void): void {
    if (this.setCommandRegistered) return;
    this.setCommandRegistered = true;

    Vim.defineEx('set', 'se', (_cm, params) => {
      const arg = params.argString.trim();
      const onResult = (text: string) => onOutput(text);

      if (arg === '') {
        // :set — list modified options
        const list = this.list(false);
        if (list.length === 0) {
          onResult('(-- no modified options --)');
        } else {
          onResult(list.map(o =>
            `${o.name}${o.value === true ? '' : '=' + o.value}`
          ).join('\n'));
        }
        return;
      }

      if (arg === 'all') {
        const list = this.list(true);
        if (list.length === 0) {
          onResult('(-- no options --)');
        } else {
          onResult(list.map(o =>
            `${o.name}=${o.value}`
          ).join('\n'));
        }
        return;
      }

      // Parse single option change
      let name = arg;
      let action: 'set' | 'unset' | 'toggle' | 'query' = 'set';

      // Check for ? suffix (query)
      if (name.endsWith('?')) {
        action = 'query';
        name = name.slice(0, -1);
      }
      // Check for ! suffix (toggle)
      else if (name.endsWith('!')) {
        action = 'toggle';
        name = name.slice(0, -1);
      }
      // Check for no prefix (unset: nomber)
      else if (name.startsWith('no')) {
        action = 'unset';
        name = name.slice(2);
      }
      // Check for inv prefix (toggle: invnumber)
      else if (name.startsWith('inv')) {
        action = 'toggle';
        name = name.slice(3);
      }

      if (!this.options.has(name)) {
        onResult(`E518: Unknown option: ${name}`);
        return;
      }

      const entry = this.options.get(name)!;
      const mainName = entry.shortName ?? entry.name;

      switch (action) {
        case 'query':
          onResult(`  ${mainName}=${entry.value}`);
          break;
        case 'toggle':
          if (entry.type !== 'boolean') {
            onResult(`E474: Invalid argument: ${name}`);
            return;
          }
          this.toggle(name);
          onResult(`  ${this.get<boolean>(name) ? '' : 'no'}${mainName}`);
          break;
        case 'unset':
          if (entry.type !== 'boolean') {
            onResult(`E474: Invalid argument: ${name}`);
            return;
          }
          this.set(name, false);
          onResult(`  no${mainName}`);
          break;
        case 'set':
          if (entry.type === 'boolean') {
            this.set(name, true);
          }
          onResult(`  ${mainName}`);
          break;
      }
    });
  }
}

export const vimOptions = new VimOptionStore();
