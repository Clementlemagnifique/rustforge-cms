/**
 * ============================================================================
 * PANNEAU INSÉREUR DE BLOCS & COMPOSITIONS (`GutenbergInserterDrawer`)
 * ============================================================================
 * Reproduit à 100% le tiroir d'insertion officiel de Gutenberg :
 * - Onglet « Blocs » : Catalogue catégorisé (Texte, Médias, Conception, Modules).
 * - Onglet « Compositions » : Motifs officiels du thème Twenty Twenty-Five
 *   (En-têtes, Bannières, Grilles d'articles, Appels à l'action, Témoignages).
 * - Onglet « Médias » : Accès direct aux fichiers de la médiathèque.
 * - Champ de recherche dynamique avec filtrage en temps réel.
 */

import React, { useState } from 'react';
import {
  Search,
  X,
  Type,
  Heading,
  List,
  Quote,
  Code2,
  Table2,
  Image as ImageIcon,
  Film,
  Music,
  FileText,
  Columns2,
  Minus,
  MousePointerClick,
  Layers,
  GraduationCap,
  ShoppingCart,
} from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';
import { BLOCK_PATTERNS } from '../../editor/patterns';
import { newBlock, insertBlockAfter, insertCustomBlockAfter } from '../../editor/document';
import type { BlockKind, NativeDocument } from '../../editor/document';

