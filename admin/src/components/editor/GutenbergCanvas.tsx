/**
 * ============================================================================
 * CANEVAS CENTRAL D'ÉCRITURE GUTENBERG (`GutenbergCanvas`)
 * ============================================================================
 * Reproduit à 100% le canevas de composition officiel de Gutenberg :
 * - Grand champ Titre H1 sans bordure (« Ajouter un titre »).
 * - Séquence de blocs interactifs avec barre d'outils flottante au focus.
 * - Réorganisation par flèches Monter/Descendre et suppression directe.
 * - Sélection active synchronisée avec l'inspecteur latéral droit (Onglet Bloc).
 * - Bascule adaptative responsive selon le mode d'aperçu (Bureau / Tablette / Mobile).
 * - Déclencheur inférieur « Tapez / pour choisir un bloc ».
 */

import React, { useRef } from 'react';
import {
  Plus,
  ArrowUp,
  ArrowDown,
  Trash2,
  Copy,
  Image as ImageIcon,
  Bold,
  Italic,
} from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';
import { insertBlockAfter, insertCustomBlockAfter, moveBlock } from '../../editor/document';
import type { NativeBlock, NativeDocument } from '../../editor/document';

export const GutenbergCanvas: React.FC = () => {
  const title = useEditorStore((s) => s.title);
  const setTitle = useEditorStore((s) => s.setTitle);
  const docHistory = useEditorStore((s) => s.docHistory);
  const pushBlocks = useEditorStore((s) => s.pushBlocks);
  const selectedBlockId = useEditorStore((s) => s.selectedBlockId);
  const setSelectedBlockId = useEditorStore((s) => s.setSelectedBlockId);
  const toggleInserter = useEditorStore((s) => s.toggleInserter);
  const previewDevice = useEditorStore((s) => s.previewDevice);

  const titleInputRef = useRef<HTMLInputElement>(null);
  const document: NativeDocument = docHistory?.present ?? {
    rf_schema: 'rf.document/1',
    rf_blocks: [],
  };

  const handleUpdateBlock = (blockId: string, patch: Partial<NativeBlock>) => {
    const updated = document.rf_blocks.map((b) =>
      b.rf_id === blockId ? { ...b, ...patch } : b
    );
    pushBlocks({ ...document, rf_blocks: updated });
  };

  const handleRemoveBlock = (blockId: string) => {
    const updated = document.rf_blocks.filter((b) => b.rf_id !== blockId);
    pushBlocks({ ...document, rf_blocks: updated });
    if (selectedBlockId === blockId) {
      setSelectedBlockId(null);
    }
  };

  const handleMoveBlock = (blockId: string, direction: 'up' | 'down') => {
    const dir = direction === 'up' ? -1 : 1;
    const updated = moveBlock(document, blockId, dir);
    pushBlocks(updated);
  };

  const handleDuplicateBlock = (block: NativeBlock) => {
    const duplicate: NativeBlock = {
      ...block,
      rf_id: crypto.randomUUID(),
    };
    const updated = insertCustomBlockAfter(document, block.rf_id, duplicate);
    pushBlocks(updated);
    setSelectedBlockId(duplicate.rf_id);
  };

  const handleAddParagraphAtEnd = () => {
    const updated = insertBlockAfter(document, undefined, 'paragraph');
    pushBlocks(updated);
    const last = updated.rf_blocks[updated.rf_blocks.length - 1];
    if (last) setSelectedBlockId(last.rf_id);
  };

  const getDeviceContainerClass = () => {
    if (previewDevice === 'mobile') return 'max-w-[375px] shadow-2xl border-x border-[#dcdcde] dark:border-[#2c3338] my-4 rounded-xl';
    if (previewDevice === 'tablet') return 'max-w-[768px] shadow-2xl border-x border-[#dcdcde] dark:border-[#2c3338] my-4 rounded-xl';
    return 'w-full max-w-4xl';
  };

  return (
    <div className="flex-1 overflow-y-auto bg-[#f0f0f1] dark:bg-[#101520] flex justify-center py-6 px-4 transition-all">
      <div
        className={`${getDeviceContainerClass()} bg-white dark:bg-[#182030] min-h-[calc(100vh-120px)] p-8 sm:p-14 shadow-xs transition-all`}
        onClick={() => {
          // Clic sur le canevas en dehors d'un bloc désélectionne
        }}
      >
        {/* ================================================================== */}
        {/* GRAND TITRE H1 SANS BORDURE                                        */}
        {/* ================================================================== */}
        <div className="mb-8">
          <input
            ref={titleInputRef}
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Ajouter un titre"
            className="w-full text-3xl sm:text-4xl font-serif font-bold text-[#1d2327] dark:text-white placeholder:text-slate-300 dark:placeholder:text-slate-600 outline-none bg-transparent leading-tight border-b border-transparent focus:border-slate-200 dark:focus:border-[#2c3338] pb-2 transition-colors"
          />
        </div>

        {/* ================================================================== */}
        {/* SÉQUENCE DE BLOCS INTERACTIFS                                      */}
        {/* ================================================================== */}
        <div className="space-y-4">
          {document.rf_blocks.length === 0 ? (
            <div className="py-12 text-center text-slate-400">
              <p className="text-sm font-sans mb-3">Le document est vide pour l'instant.</p>
              <button
                type="button"
                onClick={handleAddParagraphAtEnd}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white text-xs font-medium transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Commencer à écrire</span>
              </button>
            </div>
          ) : (
            document.rf_blocks.map((block, index) => {
              const isSelected = selectedBlockId === block.rf_id;
              return (
                <div
                  key={block.rf_id}
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedBlockId(block.rf_id);
                  }}
                  className={`relative group rounded-md transition-all ${
                    isSelected
                      ? 'ring-1 ring-[#2271b1] bg-blue-50/20 dark:bg-blue-950/20 p-2'
                      : 'hover:bg-slate-50 dark:hover:bg-[#1a2332] p-2'
                  }`}
                >
                  {/* Barre d'outils contextuelle flottante au-dessus du bloc */}
                  {isSelected && (
                    <div className="absolute -top-10 left-0 bg-white dark:bg-[#1d2327] border border-[#dcdcde] dark:border-[#2c3338] rounded shadow-lg px-2 py-1 flex items-center gap-1 z-30 animate-in fade-in duration-100">
                      <span className="text-[10px] uppercase font-bold text-slate-400 font-mono pr-1.5 border-r border-slate-200 dark:border-[#2c3338]">
                        {block.rf_type}
                      </span>

                      {/* Déplacement haut / bas */}
                      <button
                        type="button"
                        disabled={index === 0}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMoveBlock(block.rf_id, 'up');
                        }}
                        title="Monter le bloc"
                        className="p-1 rounded hover:bg-slate-100 dark:hover:bg-[#2c3338] disabled:opacity-30 text-slate-600 dark:text-slate-300"
                      >
                        <ArrowUp className="w-3.5 h-3.5" />
                      </button>

                      <button
                        type="button"
                        disabled={index === document.rf_blocks.length - 1}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMoveBlock(block.rf_id, 'down');
                        }}
                        title="Descendre le bloc"
                        className="p-1 rounded hover:bg-slate-100 dark:hover:bg-[#2c3338] disabled:opacity-30 text-slate-600 dark:text-slate-300"
                      >
                        <ArrowDown className="w-3.5 h-3.5" />
                      </button>

                      <div className="h-3.5 w-px bg-slate-200 dark:bg-[#2c3338] mx-0.5" />

                      {/* Formatage rapide */}
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleUpdateBlock(block.rf_id, {
                            rf_text: `**${block.rf_text}**`,
                          });
                        }}
                        title="Mettre en gras"
                        className="p-1 rounded hover:bg-slate-100 dark:hover:bg-[#2c3338] text-slate-600 dark:text-slate-300"
                      >
                        <Bold className="w-3.5 h-3.5" />
                      </button>

                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleUpdateBlock(block.rf_id, {
                            rf_text: `*${block.rf_text}*`,
                          });
                        }}
                        title="Mettre en italique"
                        className="p-1 rounded hover:bg-slate-100 dark:hover:bg-[#2c3338] text-slate-600 dark:text-slate-300"
                      >
                        <Italic className="w-3.5 h-3.5" />
                      </button>

                      <div className="h-3.5 w-px bg-slate-200 dark:bg-[#2c3338] mx-0.5" />

                      {/* Dupliquer */}
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDuplicateBlock(block);
                        }}
                        title="Dupliquer le bloc"
                        className="p-1 rounded hover:bg-slate-100 dark:hover:bg-[#2c3338] text-slate-600 dark:text-slate-300"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>

                      {/* Supprimer */}
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRemoveBlock(block.rf_id);
                        }}
                        title="Supprimer le bloc"
                        className="p-1 rounded hover:bg-red-50 dark:hover:bg-red-950 text-red-500"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )}

                  {/* Rendu spécifique selon le type de bloc */}
                  {block.rf_type === 'paragraph' && (
                    <textarea
                      rows={Math.max(1, Math.ceil((block.rf_text.length || 1) / 60))}
                      value={block.rf_text}
                      onChange={(e) => handleUpdateBlock(block.rf_id, { rf_text: e.target.value })}
                      placeholder="Saisissez votre paragraphe..."
                      className="w-full bg-transparent text-[#2c3338] dark:text-slate-100 text-base leading-relaxed outline-none resize-none"
                    />
                  )}

                  {block.rf_type === 'heading' && (
                    <input
                      type="text"
                      value={block.rf_text}
                      onChange={(e) => handleUpdateBlock(block.rf_id, { rf_text: e.target.value })}
                      placeholder="Titre de section..."
                      className={`w-full bg-transparent text-[#1d2327] dark:text-white font-bold outline-none leading-snug ${
                        block.rf_level === 1
                          ? 'text-2xl font-serif'
                          : block.rf_level === 2
                          ? 'text-xl'
                          : 'text-lg'
                      }`}
                    />
                  )}

                  {block.rf_type === 'quote' && (
                    <div className="border-l-4 border-[#2271b1] pl-4 py-1 italic text-slate-700 dark:text-slate-300">
                      <textarea
                        rows={2}
                        value={block.rf_text}
                        onChange={(e) => handleUpdateBlock(block.rf_id, { rf_text: e.target.value })}
                        placeholder="Texte de la citation..."
                        className="w-full bg-transparent text-base outline-none resize-none"
                      />
                      <input
                        type="text"
                        value={block.rf_caption || ''}
                        onChange={(e) => handleUpdateBlock(block.rf_id, { rf_caption: e.target.value })}
                        placeholder="Auteur de la citation..."
                        className="w-full bg-transparent text-xs text-slate-400 outline-none mt-1"
                      />
                    </div>
                  )}

                  {block.rf_type === 'code' && (
                    <div className="bg-[#131619] rounded p-3 font-mono text-xs text-emerald-400">
                      <textarea
                        rows={3}
                        value={block.rf_text}
                        onChange={(e) => handleUpdateBlock(block.rf_id, { rf_text: e.target.value })}
                        placeholder="// Insérez votre code ici..."
                        className="w-full bg-transparent outline-none resize-none font-mono"
                      />
                    </div>
                  )}

                  {block.rf_type === 'image' && (
                    <div className="space-y-2">
                      {block.rf_url ? (
                        <div>
                          <img
                            src={block.rf_url}
                            alt={block.rf_alt || ''}
                            className="max-h-96 w-auto rounded object-contain mx-auto"
                          />
                          <input
                            type="text"
                            value={block.rf_caption || ''}
                            onChange={(e) =>
                              handleUpdateBlock(block.rf_id, { rf_caption: e.target.value })
                            }
                            placeholder="Ajouter une légende..."
                            className="w-full text-center text-xs text-slate-400 bg-transparent outline-none mt-1 italic"
                          />
                        </div>
                      ) : (
                        <div className="border-2 border-dashed border-slate-300 dark:border-[#2c3338] p-8 text-center rounded">
                          <ImageIcon className="w-8 h-8 mx-auto text-slate-400 mb-2" />
                          <input
                            type="text"
                            value={block.rf_url || ''}
                            onChange={(e) =>
                              handleUpdateBlock(block.rf_id, { rf_url: e.target.value })
                            }
                            placeholder="Entrez l'URL de l'image ou collez un lien..."
                            className="w-full max-w-sm border border-[#8c8f94] bg-white dark:bg-[#131619] rounded px-2.5 py-1 text-xs text-center"
                          />
                        </div>
                      )}
                    </div>
                  )}

                  {block.rf_type === 'divider' && (
                    <div className="py-2">
                      <hr className="border-[#dcdcde] dark:border-[#2c3338]" />
                    </div>
                  )}

                  {block.rf_type === 'button' && (
                    <div className="py-2">
                      <button
                        type="button"
                        className="px-4 py-2 rounded bg-[#2271b1] text-white text-xs font-semibold shadow-xs"
                      >
                        {block.rf_text || 'Bouton d’action'}
                      </button>
                    </div>
                  )}

                  {/* Fallback générique pour les autres types */}
                  {!['paragraph', 'heading', 'quote', 'code', 'image', 'divider', 'button'].includes(
                    block.rf_type
                  ) && (
                    <div className="p-3 rounded border border-slate-200 dark:border-[#2c3338] bg-slate-50 dark:bg-[#131619] text-xs">
                      <span className="font-semibold text-[#1d2327] dark:text-white uppercase tracking-wider block mb-1">
                        Bloc {block.rf_type}
                      </span>
                      <p className="text-slate-600 dark:text-slate-300">
                        {block.rf_text || block.rf_caption || '(Bloc configuré)'}
                      </p>
                    </div>
                  )}
                </div>
              );
            })
          )}

          {/* Déclencheur inférieur « Tapez / pour choisir un bloc » */}
          <div className="pt-4 flex items-center gap-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
            <button
              type="button"
              onClick={toggleInserter}
              title="Ajouter un bloc"
              className="p-1 rounded border border-slate-300 dark:border-[#2c3338] hover:border-[#2271b1] text-slate-500 hover:text-[#2271b1]"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
            <span
              onClick={handleAddParagraphAtEnd}
              className="text-xs cursor-text select-none"
            >
              Tapez <kbd className="px-1 py-0.5 rounded bg-slate-200 dark:bg-slate-800 text-[10px] font-mono">/</kbd> pour choisir un bloc ou cliquez pour continuer à écrire...
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};
