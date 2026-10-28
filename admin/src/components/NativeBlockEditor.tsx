import { useState } from 'react';
import { BLOCK_TYPES, newBlock, moveBlock, insertBlockAfter } from '../editor/document';
import type { BlockKind, NativeBlock, NativeDocument } from '../editor/document';
import { editorMessages as text } from '../editor/messages.fr';
import type { Media } from '../types';

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
  const [inserterQuery, setInserterQuery] = useState('');

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