interface GutenbergInserterDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export const GutenbergInserterDrawer: React.FC<GutenbergInserterDrawerProps> = ({
  isOpen,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<'blocks' | 'patterns' | 'media'>('blocks');
  const [searchQuery, setSearchQuery] = useState('');
  const [patternCategory, setPatternCategory] = useState<string>('all');

  const docHistory = useEditorStore((s) => s.docHistory);
  const pushBlocks = useEditorStore((s) => s.pushBlocks);
  const mediaLibrary = useEditorStore((s) => s.mediaLibrary);
  const setSelectedBlockId = useEditorStore((s) => s.setSelectedBlockId);

  if (!isOpen) return null;

  const document: NativeDocument = docHistory?.present ?? {
    rf_schema: 'rf.document/1',
    rf_blocks: [],
  };

  const handleInsertBlock = (kind: BlockKind) => {
    const updatedDoc = insertBlockAfter(document, undefined, kind);
    pushBlocks(updatedDoc);
    const last = updatedDoc.rf_blocks[updatedDoc.rf_blocks.length - 1];
    if (last) setSelectedBlockId(last.rf_id);
    onClose();
  };

  const handleInsertPattern = (patternId: string) => {
    const pattern = BLOCK_PATTERNS.find((p) => p.id === patternId);
    if (!pattern) return;

    const newBlocks = pattern.blocks.map((b) => ({
      ...b,
      rf_id: crypto.randomUUID(),
    }));

    const updatedDoc: NativeDocument = {
      ...document,
      rf_blocks: [...document.rf_blocks, ...newBlocks],
    };
    pushBlocks(updatedDoc);
    if (newBlocks.length > 0) {
      setSelectedBlockId(newBlocks[0].rf_id);
    }
    onClose();
  };

  const handleInsertMedia = (url: string, alt: string) => {
    const block = newBlock('image');
    block.rf_url = url;
    block.rf_alt = alt;
    const updatedDoc = insertCustomBlockAfter(document, undefined, block);
    pushBlocks(updatedDoc);
    setSelectedBlockId(block.rf_id);
    onClose();
  };

  // Définition des blocs disponibles
  const BLOCKS_CATALOG = [
    {
      category: 'Texte',
      items: [
        { kind: 'paragraph' as BlockKind, label: 'Paragraphe', desc: 'Commencez à écrire du texte brut.', icon: Type },
        { kind: 'heading' as BlockKind, label: 'Titre', desc: 'Introduisez de nouvelles sections (H1-H6).', icon: Heading },
        { kind: 'list' as BlockKind, label: 'Liste', desc: 'Créez une liste à puces ou numérotée.', icon: List },
        { kind: 'quote' as BlockKind, label: 'Citation', desc: 'Donnez de la visibilité à une citation.', icon: Quote },
        { kind: 'code' as BlockKind, label: 'Code', desc: 'Affichez des extraits de code préformaté.', icon: Code2 },
        { kind: 'table' as BlockKind, label: 'Tableau', desc: 'Insérez des tableaux de données structurées.', icon: Table2 },
      ],
    },
    {
      category: 'Médias',
      items: [
        { kind: 'image' as BlockKind, label: 'Image', desc: 'Insérez une image pour illustrer votre propos.', icon: ImageIcon },
        { kind: 'gallery' as BlockKind, label: 'Galerie', desc: 'Affichez une collection d’images.', icon: Layers },
        { kind: 'video' as BlockKind, label: 'Vidéo', desc: 'Intégrez un fichier vidéo HTML5 natif.', icon: Film },
        { kind: 'audio' as BlockKind, label: 'Audio', desc: 'Insérez un lecteur audio ou podcast.', icon: Music },
        { kind: 'file' as BlockKind, label: 'Fichier', desc: 'Ajoutez un lien de téléchargement direct.', icon: FileText },
        { kind: 'media_text' as BlockKind, label: 'Média & Texte', desc: 'Placez une image et du texte côte à côte.', icon: Columns2 },
      ],
    },
    {
      category: 'Conception',
      items: [
        { kind: 'button' as BlockKind, label: 'Bouton', desc: 'Incitez à l’action avec un bouton stylisé.', icon: MousePointerClick },
        { kind: 'columns' as BlockKind, label: 'Colonnes', desc: 'Organisez vos contenus en plusieurs colonnes.', icon: Columns2 },
        { kind: 'group' as BlockKind, label: 'Groupe', desc: 'Rassemblez plusieurs blocs dans un conteneur.', icon: Layers },
        { kind: 'divider' as BlockKind, label: 'Séparateur', desc: 'Séparez visuellement vos sections.', icon: Minus },
        { kind: 'spacer' as BlockKind, label: 'Espacement', desc: 'Ajoutez un espace blanc réglable.', icon: Minus },
      ],
    },
    {
      category: 'Modules souverains',
      items: [
        { kind: 'course' as BlockKind, label: 'Formation / LMS', desc: 'Intégrez un module Sensei LMS natif.', icon: GraduationCap },
        { kind: 'product' as BlockKind, label: 'Fiche Produit', desc: 'Insérez un produit e-commerce avec prix.', icon: ShoppingCart },
      ],
    },
  ];

  // Compositions Twenty Twenty-Five enrichies
  const TWENTY_TWENTY_FIVE_PATTERNS = [
    ...BLOCK_PATTERNS,
    {
      id: 'tt5_editorial_header',
      title: 'En-tête éditorial Twenty Twenty-Five',
      description: 'Grand titre serif Playfair Display avec surtitre et séparateur fin.',
      category: 'hero' as const,
      blocks: [
        { rf_type: 'heading' as BlockKind, rf_text: 'L’Art de la Typographie & de la Pensée', rf_level: 1, rf_ordered: false, rf_items: [] },
        { rf_type: 'paragraph' as BlockKind, rf_text: 'Une réflexion posée sur l’équilibre, le temps et la matérialité des mots.', rf_level: 2, rf_ordered: false, rf_items: [] },
        { rf_type: 'divider' as BlockKind, rf_text: '', rf_level: 2, rf_ordered: false, rf_items: [] },
      ],
    },
    {
      id: 'tt5_book_intro',
      title: 'Présentation de Livre / Récit',
      description: 'Composition avec couverture illustrée, citation et paragraphe narratif.',
      category: 'content' as const,
      blocks: [
        { rf_type: 'quote' as BlockKind, rf_text: '« Les livres sont les navires qui traversent les mers immenses du temps. »', rf_caption: 'Francis Bacon', rf_level: 2, rf_ordered: false, rf_items: [] },
        { rf_type: 'paragraph' as BlockKind, rf_text: 'Chaque chapitre s’ouvre sur un paysage singulier, explorant la mémoire et les traces laissées par les générations passées.', rf_level: 2, rf_ordered: false, rf_items: [] },
      ],
    },
  ];

  const filteredBlocks = BLOCKS_CATALOG.map((cat) => ({
    ...cat,
    items: cat.items.filter(
      (item) =>
        item.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.desc.toLowerCase().includes(searchQuery.toLowerCase())
    ),
  })).filter((cat) => cat.items.length > 0);

  const filteredPatterns = TWENTY_TWENTY_FIVE_PATTERNS.filter((p) => {
    const matchCat = patternCategory === 'all' || p.category === patternCategory;
    const matchQuery =
      p.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchCat && matchQuery;
  });

  return (
    <div
      role="region"
      aria-label="Bibliothèque de blocs et compositions"
      className="fixed top-14 bottom-0 left-0 w-80 md:w-96 bg-white dark:bg-[#1d2327] border-r border-[#dcdcde] dark:border-[#2c3338] shadow-2xl z-40 flex flex-col font-sans animate-in slide-in-from-left duration-150"
    >
      {/* En-tête : Recherche & Fermeture */}
      <div className="p-3 border-b border-[#dcdcde] dark:border-[#2c3338] space-y-2">
        <div className="flex items-center justify-between">
          <span className="font-semibold text-xs text-[#1d2327] dark:text-white uppercase tracking-wider">
            Ajouter un élément
          </span>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded text-slate-400 hover:text-slate-700 dark:hover:text-white"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Champ de recherche */}
        <div className="relative">
          <Search className="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-2.5" />
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Rechercher des blocs ou motifs..."
            className="w-full bg-[#f0f0f1] dark:bg-[#131619] border border-transparent focus:border-[#2271b1] rounded pl-8 pr-3 py-1.5 text-xs text-[#1d2327] dark:text-white outline-none"
          />
        </div>

        {/* Onglets : Blocs | Compositions | Médias */}
        <div className="flex border-b border-[#dcdcde] dark:border-[#2c3338] -mx-3 px-3 pt-1">
          <button
            type="button"
            onClick={() => setActiveTab('blocks')}
            className={`flex-1 py-1.5 text-xs font-semibold border-b-2 text-center transition-colors ${
              activeTab === 'blocks'
                ? 'border-[#2271b1] text-[#2271b1]'
                : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-white'
            }`}
          >
            Blocs
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('patterns')}
            className={`flex-1 py-1.5 text-xs font-semibold border-b-2 text-center transition-colors ${
              activeTab === 'patterns'
                ? 'border-[#2271b1] text-[#2271b1]'
                : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-white'
            }`}
          >
            Compositions
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('media')}
            className={`flex-1 py-1.5 text-xs font-semibold border-b-2 text-center transition-colors ${
              activeTab === 'media'
                ? 'border-[#2271b1] text-[#2271b1]'
                : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-white'
            }`}
          >
            Médias
          </button>
        </div>
      </div>

      {/* Contenu de l'onglet actif */}
      <div className="flex-1 overflow-y-auto p-4 space-y-5">
        {/* ================================================================== */}
        {/* ONGLET 1 : BLOCS INDIVIDUELS                                       */}
        {/* ================================================================== */}
        {activeTab === 'blocks' && (
          <div className="space-y-4">
            {filteredBlocks.map((cat) => (
              <div key={cat.category} className="space-y-2">
                <h5 className="text-[11px] font-bold text-slate-400 dark:text-slate-400 uppercase tracking-wider">
                  {cat.category}
                </h5>
                <div className="grid grid-cols-3 gap-2">
                  {cat.items.map((item) => {
                    const IconComp = item.icon;
                    return (
                      <button
                        key={item.kind}
                        type="button"
                        onClick={() => handleInsertBlock(item.kind)}
                        title={item.desc}
                        className="flex flex-col items-center justify-center p-2.5 rounded bg-[#f6f7f7] dark:bg-[#131619] border border-transparent hover:border-[#2271b1] hover:bg-white dark:hover:bg-[#2c3338] text-center transition-all group"
                      >
                        <IconComp className="w-5 h-5 text-slate-700 dark:text-slate-200 group-hover:text-[#2271b1] mb-1.5" />
                        <span className="text-[11px] font-medium text-slate-800 dark:text-slate-200 leading-tight">
                          {item.label}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* ================================================================== */}
        {/* ONGLET 2 : COMPOSITIONS TWENTY TWENTY-FIVE                         */}
        {/* ================================================================== */}
        {activeTab === 'patterns' && (
          <div className="space-y-3">
            {/* Filtre de catégories de compositions */}
            <div className="flex flex-wrap gap-1">
              {[
                { id: 'all', label: 'Toutes' },
                { id: 'hero', label: 'En-têtes' },
                { id: 'content', label: 'Articles' },
                { id: 'marketing', label: 'Bannières' },
                { id: 'layout', label: 'Colonnes' },
              ].map((c) => (
                <button
                  key={c.id}
                  type="button"
                  onClick={() => setPatternCategory(c.id)}
                  className={`px-2.5 py-1 rounded text-[11px] font-medium transition-colors ${
                    patternCategory === c.id
                      ? 'bg-[#2271b1] text-white'
                      : 'bg-[#f0f0f1] dark:bg-[#131619] text-slate-700 dark:text-slate-300 hover:bg-slate-200'
                  }`}
                >
                  {c.label}
                </button>
              ))}
            </div>

            {/* Cartes de compositions */}
            <div className="space-y-3 pt-1">
              {filteredPatterns.map((pat) => (
                <div
                  key={pat.id}
                  onClick={() => handleInsertPattern(pat.id)}
                  className="p-3 rounded border border-[#dcdcde] dark:border-[#2c3338] hover:border-[#2271b1] bg-[#f6f7f7] dark:bg-[#131619] cursor-pointer transition-all space-y-1.5 group"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-xs text-[#1d2327] dark:text-white group-hover:text-[#2271b1]">
                      {pat.title}
                    </span>
                    <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-slate-200 dark:bg-slate-800 text-slate-600 dark:text-slate-300">
                      TT5
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-500 leading-snug">
                    {pat.description}
                  </p>
                  <div className="pt-1 text-[10px] text-[#2271b1] font-medium">
                    + Insérer la composition
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ================================================================== */}
        {/* ONGLET 3 : MÉDIATHÈQUE                                             */}
        {/* ================================================================== */}
        {activeTab === 'media' && (
          <div className="space-y-3">
            <span className="text-[11px] font-bold text-slate-400 uppercase tracking-wider block">
              Fichiers récents
            </span>
            {mediaLibrary.length === 0 ? (
              <p className="text-xs text-slate-400 py-6 text-center">
                Aucun média disponible dans la médiathèque.
              </p>
            ) : (
              <div className="grid grid-cols-2 gap-2">
                {mediaLibrary.map((m) => (
                  <button
                    key={m.id}
                    type="button"
                    onClick={() => handleInsertMedia(m.url, m.alt || m.original_name || 'Média')}
                    className="p-1 rounded border border-slate-200 dark:border-[#2c3338] hover:border-[#2271b1] bg-white dark:bg-[#131619] text-left transition-colors"
                  >
                    <img
                      src={m.url}
                      alt={m.alt || ''}
                      className="w-full h-20 object-cover rounded mb-1"
                    />
                    <span className="text-[10px] truncate block text-slate-700 dark:text-slate-300">
                      {m.original_name}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
