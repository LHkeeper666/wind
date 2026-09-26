import { invoke } from '@tauri-apps/api/core';
import type { CompletionSource, Completion } from '@codemirror/autocomplete';
import { logError } from '../utils/log';

interface ApiMember {
  name: string;
  signature: string | null;
  kind: string;
}

interface PackageApi {
  name: string;
  version: string;
  members: ApiMember[];
}

// In-memory cache: package_name → PackageApi
const apiCache = new Map<string, PackageApi>();
// Pending requests to avoid duplicate invokes
const pendingRequests = new Map<string, Promise<PackageApi | null>>();

async function loadPackageApi(packageName: string): Promise<PackageApi | null> {
  const cached = apiCache.get(packageName);
  if (cached) return cached;

  const pending = pendingRequests.get(packageName);
  if (pending) return pending;

  const promise = invoke<PackageApi>('get_package_api', {
    packageName,
    pythonExe: null,
    forceRefresh: false,
  })
    .then((api) => {
      apiCache.set(packageName, api);
      pendingRequests.delete(packageName);
      return api;
    })
    .catch((err) => {
      pendingRequests.delete(packageName);
      logError('python-completion', `Failed to load API for ${packageName}: ${err}`);
      return null;
    });

  pendingRequests.set(packageName, promise);
  return promise;
}

function parseImports(docText: string): Map<string, string> {
  const aliasToPackage = new Map<string, string>();

  // import numpy → { numpy: "numpy" }
  // import numpy as np → { np: "numpy" }
  // import numpy, pandas → { numpy: "numpy", pandas: "pandas" }
  const importRe = /^import\s+(.+)$/gm;
  for (const m of docText.matchAll(importRe)) {
    const parts = m[1].split(',');
    for (const part of parts) {
      const trimmed = part.trim();
      if (!trimmed) continue;
      const asMatch = trimmed.match(/^(\S+)\s+as\s+(\S+)$/);
      if (asMatch) {
        aliasToPackage.set(asMatch[2], asMatch[1]);
      } else {
        // Top-level module: alias same as package name
        aliasToPackage.set(trimmed, trimmed);
      }
    }
  }

  // from numpy import array → { array: "numpy" }
  // from numpy import array as arr → { arr: "numpy" }
  // from numpy import array, zeros → { array: "numpy", zeros: "numpy" }
  const fromRe = /^from\s+(\S+)\s+import\s+(.+)$/gm;
  for (const m of docText.matchAll(fromRe)) {
    const packageName = m[1];
    const parts = m[2].split(',');
    for (const part of parts) {
      const trimmed = part.trim();
      if (!trimmed) continue;
      const asMatch = trimmed.match(/^(\S+)\s+as\s+(\S+)$/);
      if (asMatch) {
        aliasToPackage.set(asMatch[2], packageName);
      } else {
        aliasToPackage.set(trimmed, packageName);
      }
    }
  }

  return aliasToPackage;
}

export const pythonCompletionSource: CompletionSource = async (context) => {
  const text = context.state.doc.toString();
  const pos = context.pos;

  // Search backwards from cursor to find the dot of an attribute access
  // e.g. cursor after "numpy." or "numpy.ar" → find the dot after "numpy"
  let dotIdx = -1;
  for (let i = pos - 1; i >= 0; i--) {
    if (text[i] === '.') {
      // Check there's an identifier before the dot
      let j = i - 1;
      while (j >= 0 && /[a-zA-Z0-9_]/.test(text[j])) j--;
      if (j < i - 1) {
        dotIdx = i;
      }
      break;
    }
    if (!/[a-zA-Z0-9_]/.test(text[i])) break;
  }
  if (dotIdx < 0) return null;

  // Extract the identifier before the dot
  let wordStart = dotIdx - 1;
  while (wordStart >= 0 && /[a-zA-Z0-9_]/.test(text[wordStart])) wordStart--;
  const alias = text.slice(wordStart + 1, dotIdx);
  if (!alias) return null;

  const imports = parseImports(text);
  const packageName = imports.get(alias);
  if (!packageName) return null;

  const api = await loadPackageApi(packageName);
  if (!api) return null;

  const completions: Completion[] = api.members.map((m) => {
    const detail = m.signature ? `${m.name}${m.signature}` : m.name;
    return {
      label: m.name,
      type: m.kind === 'function' || m.kind === 'builtin_function_or_method' ? 'function'
        : m.kind === 'type' || m.kind === 'ABCMeta' ? 'class'
        : m.kind === 'module' ? 'namespace'
        : 'variable',
      detail,
      boost: m.name.startsWith('__') ? -10 : 0,
    };
  });

  return {
    from: dotIdx + 1,
    options: completions,
    validFor: /^\w*$/,
  };
};
