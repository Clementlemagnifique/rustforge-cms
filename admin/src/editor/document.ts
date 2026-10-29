export const DOCUMENT_SCHEMA = 'rf.document/1' as const;
export const BLOCK_TYPES = [
  'paragraph',
  'heading',
  'quote',
  'code',
  'list',
  'divider',
  'image',
  'button',
  'callout',
  'details',
  'columns',
  'media_text',
  'pattern',
  'table',
  'spacer',
  'cover',
  'group',
  'product',
] as const;
export type BlockKind = typeof BLOCK_TYPES[number];
export interface NativeBlock {
  rf_id: string; rf_type: BlockKind; rf_text: string; rf_level: number;
  rf_ordered: boolean; rf_items: string[];
  rf_url?: string; rf_alt?: string; rf_caption?: string; rf_media_id?: string;
  rf_style?: string; rf_ref?: string;
}
export interface NativeDocument { rf_schema: typeof DOCUMENT_SCHEMA; rf_blocks: NativeBlock[] }

const ALLOWED_BLOCK_KEYS = new Set([
  'rf_id', 'rf_type', 'rf_text', 'rf_level', 'rf_ordered', 'rf_items',
  'rf_url', 'rf_alt', 'rf_caption', 'rf_media_id', 'rf_style', 'rf_ref',
]);

export function newBlock(kind: BlockKind): NativeBlock {
  const base: NativeBlock = { rf_id: crypto.randomUUID(), rf_type: kind, rf_text: '', rf_level: 2, rf_ordered: false, rf_items: [] };
  if (kind === 'image') {
    base.rf_url = '';
    base.rf_alt = '';
    base.rf_caption = '';
  } else if (kind === 'button') {
    base.rf_url = '';
    base.rf_style = 'primary';
  } else if (kind === 'quote') {
    base.rf_caption = '';
  } else if (kind === 'callout') {
    base.rf_style = 'info';
  } else if (kind === 'details') {
    base.rf_caption = 'Détails';
  } else if (kind === 'columns') {
    base.rf_items = ['Colonne 1', 'Colonne 2'];
  } else if (kind === 'media_text') {
    base.rf_url = '';
    base.rf_alt = '';
    base.rf_style = 'media-left';
  } else if (kind === 'pattern') {
    base.rf_ref = '';
    base.rf_caption = 'Motif synchronisé';
  } else if (kind === 'table') {
    base.rf_items = ['Colonne 1 | Colonne 2', 'Valeur 1 | Valeur 2'];
    base.rf_caption = '';
  } else if (kind === 'spacer') {
    base.rf_style = 'md';
  } else if (kind === 'cover') {
    base.rf_url = '';
    base.rf_alt = '';
    base.rf_caption = '';
    base.rf_style = 'dark';
    base.rf_text = '# Titre principal';
  } else if (kind === 'group') {
    base.rf_style = 'card';
    base.rf_text = 'Contenu de la carte encadrée';
  } else if (kind === 'product') {
    base.rf_style = 'card';
    base.rf_text = 'Produit numérique';
    base.rf_caption = '29,00 €';
    base.rf_ref = '';
    base.rf_url = '';
    base.rf_alt = 'Acheter';
  }
  return base;
}
export function newDocument(): NativeDocument { return { rf_schema: DOCUMENT_SCHEMA, rf_blocks: [newBlock('paragraph')] }; }

