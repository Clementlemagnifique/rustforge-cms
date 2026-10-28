/**
 * ============================================================================
 * STORE ZUSTAND : ÉDITEUR D'ARTICLE / PAGE & BLOCS NATIFS (`useEditorStore`)
 * ============================================================================
 * Responsabilités :
 * - Cycle de vie complet de l'édition d'un article ou d'une page (`WP-23`).
 * - Gestion du document natif `rf.document/1`, de sa pile immuable `undo/redo`
 *   (`WP-30`) et de la conversion bidirectionnelle explicite Markdown <-> Blocs (`WP-24`).
 * - Autosauvegarde locale versionnée (`rf.autosave.v1:<type>:<id>`) et restauration
 *   contrôlée (`WP-29`).
 * - Métadonnées de publication (`WP-27`, `WP-32`), hiérarchie `parent_id` (`WP-20`),
 *   taxonomies, image mise en avant et conservation intégrale du coffre `_rf_vault`.
 * - Historique et restauration des révisions serveur (`WP-28`).
 */

import { create } from 'zustand';
import { api } from '../api';
import {
  autosaveStorageKey,
  createHistory,
  newDocument,
  parseAutosaveDraft,
  pushHistory,
  readDocument,
  redoHistory,
  undoHistory,
} from '../editor/document';
import type { AutosaveDraft, DocumentHistory, NativeDocument } from '../editor/document';
import { editorMessages } from '../i18n/messages.fr';
import type { Entry, EntryStatus, Media, TermRef } from '../types';

export type EditorTab = 'content' | 'document' | 'intent' | 'design';

export interface RevisionSummary {
  id: string;
  title: string;
  excerpt?: string;
  body?: string;
  fields?: Record<string, any>;
  snapshot?: Record<string, any> | null;
  created_at: string;
  author_name?: string;
}

export interface EditorStoreState {
  // --- État de navigation interne et statut d'opération ---
  activeTab: EditorTab;
  loading: boolean;
  saving: boolean;
  converting: boolean;
  isDirty: boolean;

  // --- Identité et version de concurrence ---
  entryId?: string;
  effectiveType: string;
  expectedVersion?: string;

  // --- Contenu principal & Blocs natifs rf.document/1 ---
  title: string;
  slug: string;
  excerpt: string;
  body: string;
  rawBlocks: unknown;
  docHistory: DocumentHistory | null;
  unsupportedNative: boolean;

  // --- Publication, planification, médias & taxonomies ---
  status: EntryStatus;
  publishedAt: string;
  scheduledUnpublishAt: string;
  featuredMediaId: string | null;
  parentId: string | null;
  selectedTermIds: string[];

  // --- Référentiels chargés pour les sélecteurs ---
  availableTerms: TermRef[];
  newTermName: string;
  newTermTaxonomy: 'category' | 'tag';
  mediaLibrary: Media[];
  candidateParents: Entry[];

  // --- Autosauvegarde, Intention IA, Style & Coffre (_rf_vault) ---
  recoverableDraft: AutosaveDraft | null;
  intent: Record<string, any>;
  designTokens: Record<string, any>;
  fields: Record<string, any>;

  // --- Aperçu serveur & Révisions ---
  preview: string;
  previewError: string;
  loadError: string;
  revisions: RevisionSummary[] | null;
  historyError: string;

  // --- Mutateurs atomiques (marquent automatiquement `isDirty` lorsque pertinent) ---
  setActiveTab: (tab: EditorTab) => void;
  markDirty: () => void;
  setTitle: (title: string) => void;
  setSlug: (slug: string) => void;
  setExcerpt: (excerpt: string) => void;
  setBody: (body: string) => void;
  setStatus: (status: EntryStatus) => void;
  setPublishedAt: (publishedAt: string) => void;
  setScheduledUnpublishAt: (scheduledUnpublishAt: string) => void;
  setFeaturedMediaId: (mediaId: string | null) => void;
  setParentId: (parentId: string | null) => void;
  toggleTermId: (termId: string, checked: boolean) => void;
  setNewTermName: (name: string) => void;
  setNewTermTaxonomy: (taxonomy: 'category' | 'tag') => void;
  setIntent: (intent: Record<string, any>) => void;
  setDesignTokens: (tokens: Record<string, any>) => void;
  closeRevisions: () => void;

