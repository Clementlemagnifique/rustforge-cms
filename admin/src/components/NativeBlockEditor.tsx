import { useState } from 'react';
import { BLOCK_TYPES, newBlock, moveBlock, insertBlockAfter, readDocument } from '../editor/document';
import type { BlockKind, NativeBlock, NativeDocument } from '../editor/document';
import { BLOCK_PATTERNS, insertPatternAfter, insertSyncedPatternAfter, detachPatternBlock } from '../editor/patterns';
import type { SyncedPattern } from '../editor/patterns';
import { editorMessages as text } from '../editor/messages.fr';
import type { Media } from '../types';
import { api } from '../api';

export function NativeBlockEditor({
  document,
  onChange,
  disabled,
  canUndo = false,
  canRedo = false,
  onUndo,
  onRedo,
  mediaLibrary = [],
  onUploadMedia,
}: {
  document: NativeDocument;
  onChange: (doc: NativeDocument) => void;
  disabled: boolean;
  canUndo?: boolean;
  canRedo?: boolean;
  onUndo?: () => void;
  onRedo?: () => void;
  mediaLibrary?: Media[];
  onUploadMedia?: (file: File) => Promise<Media | null>;
}) {
  const [pickerBlockId, setPickerBlockId] = useState<string | null>(null);
  const [uploadingBlockId, setUploadingBlockId] = useState<string | null>(null);
  const [showOutline, setShowOutline] = useState(false);
  const [showPatterns, setShowPatterns] = useState(false);
  const [patternTargetId, setPatternTargetId] = useState<string | undefined>(undefined);
  const [syncedPatterns, setSyncedPatterns] = useState<SyncedPattern[]>([]);
  const [patternTab, setPatternTab] = useState<'static' | 'synced'>('static');
  const [loadingPatterns, setLoadingPatterns] = useState(false);
  const [patternNotice, setPatternNotice] = useState<string | null>(null);
  const [inserterQuery, setInserterQuery] = useState('');

  function openPatterns(targetId?: string) {
    setPatternTargetId(targetId);
    setShowPatterns(true);
    setLoadingPatterns(true);
    api.patterns
      .list()
      .then(setSyncedPatterns)
      .catch(() => {})
      .finally(() => setLoadingPatterns(false));
  }

  async function handleCreateSyncedPatternFromBlock(block: NativeBlock) {
    const title = window.prompt(
      'Nom du motif synchronisé :',
      block.rf_caption || `Motif ${text.types[block.rf_type] || block.rf_type}`
    );
    if (!title || !title.trim()) return;
    try {
      const patternDoc = {
        rf_schema: 'rf.document/1',
        rf_blocks: [
          {
            ...block,
            rf_id: crypto.randomUUID(),
          },
        ],
      };
      const created = await api.patterns.create({
        title: title.trim(),
        document: patternDoc,
        is_synced: true,
        category: 'content',
      });
      const updatedBlocks = document.rf_blocks.map(b =>
        b.rf_id === block.rf_id
          ? {
              rf_id: b.rf_id,
              rf_type: 'pattern' as const,
              rf_ref: created.slug,
              rf_caption: created.title,
              rf_text: '',
              rf_level: 2,
              rf_ordered: false,
              rf_items: [b.rf_text || b.rf_caption || ''],
            }
          : b
      );
      onChange({ ...document, rf_blocks: updatedBlocks });
      setPatternNotice(`Motif « ${created.title} » créé et synchronisé.`);
      setTimeout(() => setPatternNotice(null), 4000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Erreur lors de la création du motif');
    }
  }

  async function handleDetachPattern(blockId: string, ref: string) {
    let found = syncedPatterns.find(p => p.slug === ref || p.id === ref);
    if (!found) {
      try {
        const all = await api.patterns.list();
        setSyncedPatterns(all);
        found = all.find(p => p.slug === ref || p.id === ref);
      } catch {}
    }
    const detached = detachPatternBlock(document, blockId, found);
    onChange(detached);
  }

  const filteredBlockTypes = BLOCK_TYPES.filter(kind =>
    text.types[kind].toLowerCase().includes(inserterQuery.trim().toLowerCase()) ||
    kind.toLowerCase().includes(inserterQuery.trim().toLowerCase())
  );

  function edit(id: string, patch: Partial<NativeBlock>) {
    onChange({
      ...document,
      rf_blocks: document.rf_blocks.map(block => (block.rf_id === id ? { ...block, ...patch } : block)),
    });
  }

  return (
    <fieldset
      disabled={disabled}
      className="min-w-0 space-y-4"
      onKeyDown={e => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
          if (e.shiftKey && onRedo && canRedo) {
            e.preventDefault();
            onRedo();
          } else if (!e.shiftKey && onUndo && canUndo) {
            e.preventDefault();
            onUndo();
          }
        } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'y' && onRedo && canRedo) {
          e.preventDefault();
          onRedo();
        }
      }}
    >
      <legend className="sr-only">{text.blocks}</legend>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-xs text-slate-400">{text.initial}</p>
        <div className="flex flex-wrap items-center gap-1.5 text-xs">
          <button
            type="button"
            onClick={() => setShowOutline(open => !open)}
            className={`rounded-lg border px-2.5 py-1 transition-colors ${
              showOutline
                ? 'border-indigo-400/40 bg-indigo-500/20 text-indigo-200'
                : 'border-white/10 bg-white/5 text-slate-200 hover:bg-white/10'
            }`}
          >
            ☰ {text.outlineTitle} ({document.rf_blocks.length})
          </button>
          <button
            type="button"
            onClick={() => openPatterns(undefined)}
            className="rounded-lg border border-purple-400/40 bg-purple-500/20 px-2.5 py-1 text-purple-200 hover:bg-purple-500/30"
          >
            ✦ {text.patterns}
          </button>
          {(onUndo || onRedo) && (
            <>
              <button
                type="button"
                disabled={!canUndo}
                onClick={onUndo}
                className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-slate-200 hover:bg-white/10 disabled:opacity-30"
              >
                ↶ {text.undo}
              </button>
              <button
                type="button"
                disabled={!canRedo}
                onClick={onRedo}
                className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-slate-200 hover:bg-white/10 disabled:opacity-30"
              >
                ↷ {text.redo}
              </button>
            </>
          )}
        </div>
      </div>

      {/* Modal / Tiroir des Motifs de blocs (WP-34) */}
      {patternNotice && (
        <div className="rounded-lg bg-emerald-500/20 border border-emerald-400/40 p-2.5 text-xs text-emerald-200">
          ✓ {patternNotice}
        </div>
      )}
      {showPatterns && (
        <div className="rounded-xl border border-purple-500/30 bg-purple-950/40 p-4 space-y-3">
          <div className="flex items-center justify-between">
            <h4 className="font-semibold text-purple-200 text-sm flex items-center gap-2">
              <span>✦</span> {text.patterns}
            </h4>
            <button
              type="button"
              onClick={() => setShowPatterns(false)}
              className="text-xs text-purple-300 hover:text-white"
            >
              ✕ Fermer
            </button>
          </div>
          <div className="flex items-center gap-2 border-b border-purple-500/20 pb-2">
            <button
              type="button"
              onClick={() => setPatternTab('static')}
              className={`px-2.5 py-1 text-xs rounded transition-colors ${
                patternTab === 'static'
                  ? 'bg-purple-600 text-white font-medium'
                  : 'text-purple-300 hover:bg-purple-900/40'
              }`}
            >
              Modèles prédéfinis ({BLOCK_PATTERNS.length})
            </button>
            <button
              type="button"
              onClick={() => setPatternTab('synced')}
              className={`px-2.5 py-1 text-xs rounded transition-colors ${
                patternTab === 'synced'
                  ? 'bg-purple-600 text-white font-medium'
                  : 'text-purple-300 hover:bg-purple-900/40'
              }`}
            >
              Motifs synchronisés ({syncedPatterns.length})
            </button>
          </div>

          {patternTab === 'static' ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 pt-1">
              {BLOCK_PATTERNS.map(p => (
                <div
                  key={p.id}
                  className="rounded-lg border border-white/10 bg-black/40 p-3 flex flex-col justify-between gap-2.5 hover:border-purple-400/50 transition-colors"
                >
                  <div>
                    <div className="font-medium text-slate-100 text-xs">{p.title}</div>
                    <div className="text-[11px] text-slate-400 mt-1">{p.description}</div>
                  </div>
                  <button
                    type="button"
                    disabled={document.rf_blocks.length + p.blocks.length > 256}
                    onClick={() => {
                      onChange(insertPatternAfter(document, patternTargetId, p));
                      setShowPatterns(false);
                    }}
                    className="rounded bg-purple-600 hover:bg-purple-500 px-2.5 py-1 text-xs font-medium text-white self-start transition-colors disabled:opacity-40"
                  >
                    + {text.insertPattern}
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-3 pt-1">
              {loadingPatterns && (
                <p className="text-xs text-purple-300/70">Chargement des motifs synchronisés…</p>
              )}
              {!loadingPatterns && syncedPatterns.length === 0 && (
                <p className="text-xs text-purple-300/80">
                  Aucun motif synchronisé enregistré. Créez-en un à tout moment depuis un bloc en cliquant sur « ✦ Motif ».
                </p>
              )}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {syncedPatterns.map(p => (
                  <div
                    key={p.id}
                    className="rounded-lg border border-purple-500/20 bg-black/40 p-3 flex flex-col justify-between gap-2.5 hover:border-purple-400/50 transition-colors"
                  >
                    <div>
                      <div className="flex items-center gap-1.5">
                        <span className="font-medium text-slate-100 text-xs">{p.title}</span>
                        <span className="rounded bg-purple-500/30 px-1 text-[10px] text-purple-200 uppercase font-mono">
                          {p.category}
                        </span>
                      </div>
                      {p.description && (
                        <div className="text-[11px] text-slate-400 mt-1">{p.description}</div>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-1.5">
                      <button
                        type="button"
                        disabled={document.rf_blocks.length >= 256}
                        onClick={() => {
                          onChange(insertSyncedPatternAfter(document, patternTargetId, p));
                          setShowPatterns(false);
                        }}
                        className="rounded bg-purple-600 hover:bg-purple-500 px-2 py-1 text-[11px] font-medium text-white transition-colors disabled:opacity-40"
                      >
                        + Synchronisé
                      </button>
                      <button
                        type="button"
                        onClick={() => {
                          const pDoc = readDocument(p.document);
                          const blocks = pDoc ? pDoc.rf_blocks : [];
                          onChange(
                            insertPatternAfter(document, patternTargetId, {
                              id: p.slug,
                              title: p.title,
                              description: p.description,
                              category: 'content',
                              blocks,
                            })
                          );
                          setShowPatterns(false);
                        }}
                        className="rounded border border-white/10 bg-white/5 hover:bg-white/10 px-2 py-1 text-[11px] text-slate-300 transition-colors"
                      >
                        + Détaché (copie)
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Vue liste / Plan des blocs (WP-31) */}
      {showOutline && (
        <div
          aria-label={text.outlineTitle}
          className="rounded-xl border border-indigo-500/20 bg-indigo-500/5 p-3 space-y-2 text-xs"
        >
          <div className="font-semibold text-indigo-300">{text.outlineTitle}</div>
          {document.rf_blocks.length === 0 ? (
            <p className="text-slate-400">{text.outlineEmpty}</p>
          ) : (
            <ol className="space-y-1">
              {document.rf_blocks.map((b, idx) => {
                const summary =
                  b.rf_type === 'list'
                    ? b.rf_items.filter(Boolean).join(' • ')
                    : b.rf_type === 'image'
                    ? b.rf_alt || b.rf_url || 'Image'
                    : b.rf_type === 'divider'
                    ? '—'
                    : b.rf_type === 'callout'
                    ? `[${b.rf_style || 'info'}] ${b.rf_text}`
                    : b.rf_type === 'details'
                    ? `▶ ${b.rf_caption || 'Détails'}`
                    : b.rf_type === 'columns'
                    ? `${b.rf_items.length} colonnes`
                    : b.rf_type === 'media_text'
                    ? `📷 ${b.rf_alt || b.rf_url || 'Média'} + Texte`
                    : b.rf_text;
                return (
                  <li
                    key={b.rf_id}
                    className="flex items-center justify-between gap-2 rounded-lg bg-black/30 px-2.5 py-1.5 border border-white/5"
                  >
                    <button
                      type="button"
                      onClick={() => {
                        const el = window.document.getElementById(`rf-block-${b.rf_id}`);
                        el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
                      }}
                      className="flex items-center gap-2 min-w-0 text-left hover:text-indigo-300"
                    >
                      <span className="font-mono text-[10px] text-slate-500">#{idx + 1}</span>
                      <span className="font-semibold text-slate-200">
                        {text.types[b.rf_type]}
                        {b.rf_type === 'heading' ? ` (H${b.rf_level})` : ''}
                      </span>
                      <span className="truncate text-slate-400 max-w-[220px]">
                        {summary || '…'}
                      </span>
                    </button>
                    <div className="flex items-center gap-1 shrink-0">
                      <button
                        type="button"
                        disabled={idx === 0}
                        onClick={() => onChange(moveBlock(document, b.rf_id, -1))}
                        className="rounded px-1.5 py-0.5 hover:bg-white/10 disabled:opacity-30"
                        title={text.up}
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        disabled={idx === document.rf_blocks.length - 1}
                        onClick={() => onChange(moveBlock(document, b.rf_id, 1))}
                        className="rounded px-1.5 py-0.5 hover:bg-white/10 disabled:opacity-30"
                        title={text.down}
                      >
                        ↓
                      </button>
                    </div>
                  </li>
                );
              })}
            </ol>
          )}
        </div>
      )}

      {document.rf_blocks.length === 0 && <p className="text-slate-400">{text.empty}</p>}

      {document.rf_blocks.map((block, index) => {
        const name = text.position.replace('{type}', text.types[block.rf_type]).replace('{n}', String(index + 1));
        const hasContent = Boolean(block.rf_text || block.rf_url || block.rf_items.some(Boolean));
        return (
          <section id={`rf-block-${block.rf_id}`} key={block.rf_id} aria-label={name} className="rounded-xl border border-white/10 bg-black/15 p-4 space-y-3">
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <span className="font-semibold mr-auto text-slate-300">{text.types[block.rf_type]}</span>
              <button
                type="button"
                disabled={index === 0}
                onClick={() => onChange(moveBlock(document, block.rf_id, -1))}
                className="disabled:opacity-30 rounded px-2 py-1 hover:bg-white/10"
              >
                {text.up}
              </button>
              <button
                type="button"
                disabled={index === document.rf_blocks.length - 1}
                onClick={() => onChange(moveBlock(document, block.rf_id, 1))}
                className="disabled:opacity-30 rounded px-2 py-1 hover:bg-white/10"
              >
                {text.down}
              </button>
              <select
                aria-label={`${text.insertAfter} (${name})`}
                disabled={document.rf_blocks.length >= 256}
                value=""
                onChange={e => {
                  const kind = e.target.value as BlockKind;
                  if (kind) onChange(insertBlockAfter(document, block.rf_id, kind));
                }}
                className="rounded bg-slate-800/80 border border-white/10 px-2 py-1 text-xs text-slate-300"
              >
                <option value="">+ {text.insertAfter}…</option>
                {BLOCK_TYPES.map(kind => (
                  <option key={kind} value={kind}>
                    {text.types[kind]}
                  </option>
                ))}
              </select>
              <button
                type="button"
                disabled={document.rf_blocks.length >= 256}
                onClick={() => {
                  const blocks = [...document.rf_blocks];
                  blocks.splice(index + 1, 0, {
                    ...block,
                    rf_id: crypto.randomUUID(),
                    rf_items: [...block.rf_items],
                  });
                  onChange({ ...document, rf_blocks: blocks });
                }}
                className="rounded px-2 py-1 hover:bg-white/10"
              >
                {text.duplicate}
              </button>
              {block.rf_type !== 'pattern' && (
                <button
                  type="button"
                  onClick={() => handleCreateSyncedPatternFromBlock(block)}
                  className="rounded px-2 py-1 text-purple-300 hover:bg-purple-500/10"
                  title={text.createSyncedPattern}
                >
                  ✦ Motif
                </button>
              )}
              <button
                type="button"
                onClick={() => {
                  if (hasContent && !window.confirm(text.removeConfirm)) return;
                  onChange({
                    ...document,
                    rf_blocks: document.rf_blocks.filter(b => b.rf_id !== block.rf_id),
                  });
                }}
                className="text-red-300 rounded px-2 py-1 hover:bg-white/10"
              >
                {text.remove}
              </button>
            </div>

            {block.rf_type === 'heading' && (
              <label className="text-xs flex gap-3 items-center">
                {text.level}
                <select
                  aria-label={text.level}
                  value={block.rf_level}
                  onChange={e => edit(block.rf_id, { rf_level: Number(e.target.value) })}
                  className="rounded bg-slate-800 p-2"
                >
                  {[1, 2, 3, 4, 5, 6].map(level => (
                    <option key={level} value={level}>
                      H{level}
                    </option>
                  ))}
                </select>
              </label>
            )}

            {block.rf_type === 'list' ? (
              <>
                <label className="text-xs flex gap-2">
                  <input
                    type="checkbox"
                    checked={block.rf_ordered}
                    onChange={e => edit(block.rf_id, { rf_ordered: e.target.checked })}
                  />
                  {text.ordered}
                </label>
                <label className="text-xs block">
                  {text.items}
                  <textarea
                    aria-label={name}
                    value={block.rf_items.join('\n')}
                    onChange={e => edit(block.rf_id, { rf_items: e.target.value.split('\n') })}
                    rows={4}
                    className="mt-2 w-full rounded-lg bg-black/20 p-3 text-base"
                  />
                </label>
              </>
            ) : block.rf_type === 'divider' ? (
              <hr className="border-white/20" />
            ) : block.rf_type === 'image' ? (
              <div className="space-y-3 text-xs">
                <div className="flex flex-wrap items-center gap-2">
                  {mediaLibrary.length > 0 && (
                    <button
                      type="button"
                      onClick={() => setPickerBlockId(pickerBlockId === block.rf_id ? null : block.rf_id)}
                      className="rounded-lg border border-indigo-400/30 bg-indigo-500/10 px-3 py-1.5 text-indigo-200 hover:bg-indigo-500/20"
                    >
                      {text.pickMedia}
                    </button>
                  )}
                  {onUploadMedia && (
                    <label className="cursor-pointer rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-slate-200 hover:bg-white/10">
                      {uploadingBlockId === block.rf_id ? 'Téléversement…' : text.uploadMedia}
                      <input
                        type="file"
                        accept="image/*"
                        className="sr-only"
                        disabled={uploadingBlockId === block.rf_id}
                        onChange={async e => {
                          const file = e.target.files?.[0];
                          if (!file || !onUploadMedia) return;
                          setUploadingBlockId(block.rf_id);
                          try {
                            const uploaded = await onUploadMedia(file);
                            if (uploaded) {
                              edit(block.rf_id, {
                                rf_url: uploaded.url || `/uploads/${uploaded.path}`,
                                rf_alt: block.rf_alt || uploaded.alt || uploaded.original_name,
                                rf_media_id: uploaded.id,
                              });
                            }
                          } finally {
                            setUploadingBlockId(null);
                            e.target.value = '';
                          }
                        }}
                      />
                    </label>
                  )}
                </div>

                {pickerBlockId === block.rf_id && mediaLibrary.length > 0 && (
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 rounded-lg border border-white/10 bg-black/30 p-2 max-h-48 overflow-y-auto">
                    {mediaLibrary
                      .filter(m => m.mime_type.startsWith('image/'))
                      .map(m => {
                        const mediaUrl = m.url || `/uploads/${m.path}`;
                        return (
                          <button
                            key={m.id}
                            type="button"
                            onClick={() => {
                              edit(block.rf_id, {
                                rf_url: mediaUrl,
                                rf_alt: block.rf_alt || m.alt || m.original_name,
                                rf_media_id: m.id,
                              });
                              setPickerBlockId(null);
                            }}
                            className="flex flex-col items-start gap-1 rounded border border-white/10 bg-white/5 p-2 text-left hover:border-indigo-400"
                          >
                            <span className="truncate w-full font-medium text-slate-200">{m.original_name}</span>
                            <span className="truncate w-full text-[10px] text-slate-400 font-mono">{mediaUrl}</span>
                          </button>
                        );
                      })}
                  </div>
                )}

                <label className="block space-y-1">
                  <span className="text-slate-400">{text.imageUrl}</span>
                  <input
                    type="text"
                    value={block.rf_url ?? ''}
                    onChange={e => edit(block.rf_id, { rf_url: e.target.value })}
                    placeholder="/uploads/2026/10/image.webp"
                    className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 font-mono text-xs text-white"
                  />
                </label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <label className="block space-y-1">
                    <span className="text-slate-400">{text.imageAlt}</span>
                    <input
                      type="text"
                      maxLength={512}
                      value={block.rf_alt ?? ''}
                      onChange={e => edit(block.rf_id, { rf_alt: e.target.value })}
                      placeholder="Description concise de l’image"
                      className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 text-xs text-white"
                    />
                  </label>
                  <label className="block space-y-1">
                    <span className="text-slate-400">{text.imageCaption}</span>
                    <input
                      type="text"
                      maxLength={1024}
                      value={block.rf_caption ?? ''}
                      onChange={e => edit(block.rf_id, { rf_caption: e.target.value })}
                      placeholder="Légende affichée sous l’image"
                      className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 text-xs text-white"
                    />
                  </label>
                </div>
              </div>
            ) : block.rf_type === 'button' ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                <label className="block space-y-1">
                  <span className="text-slate-400">{text.buttonLabel}</span>
                  <input
                    type="text"
                    maxLength={256}
                    value={block.rf_text}
                    onChange={e => edit(block.rf_id, { rf_text: e.target.value })}
                    placeholder="Ex: Découvrir le guide"
                    className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 text-xs text-white"
                  />
                </label>
                <label className="block space-y-1">
                  <span className="text-slate-400">{text.buttonUrl}</span>
                  <input
                    type="text"
                    maxLength={2048}
                    value={block.rf_url ?? ''}
                    onChange={e => edit(block.rf_id, { rf_url: e.target.value })}
                    placeholder="/contact ou https://…"
                    className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 font-mono text-xs text-white"
                  />
                </label>
              </div>
            ) : block.rf_type === 'callout' ? (
              <div className="space-y-2 text-xs">
                <label className="block space-y-1">
                  <span className="text-slate-400">{text.calloutStyle}</span>
                  <select
                    value={block.rf_style || 'info'}
                    onChange={e => edit(block.rf_id, { rf_style: e.target.value })}
                    className="rounded bg-slate-800 border border-white/10 p-2 text-xs text-white"
                  >
                    <option value="info">{text.calloutInfo}</option>
                    <option value="warning">{text.calloutWarning}</option>
                    <option value="success">{text.calloutSuccess}</option>
                    <option value="tip">{text.calloutTip}</option>
                  </select>
                </label>
                <div className="flex flex-wrap items-center gap-1 text-[11px] text-slate-400">
                  <button
                    type="button"
                    onClick={() => edit(block.rf_id, { rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}**texte en gras**` })}
                    className="rounded border border-white/10 bg-white/5 px-2 py-0.5 font-bold text-slate-200 hover:bg-white/10"
                    title="Insérer une emphase forte (Gras)"
                  >
                    B
                  </button>
                  <button
                    type="button"
                    onClick={() => edit(block.rf_id, { rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}*texte en italique*` })}
                    className="rounded border border-white/10 bg-white/5 px-2 py-0.5 italic text-slate-200 hover:bg-white/10"
                    title="Insérer une emphase (Italique)"
                  >
                    I
                  </button>
                  <button
                    type="button"
                    onClick={() => edit(block.rf_id, { rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}\`code\`` })}
                    className="rounded border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-slate-200 hover:bg-white/10"
                    title="Insérer du code en ligne"
                  >
                    {'<>'}
                  </button>
                </div>
                <textarea
                  aria-label={name}
                  maxLength={65536}
                  value={block.rf_text}
                  onChange={e => edit(block.rf_id, { rf_text: e.target.value })}
                  rows={3}
                  className="w-full rounded-lg bg-black/20 p-2.5 text-sm"
                />
              </div>
            ) : block.rf_type === 'details' ? (
              <div className="space-y-3 text-xs">
                <label className="block space-y-1">
                  <span className="text-slate-400">{text.detailsSummary}</span>
                  <input
                    type="text"
                    maxLength={1024}
                    value={block.rf_caption ?? ''}
                    onChange={e => edit(block.rf_id, { rf_caption: e.target.value })}
                    placeholder="Titre de l’accordéon (résumé cliquable)"
                    className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-2 text-xs text-white"
                  />
                </label>
                <label className="block space-y-1">
                  <span className="text-slate-400">Contenu développé</span>
                  <textarea
                    aria-label={name}
                    maxLength={65536}
                    value={block.rf_text}
                    onChange={e => edit(block.rf_id, { rf_text: e.target.value })}
                    rows={4}
                    className="w-full rounded-lg bg-black/20 p-2.5 text-sm"
                  />
                </label>
              </div>
            ) : block.rf_type === 'columns' ? (
              <div className="space-y-3 text-xs">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <span className="text-slate-400">{text.columnsLayout} :</span>
                    <select
                      value={block.rf_style || (block.rf_items.length === 3 ? '33-33-33' : '50-50')}
                      onChange={e => edit(block.rf_id, { rf_style: e.target.value })}
                      className="rounded bg-slate-800 border border-white/10 p-1 text-xs text-white"
                    >
                      <option value="50-50">{text.columnsLayoutBalanced}</option>
                      <option value="33-33-33">{text.columnsLayoutThree}</option>
                      <option value="25-75">{text.columnsLayoutSidebarLeft}</option>
                      <option value="75-25">{text.columnsLayoutSidebarRight}</option>
                      <option value="wide">{text.columnsLayoutWide}</option>
                    </select>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-slate-400">{text.columnsCount.replace('{count}', String(block.rf_items.length))}</span>
                    <div className="flex gap-2">
                    {block.rf_items.length < 4 && (
                      <button
                        type="button"
                        onClick={() => edit(block.rf_id, { rf_items: [...block.rf_items, `Colonne ${block.rf_items.length + 1}`] })}
                        className="rounded border border-indigo-400/30 bg-indigo-500/10 px-2 py-1 text-xs text-indigo-200 hover:bg-indigo-500/20"
                      >
                        + {text.addColumn}
                      </button>
                    )}
                    {block.rf_items.length > 1 && (
                      <button
                        type="button"
                        onClick={() => edit(block.rf_id, { rf_items: block.rf_items.slice(0, -1) })}
                        className="rounded border border-white/10 bg-white/5 px-2 py-1 text-xs text-slate-300 hover:bg-white/10"
                      >
                        - {text.removeColumn}
                      </button>
                    )}
                    </div>
                  </div>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {block.rf_items.map((colContent, colIdx) => (
                    <div key={colIdx} className="rounded-lg border border-white/10 bg-black/20 p-2.5 space-y-1">
                      <span className="font-semibold text-slate-300">Colonne {colIdx + 1}</span>
                      <textarea
                        value={colContent}
                        onChange={e => {
                          const updated = [...block.rf_items];
                          updated[colIdx] = e.target.value;
                          edit(block.rf_id, { rf_items: updated });
                        }}
                        rows={4}
                        className="w-full rounded bg-transparent p-1.5 text-xs text-white"
                      />
                    </div>
                  ))}
                </div>
              </div>
            ) : block.rf_type === 'media_text' ? (
              <div className="space-y-3 text-xs">
                <div className="flex flex-wrap items-center gap-3">
                  <label className="flex items-center gap-2">
                    <span className="text-slate-400">{text.mediaPosition}</span>
                    <select
                      value={block.rf_style || 'media-left'}
                      onChange={e => edit(block.rf_id, { rf_style: e.target.value })}
                      className="rounded bg-slate-800 border border-white/10 p-1.5 text-xs text-white"
                    >
                      <option value="media-left">{text.mediaLeft}</option>
                      <option value="media-right">{text.mediaRight}</option>
                    </select>
                  </label>
                  {mediaLibrary.length > 0 && (
                    <button
                      type="button"
                      onClick={() => setPickerBlockId(pickerBlockId === block.rf_id ? null : block.rf_id)}
                      className="rounded border border-indigo-400/30 bg-indigo-500/10 px-2.5 py-1 text-indigo-200 hover:bg-indigo-500/20"
                    >
                      {text.pickMedia}
                    </button>
                  )}
                </div>

                {pickerBlockId === block.rf_id && mediaLibrary.length > 0 && (
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 rounded-lg border border-white/10 bg-black/30 p-2 max-h-48 overflow-y-auto">
                    {mediaLibrary
                      .filter(m => m.mime_type.startsWith('image/'))
                      .map(m => {
                        const mediaUrl = m.url || `/uploads/${m.path}`;
                        return (
                          <button
                            key={m.id}
                            type="button"
                            onClick={() => {
                              edit(block.rf_id, {
                                rf_url: mediaUrl,
                                rf_alt: block.rf_alt || m.alt || m.original_name,
                                rf_media_id: m.id,
                              });
                              setPickerBlockId(null);
                            }}
                            className="flex flex-col items-start gap-1 rounded border border-white/10 bg-white/5 p-2 text-left hover:border-indigo-400"
                          >
                            <span className="truncate w-full font-medium text-slate-200">{m.original_name}</span>
                            <span className="truncate w-full text-[10px] text-slate-400 font-mono">{mediaUrl}</span>
                          </button>
                        );
                      })}
                  </div>
                )}

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <label className="block space-y-1">
                    <span className="text-slate-400">{text.imageUrl}</span>
                    <input
                      type="text"
                      value={block.rf_url ?? ''}
                      onChange={e => edit(block.rf_id, { rf_url: e.target.value })}
                      placeholder="/uploads/photo.webp"
                      className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-1.5 font-mono text-xs text-white"
                    />
                  </label>
                  <label className="block space-y-1">
                    <span className="text-slate-400">{text.imageAlt}</span>
                    <input
                      type="text"
                      maxLength={512}
                      value={block.rf_alt ?? ''}
                      onChange={e => edit(block.rf_id, { rf_alt: e.target.value })}
                      placeholder="Description de l’image"
                      className="w-full rounded-lg border border-white/10 bg-black/25 px-3 py-1.5 text-xs text-white"
                    />
                  </label>
                </div>
                <label className="block space-y-1">
                  <span className="text-slate-400">Texte d’accompagnement</span>
                  <textarea
                    aria-label={name}
                    maxLength={65536}
                    value={block.rf_text}
                    onChange={e => edit(block.rf_id, { rf_text: e.target.value })}
                    rows={4}
                    className="w-full rounded-lg bg-black/20 p-2.5 text-sm"
                  />
                </label>
              </div>
            ) : block.rf_type === 'pattern' ? (
              <div className="space-y-3 rounded-lg border border-purple-500/30 bg-purple-950/20 p-3 text-xs">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <span className="rounded bg-purple-500/20 border border-purple-400/40 px-2 py-0.5 font-medium text-purple-200">
                      ✦ {text.syncedPatternBadge} : {block.rf_caption || block.rf_ref}
                    </span>
                    <span className="font-mono text-[11px] text-purple-300/70">
                      ref: {block.rf_ref}
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleDetachPattern(block.rf_id, block.rf_ref || '')}
                    className="rounded border border-purple-400/30 bg-purple-500/10 px-2.5 py-1 text-purple-200 hover:bg-purple-500/20"
                  >
                    {text.detachPattern}
                  </button>
                </div>
                <p className="text-[11px] text-purple-300/80">
                  {text.patternSyncedNotice}
                </p>
                {block.rf_items && block.rf_items.length > 0 && (
                  <div className="space-y-1 rounded bg-black/30 p-2 font-mono text-[11px] text-slate-300">
                    <span className="text-slate-400">Aperçu du contenu synchronisé :</span>
                    <ul className="list-disc pl-4 space-y-0.5">
                      {block.rf_items.map((item, i) => (
                        <li key={i} className="truncate">{item}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            ) : (
              <div className="space-y-1.5">
                {(block.rf_type === 'paragraph' || block.rf_type === 'quote') && (
                  <div className="flex flex-wrap items-center gap-1 text-[11px] text-slate-400">
                    <button
                      type="button"
                      onClick={() =>
                        edit(block.rf_id, {
                          rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}**texte en gras**`,
                        })
                      }
                      className="rounded border border-white/10 bg-white/5 px-2 py-0.5 font-bold text-slate-200 hover:bg-white/10"
                      title="Insérer une emphase forte (Gras)"
                    >
                      B
                    </button>
                    <button
                      type="button"
                      onClick={() =>
                        edit(block.rf_id, {
                          rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}*texte en italique*`,
                        })
                      }
                      className="rounded border border-white/10 bg-white/5 px-2 py-0.5 italic text-slate-200 hover:bg-white/10"
                      title="Insérer une emphase (Italique)"
                    >
                      I
                    </button>
                    <button
                      type="button"
                      onClick={() =>
                        edit(block.rf_id, {
                          rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}\`code\``,
                        })
                      }
                      className="rounded border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-slate-200 hover:bg-white/10"
                      title="Insérer du code en ligne"
                    >
                      {'<>'}
                    </button>
                    <button
                      type="button"
                      onClick={() =>
                        edit(block.rf_id, {
                          rf_text: `${block.rf_text}${block.rf_text ? ' ' : ''}[libellé du lien](/page)`,
                        })
                      }
                      className="rounded border border-white/10 bg-white/5 px-2 py-0.5 text-indigo-300 hover:bg-white/10"
                      title="Insérer un lien interne ou externe"
                    >
                      🔗 Lien
                    </button>
                  </div>
                )}
                <textarea
                  aria-label={name}
                  maxLength={65536}
                  value={block.rf_text}
                  onChange={e => edit(block.rf_id, { rf_text: e.target.value })}
                  rows={block.rf_type === 'heading' ? 1 : block.rf_type === 'code' ? 7 : 4}
                  className={`w-full rounded-lg bg-transparent p-2 leading-relaxed focus:outline-2 focus:outline-indigo-400 ${
                    block.rf_type === 'heading' ? 'text-2xl font-semibold' : block.rf_type === 'code' ? 'font-mono text-sm' : 'text-base'
                  }`}
                />
              </div>
            )}
          </section>
        );
      })}

      <div className="space-y-2.5 border-t border-white/10 pt-4" aria-label={text.add}>
        <input
          type="search"
          aria-label={text.inserterSearch}
          value={inserterQuery}
          onChange={e => setInserterQuery(e.target.value)}
          placeholder={text.inserterSearch}
          className="w-full sm:w-64 rounded-lg border border-white/10 bg-black/25 px-3 py-1.5 text-xs text-white placeholder-slate-500"
        />
        <div className="flex flex-wrap gap-2">
          {filteredBlockTypes.map(kind => (
            <button
              key={kind}
              type="button"
              disabled={document.rf_blocks.length >= 256}
              onClick={() => onChange({ ...document, rf_blocks: [...document.rf_blocks, newBlock(kind)] })}
              className="rounded-lg border border-indigo-400/30 px-3 py-2 text-sm text-indigo-200 hover:bg-indigo-500/10 disabled:opacity-30"
            >
              + {text.types[kind]}
            </button>
          ))}
        </div>
      </div>
      {document.rf_blocks.length >= 256 && <p className="text-amber-300 text-xs">{text.limit}</p>}
    </fieldset>
  );
}