export function readDocument(value: unknown): NativeDocument | null {
  if (!value || typeof value !== 'object') return null;
  const doc = value as Record<string, unknown>;
  if (doc.rf_schema !== DOCUMENT_SCHEMA || !Array.isArray(doc.rf_blocks) || doc.rf_blocks.length > 256 || Object.keys(doc).some(k => !['rf_schema', 'rf_blocks'].includes(k))) return null;
  const ids = new Set<string>();
  const result: NativeBlock[] = [];
  for (const candidate of doc.rf_blocks) {
    if (!candidate || typeof candidate !== 'object') return null;
    const block = candidate as Record<string, unknown>;
    if (typeof block.rf_id !== 'string' || !/^[a-zA-Z0-9_-]{1,64}$/.test(block.rf_id) || ids.has(block.rf_id)
      || !BLOCK_TYPES.includes(block.rf_type as BlockKind)
      || Object.keys(block).some(k => !ALLOWED_BLOCK_KEYS.has(k))
      || (block.rf_text !== undefined && typeof block.rf_text !== 'string')
      || (block.rf_level !== undefined && (!Number.isInteger(block.rf_level) || Number(block.rf_level) < 1 || Number(block.rf_level) > 6))
      || (block.rf_ordered !== undefined && typeof block.rf_ordered !== 'boolean')
      || (block.rf_items !== undefined && (!Array.isArray(block.rf_items) || !block.rf_items.every(v => typeof v === 'string')))
      || (block.rf_url !== undefined && typeof block.rf_url !== 'string')
      || (block.rf_alt !== undefined && typeof block.rf_alt !== 'string')
      || (block.rf_caption !== undefined && typeof block.rf_caption !== 'string')
      || (block.rf_media_id !== undefined && block.rf_media_id !== null && typeof block.rf_media_id !== 'string')
      || (block.rf_style !== undefined && (typeof block.rf_style !== 'string' || block.rf_style.length > 128))
      || (block.rf_ref !== undefined && (typeof block.rf_ref !== 'string' || block.rf_ref.length > 128))) return null;
    ids.add(block.rf_id);
    const kind = block.rf_type as BlockKind;
    const parsed: NativeBlock = {
      rf_id: block.rf_id,
      rf_type: kind,
      rf_text: (block.rf_text as string) ?? '',
      rf_level: (block.rf_level as number) ?? 2,
      rf_ordered: (block.rf_ordered as boolean) ?? false,
      rf_items: (block.rf_items as string[]) ?? [],
    };
    if (typeof block.rf_url === 'string') parsed.rf_url = block.rf_url;
    else if (kind === 'image' || kind === 'button' || kind === 'media_text' || kind === 'cover') parsed.rf_url = '';
    if (typeof block.rf_alt === 'string') parsed.rf_alt = block.rf_alt;
    else if (kind === 'image' || kind === 'media_text' || kind === 'cover') parsed.rf_alt = '';
    if (typeof block.rf_caption === 'string') parsed.rf_caption = block.rf_caption;
    else if (kind === 'image' || kind === 'quote' || kind === 'table' || kind === 'cover') parsed.rf_caption = '';
    else if (kind === 'details') parsed.rf_caption = 'Détails';
    if (typeof block.rf_media_id === 'string' && block.rf_media_id) parsed.rf_media_id = block.rf_media_id;
    if (typeof block.rf_ref === 'string') parsed.rf_ref = block.rf_ref;
    if (typeof block.rf_style === 'string') parsed.rf_style = block.rf_style;
    else if (kind === 'callout') parsed.rf_style = 'info';
    else if (kind === 'button') parsed.rf_style = 'primary';
    else if (kind === 'media_text') parsed.rf_style = 'media-left';
    else if (kind === 'spacer') parsed.rf_style = 'md';
    else if (kind === 'cover') parsed.rf_style = 'dark';
    else if (kind === 'group') parsed.rf_style = 'card';
    if (kind === 'columns' && (!Array.isArray(block.rf_items) || block.rf_items.length === 0)) {
      parsed.rf_items = ['Colonne 1', 'Colonne 2'];
    }
    if (kind === 'table' && (!Array.isArray(block.rf_items) || block.rf_items.length === 0)) {
      parsed.rf_items = ['Colonne 1 | Colonne 2', 'Valeur 1 | Valeur 2'];
    }
    result.push(parsed);
  }
  return { rf_schema: DOCUMENT_SCHEMA, rf_blocks: result };
}

export function moveBlock(doc: NativeDocument, id: string, direction: -1 | 1): NativeDocument {
  const index = doc.rf_blocks.findIndex(block => block.rf_id === id);
  const target = index + direction;
  if (index < 0 || target < 0 || target >= doc.rf_blocks.length) return doc;
  const blocks = [...doc.rf_blocks]; [blocks[index], blocks[target]] = [blocks[target], blocks[index]];
  return { ...doc, rf_blocks: blocks };
}

export function insertBlockAfter(doc: NativeDocument, afterId: string, kind: BlockKind): NativeDocument {
  if (doc.rf_blocks.length >= 256) return doc;
  const index = doc.rf_blocks.findIndex(block => block.rf_id === afterId);
  const created = newBlock(kind);
  if (index < 0) return { ...doc, rf_blocks: [...doc.rf_blocks, created] };
  const blocks = [...doc.rf_blocks];
  blocks.splice(index + 1, 0, created);
  return { ...doc, rf_blocks: blocks };
}

export interface DocumentHistory {
  past: NativeDocument[];
  present: NativeDocument;
  future: NativeDocument[];
}

export function createHistory(initial: NativeDocument): DocumentHistory {
  return { past: [], present: initial, future: [] };
}

export function pushHistory(history: DocumentHistory, next: NativeDocument): DocumentHistory {
  if (JSON.stringify(history.present) === JSON.stringify(next)) return history;
  const past = [...history.past, history.present].slice(-50);
  return { past, present: next, future: [] };
}

export function undoHistory(history: DocumentHistory): DocumentHistory {
  if (history.past.length === 0) return history;
  const previous = history.past[history.past.length - 1];
  return {
    past: history.past.slice(0, -1),
    present: previous,
    future: [history.present, ...history.future].slice(0, 50),
  };
}

export function redoHistory(history: DocumentHistory): DocumentHistory {
  if (history.future.length === 0) return history;
  const [next, ...rest] = history.future;
  return {
    past: [...history.past, history.present].slice(-50),
    present: next,
    future: rest,
  };
}

export interface AutosaveDraft {
  rf_autosave_version: 1;
  saved_at: string;
  base_updated_at?: string;
  title: string;
  slug: string;
  excerpt: string;
  body: string;
  blocks: unknown;
  status: string;
  featured_media_id?: string | null;
  parent_id?: string | null;
  published_at?: string | null;
  scheduled_unpublish_at?: string | null;
  term_ids?: string[];
}

export function autosaveStorageKey(contentType: string, entryId?: string): string {
  return `rf.autosave.v1:${contentType}:${entryId || 'new'}`;
}

export function parseAutosaveDraft(raw: string | null, serverUpdatedAt?: string): AutosaveDraft | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    if (parsed.rf_autosave_version !== 1 || typeof parsed.saved_at !== 'string' || typeof parsed.title !== 'string') return null;
    if (serverUpdatedAt && typeof parsed.base_updated_at === 'string' && parsed.base_updated_at !== serverUpdatedAt) {
      if (Date.parse(serverUpdatedAt) > Date.parse(parsed.saved_at)) return null;
    }
    return parsed as unknown as AutosaveDraft;
  } catch {
    return null;
  }
}