  // --- Actions sur l'historique de blocs (Undo / Redo) ---
  undo: () => void;
  redo: () => void;
  pushBlocks: (next: NativeDocument) => void;

  // --- Actions d'autosauvegarde locale ---
  applyRecoveredDraft: () => void;
  discardRecoveredDraft: () => void;
  persistAutosave: () => void;

  // --- Actions asynchrones (API) ---
  initEditor: (entryId: string | undefined, contentTypeSlug: string) => Promise<void>;
  loadCandidateParents: (
    contentTypeSlug: string,
    entryId: string | undefined,
    isHierarchical: boolean
  ) => Promise<void>;
  refreshPreview: (signal?: AbortSignal) => Promise<void>;
  convertToSource: () => Promise<void>;
  convertToBlocks: () => Promise<void>;
  uploadMedia: (file: File) => Promise<Media | null>;
  createTerm: () => Promise<void>;
  fetchRevisions: () => Promise<void>;
  restoreRevision: (revisionId: string, onSaved: (type: string) => void) => Promise<void>;
  saveEntry: (
    isHierarchical: boolean,
    onSaved: (type: string) => void,
    forceStatus?: EntryStatus
  ) => Promise<void>;
}

const DEFAULT_INTENT = {
  prompt: '',
  target_audience: '',
  tone: 'Professionnel et percutant',
  geo_keywords: '',
};

const DEFAULT_DESIGN_TOKENS = {
  accent_color: '#6366f1',
  layout_mode: 'standard',
};

export const useEditorStore = create<EditorStoreState>((set, get) => ({
  activeTab: 'content',
  loading: false,
  saving: false,
  converting: false,
  isDirty: false,

  entryId: undefined,
  effectiveType: 'post',
  expectedVersion: undefined,

  title: '',
  slug: '',
  excerpt: '',
  body: '',
  rawBlocks: [],
  docHistory: createHistory(newDocument()),
  unsupportedNative: false,

  status: 'draft',
  publishedAt: '',
  scheduledUnpublishAt: '',
  featuredMediaId: null,
  parentId: null,
  selectedTermIds: [],

  availableTerms: [],
  newTermName: '',
  newTermTaxonomy: 'category',
  mediaLibrary: [],
  candidateParents: [],

  recoverableDraft: null,
  intent: { ...DEFAULT_INTENT },
  designTokens: { ...DEFAULT_DESIGN_TOKENS },
  fields: {},

  preview: '',
  previewError: '',
  loadError: '',
  revisions: null,
  historyError: '',

  setActiveTab: (activeTab) => set({ activeTab }),
  markDirty: () => set({ isDirty: true }),

  setTitle: (title) => set({ title, isDirty: true }),
  setSlug: (slug) => set({ slug, isDirty: true }),
  setExcerpt: (excerpt) => set({ excerpt, isDirty: true }),
  setBody: (body) => set({ body, isDirty: true }),
  setStatus: (status) => set({ status, isDirty: true }),
  setPublishedAt: (publishedAt) => set({ publishedAt, isDirty: true }),
  setScheduledUnpublishAt: (scheduledUnpublishAt) => set({ scheduledUnpublishAt, isDirty: true }),
  setFeaturedMediaId: (featuredMediaId) => set({ featuredMediaId, isDirty: true }),
  setParentId: (parentId) => set({ parentId, isDirty: true }),

  toggleTermId: (termId, checked) =>
    set((state) => ({
      selectedTermIds: checked
        ? state.selectedTermIds.includes(termId)
          ? state.selectedTermIds
          : [...state.selectedTermIds, termId]
        : state.selectedTermIds.filter((id) => id !== termId),
      isDirty: true,
    })),

  setNewTermName: (newTermName) => set({ newTermName }),
  setNewTermTaxonomy: (newTermTaxonomy) => set({ newTermTaxonomy }),
  setIntent: (intent) => set({ intent, isDirty: true }),
  setDesignTokens: (designTokens) => set({ designTokens, isDirty: true }),
  closeRevisions: () => set({ revisions: null }),

  undo: () =>
    set((state) => ({
      docHistory: state.docHistory ? undoHistory(state.docHistory) : null,
      isDirty: true,
    })),

  redo: () =>
    set((state) => ({
      docHistory: state.docHistory ? redoHistory(state.docHistory) : null,
      isDirty: true,
    })),

  pushBlocks: (next) =>
    set((state) => ({
      docHistory: state.docHistory ? pushHistory(state.docHistory, next) : createHistory(next),
      isDirty: true,
    })),

  applyRecoveredDraft: () => {
    const draft = get().recoverableDraft;
    if (!draft) return;
    const native = readDocument(draft.blocks);
    set((state) => ({
      title: draft.title,
      slug: draft.slug,
      excerpt: draft.excerpt,
      body: draft.body,
      rawBlocks: draft.blocks ?? [],
      docHistory: native ? createHistory(native) : null,
      featuredMediaId:
        draft.featured_media_id !== undefined ? draft.featured_media_id : state.featuredMediaId,
      parentId: draft.parent_id !== undefined ? draft.parent_id : state.parentId,
      publishedAt: draft.published_at
        ? new Date(draft.published_at).toISOString().slice(0, 16)
        : state.publishedAt,
      scheduledUnpublishAt: draft.scheduled_unpublish_at
        ? new Date(draft.scheduled_unpublish_at).toISOString().slice(0, 16)
        : state.scheduledUnpublishAt,
      selectedTermIds: Array.isArray(draft.term_ids) ? draft.term_ids : state.selectedTermIds,
      recoverableDraft: null,
      isDirty: true,
    }));
  },

  discardRecoveredDraft: () => {
    const { effectiveType, entryId } = get();
    localStorage.removeItem(autosaveStorageKey(effectiveType, entryId));
    set({ recoverableDraft: null });
  },

  persistAutosave: () => {
    const state = get();
    if (!state.isDirty || state.loading || state.unsupportedNative) return;
    const storageKey = autosaveStorageKey(state.effectiveType, state.entryId);
    const document = state.docHistory?.present ?? null;
    const draft: AutosaveDraft = {
      rf_autosave_version: 1,
      saved_at: new Date().toISOString(),
      base_updated_at: state.expectedVersion,
      title: state.title,
      slug: state.slug,
      excerpt: state.excerpt,
      body: state.body,
      blocks: document || state.rawBlocks,
      status: state.status,
      featured_media_id: state.featuredMediaId,
      parent_id: state.parentId,
      published_at: state.publishedAt ? new Date(state.publishedAt).toISOString() : null,
      scheduled_unpublish_at: state.scheduledUnpublishAt
        ? new Date(state.scheduledUnpublishAt).toISOString()
        : null,
      term_ids: state.selectedTermIds,
    };
    try {
      localStorage.setItem(storageKey, JSON.stringify(draft));
    } catch {
      // Quota localStorage atteint : ignorer sans bloquer l'édition
    }
  },

  initEditor: async (entryId, contentTypeSlug) => {
    // Réinitialisation complète de l'état pour la nouvelle cible d'édition
    const initialStorageKey = autosaveStorageKey(contentTypeSlug, entryId);
    set({
      activeTab: 'content',
      loading: !!entryId,
      saving: false,
      converting: false,
      isDirty: false,
      entryId,
      effectiveType: contentTypeSlug,
      expectedVersion: undefined,
      title: '',
      slug: '',
      excerpt: '',
      body: '',
      rawBlocks: [],
      docHistory: entryId ? null : createHistory(newDocument()),
      unsupportedNative: false,
      status: 'draft',
      publishedAt: '',
      scheduledUnpublishAt: '',
      featuredMediaId: null,
      parentId: null,
      selectedTermIds: [],
      newTermName: '',
      newTermTaxonomy: 'category',
      recoverableDraft: entryId
        ? null
        : parseAutosaveDraft(localStorage.getItem(initialStorageKey)),
      intent: { ...DEFAULT_INTENT },
      designTokens: { ...DEFAULT_DESIGN_TOKENS },
      fields: {},
      preview: '',
      previewError: '',
      loadError: '',
      revisions: null,
      historyError: '',
    });

    // Chargement parallèle des référentiels (médias & taxonomies)
    void api.media
      .list()
      .then((res) => set({ mediaLibrary: res.items || [] }))
      .catch(() => {});
    void api.terms
      .list()
      .then((terms) => set({ availableTerms: terms || [] }))
      .catch(() => {});

    if (!entryId) return;

    try {
      const e = await api.entries.get(entryId);
      const blocks = e.blocks === undefined ? [] : e.blocks;
      const native = readDocument(blocks);
      const unsupported =
        !!blocks && typeof blocks === 'object' && 'rf_schema' in blocks && !native;
      const key = autosaveStorageKey(e.content_type, entryId);
      const draft = parseAutosaveDraft(localStorage.getItem(key), e.updated_at);

      set({
        loading: false,
        expectedVersion: e.updated_at,
        effectiveType: e.content_type,
        title: e.title,
        slug: e.slug,
        excerpt: e.excerpt || '',
        body: e.body || '',
        rawBlocks: blocks,
        docHistory: native ? createHistory(native) : null,
        unsupportedNative: unsupported,
        status: e.status,
        featuredMediaId: e.featured_media_id || null,
        parentId: e.parent_id || null,
        selectedTermIds: (e.terms || []).map((t) => t.id),
        publishedAt: e.published_at ? new Date(e.published_at).toISOString().slice(0, 16) : '',
        scheduledUnpublishAt: e.scheduled_unpublish_at
          ? new Date(e.scheduled_unpublish_at).toISOString().slice(0, 16)
          : '',
        intent: e.intent || { ...DEFAULT_INTENT },
        designTokens: e.design_tokens || { ...DEFAULT_DESIGN_TOKENS },
        fields: e.fields || {},
        recoverableDraft: draft,
      });
    } catch (err) {
      set({
        loading: false,
        loadError: err instanceof Error ? err.message : 'Chargement impossible',
      });
    }
  },

  loadCandidateParents: async (contentTypeSlug, entryId, isHierarchical) => {
    if (!isHierarchical) {
      set({ candidateParents: [] });
      return;
    }
    try {
      const res = await api.entries.list({ content_type: contentTypeSlug, per_page: 100 });
      set({
        candidateParents: (res.items || []).filter((item) => item.id !== entryId),
      });
    } catch {
      set({ candidateParents: [] });
    }
  },

  refreshPreview: async (signal) => {
    const { body, docHistory, rawBlocks } = get();
    const document = docHistory?.present ?? null;
    try {
      const result = await api.entries.preview(body, signal, document || rawBlocks);
      if (!signal?.aborted) {
        set({ preview: result.body_html, previewError: '' });
      }
    } catch (err) {
      if (!signal?.aborted) {
        set({ previewError: err instanceof Error ? err.message : 'Aperçu indisponible' });
      }
    }
  },

  convertToSource: async () => {
    const { docHistory, body } = get();
    const document = docHistory?.present ?? null;
    if (!document || !window.confirm(editorMessages.sourceConfirm)) return;
    set({ converting: true });
    try {
      const result = await api.entries.preview(body, undefined, document);
      set({
        body: result.body,
        rawBlocks: [],
        docHistory: null,
        converting: false,
        isDirty: true,
      });
    } catch (err) {
      set({
        previewError: err instanceof Error ? err.message : 'Conversion impossible',
        converting: false,
      });
    }
  },

  convertToBlocks: async () => {
    const { docHistory, body } = get();
    if (docHistory?.present || !window.confirm(editorMessages.toBlocksConfirm)) return;
    set({ converting: true });
    try {
      const result = await api.entries.convertBlocks(body);
      const native = readDocument(result.blocks);
      if (!native) throw new Error('Structure native retournée invalide');
      set({
        docHistory: createHistory(native),
        body: result.body,
        preview: result.body_html,
        previewError: '',
        converting: false,
        isDirty: true,
      });
    } catch (err) {
      set({
        previewError: err instanceof Error ? err.message : 'Conversion en blocs impossible',
        converting: false,
      });
    }
  },

  uploadMedia: async (file) => {
    try {
      const uploaded = await api.media.upload(file);
      set((state) => ({ mediaLibrary: [uploaded, ...state.mediaLibrary] }));
      return uploaded;
    } catch (err) {
      window.alert(
        `Téléversement impossible : ${err instanceof Error ? err.message : 'Erreur'}`
      );
      return null;
    }
  },

  createTerm: async () => {
    const { newTermName, newTermTaxonomy } = get();
    const trimmed = newTermName.trim();
    if (!trimmed) return;
    try {
      const created = await api.terms.create({ taxonomy: newTermTaxonomy, name: trimmed });
      set((state) => ({
        availableTerms: [...state.availableTerms, created],
        selectedTermIds: state.selectedTermIds.includes(created.id)
          ? state.selectedTermIds
          : [...state.selectedTermIds, created.id],
        newTermName: '',
        isDirty: true,
      }));
    } catch (err) {
      window.alert(
        `Création du terme impossible : ${err instanceof Error ? err.message : 'Erreur'}`
      );
    }
  },

  fetchRevisions: async () => {
    const { entryId } = get();
    if (!entryId) return;
    set({ historyError: '' });
    try {
      const list = await api.entries.revisions(entryId);
      set({ revisions: list });
    } catch (err) {
      set({
        historyError: err instanceof Error ? err.message : 'Historique indisponible',
      });
    }
  },

  restoreRevision: async (revisionId, onSaved) => {
    const { entryId, effectiveType, expectedVersion } = get();
    if (!entryId || !window.confirm(editorMessages.historyRestoreConfirm)) return;
    set({ saving: true });
    try {
      await api.entries.restoreRevision(entryId, revisionId, expectedVersion);
      localStorage.removeItem(autosaveStorageKey(effectiveType, entryId));
      set({ saving: false, isDirty: false });
      onSaved(effectiveType);
    } catch (err) {
      set({
        saving: false,
        historyError: err instanceof Error ? err.message : 'Restauration impossible',
      });
    }
  },

  saveEntry: async (isHierarchical, onSaved, forceStatus) => {
    const state = get();
    if (!state.title.trim()) {
      window.alert('Veuillez renseigner un titre.');
      return;
    }

    set({ saving: true });
    const saveStatus = forceStatus || state.status;
    const document = state.docHistory?.present ?? null;
    const storageKey = autosaveStorageKey(state.effectiveType, state.entryId);

    const payload: Record<string, any> = {
      title: state.title,
      slug: state.slug || undefined,
      excerpt: state.excerpt,
      body: state.body,
      blocks: document || state.rawBlocks,
      status: saveStatus,
      featured_media_id: state.featuredMediaId,
      parent_id: isHierarchical ? state.parentId : null,
      published_at: state.publishedAt ? new Date(state.publishedAt).toISOString() : null,
      scheduled_unpublish_at: state.scheduledUnpublishAt
        ? new Date(state.scheduledUnpublishAt).toISOString()
        : null,
      term_ids: state.selectedTermIds,
      intent: state.intent,
      design_tokens: state.designTokens,
      fields: state.fields, // Conservation intégrale des champs personnalisés et de _rf_vault / _legacy_vault
    };

    try {
      if (!state.entryId) {
        await api.entries.create(state.effectiveType, payload);
      } else {
        await api.entries.update(state.entryId, {
          ...payload,
          expected_updated_at: state.expectedVersion,
        });
      }
      localStorage.removeItem(storageKey);
      set({ saving: false, isDirty: false });
      onSaved(state.effectiveType);
    } catch (err) {
      set({ saving: false });
      window.alert(`Erreur: ${err instanceof Error ? err.message : 'Sauvegarde impossible'}`);
    }
  },
}));

