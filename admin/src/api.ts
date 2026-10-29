import type { User, Entry, ContentType, Media, PluginInfo, DashboardStats, ToolDefinition, AiStatus, AiChatTurn, AiChatResponse } from './types';

const API_BASE = '/api/v1';
export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) { super(message); this.status = status; }
}

function getToken(): string | null {
  return localStorage.getItem('rustforge_token') || localStorage.getItem('trustpress_token');
}

export function setToken(token: string | null) {
  if (token) {
    localStorage.setItem('rustforge_token', token);
    localStorage.removeItem('trustpress_token');
  } else {
    localStorage.removeItem('rustforge_token');
    localStorage.removeItem('trustpress_token');
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers || {});
  const token = getToken();
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }
  if (!(options.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    const errorJson = await res.json().catch(() => ({}));
    throw new ApiError(errorJson.error?.message || `Erreur HTTP ${res.status}`, res.status);
  }

  return res.json();
}

export const api = {
  jobs: {
    list: (state: string, page: number) => request<import('./types').JobsPage>(`/jobs?page=${page}&per_page=20${state ? `&state=${encodeURIComponent(state)}` : ''}`),
    action: (job: import('./types').JobSummary, action: 'cancel' | 'retry' | 'resume-restored') => request<{ok: boolean}>(`/jobs/${job.id}/${action}`, {method: 'POST', body: JSON.stringify({expected_updated_at: job.updated_at})}),
    capture: (id: string) => request<import('./types').JobCapture>(`/jobs/${id}/capture`),
    captureBatch: () => request<{handled: number; mail_mode: string}>('/jobs/capture', {method: 'POST', body: JSON.stringify({limit: 10})}),
  },
  admin: {
    navigation: () => request<import('./types').AdminNavigation>('/admin/navigation'),
    updateModules: (patch: Record<string, boolean>) => request<import('./types').AdminNavigation>('/admin/modules', { method: 'PUT', body: JSON.stringify(patch) }),
  },
  migration: {
    inspect: (file: File, source: string) => {
      const form = new FormData(); form.append('file', file); if (source.trim()) form.append('source_url', source);
      return request<import('./types').MigrationInspection>('/converter/inspect-wxr', { method: 'POST', body: form });
    },
    import: (file: File, source: string, seal: string, contentOnly: boolean) => {
      const form = new FormData(); form.append('file', file); if (source.trim()) form.append('source_url', source);
      form.append('seal', seal); form.append('acknowledge', 'true'); form.append('content_only', String(contentOnly));
      return request<import('./types').MigrationImportResult>('/converter/import-wxr', { method: 'POST', body: form });
    },
  },
  auth: {
    login: (login: string, password: string) =>
      request<{ token: string; user: User }>('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ login, password }),
      }),
    me: () => request<User>('/auth/me'),
  },

  stats: {
    dashboard: () => request<DashboardStats>('/stats/dashboard'),
  },

  entries: {
    preview: (body: string, signal?: AbortSignal, blocks?: unknown) => request<{ body: string; body_html: string }>('/entries/preview', { method: 'POST', body: JSON.stringify({ body, blocks }), signal }),
    convertBlocks: (body: string) => request<{ blocks: unknown; body: string; body_html: string }>('/entries/convert-blocks', { method: 'POST', body: JSON.stringify({ body }) }),
    revisions: (id: string) =>
      request<
        {
          id: string;
          title: string;
          excerpt?: string;
          body?: string;
          fields?: Record<string, any>;
          snapshot?: Record<string, any> | null;
          created_at: string;
          author_name?: string;
        }[]
      >(`/entries/${id}/revisions`),
    restoreRevision: (id: string, revision: string, expectedUpdatedAt?: string) =>
      request<Entry>(`/entries/${id}/revisions/${revision}/restore`, {
        method: 'POST',
        body: JSON.stringify(
          expectedUpdatedAt ? { expected_updated_at: expectedUpdatedAt } : {}
        ),
      }),
    list: (params?: { content_type?: string; q?: string; status?: string; term?: string; taxonomy?: string; order?: string; page?: number; per_page?: number }) => {
      const q = new URLSearchParams();
      if (params?.content_type) q.set('content_type', params.content_type);
      if (params?.q) q.set('q', params.q);
      if (params?.status) q.set('status', params.status);
      if (params?.term) q.set('term', params.term);
      if (params?.taxonomy) q.set('taxonomy', params.taxonomy);
      if (params?.order) q.set('order', params.order);
      if (params?.page) q.set('page', params.page.toString());
      if (params?.per_page) q.set('per_page', params.per_page.toString());
      return request<{ items: Entry[]; total: number; page: number; total_pages: number }>(`/entries?${q}`);
    },
    get: (id: string) => request<Entry>(`/entries/${id}`),
    create: (contentType: string, data: Partial<Entry> & { term_ids?: string[] }) =>
      request<Entry>(`/entries?content_type=${contentType}`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    update: (id: string, data: Partial<Entry> & { expected_updated_at?: string; term_ids?: string[] }) =>
      request<Entry>(`/entries/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    delete: (id: string, force = false) =>
      request<{ success: boolean }>(`/entries/${id}?force=${force}`, {
        method: 'DELETE',
      }),
  },

  contentTypes: {
    list: () => request<ContentType[]>('/content-types'),
    create: (data: Partial<ContentType>) =>
      request<ContentType>('/content-types', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    update: (id: string, data: Partial<ContentType>) =>
      request<ContentType>(`/content-types/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    delete: (id: string) =>
      request<{ success: boolean }>(`/content-types/${id}`, {
        method: 'DELETE',
      }),
  },

  terms: {
    list: (taxonomy?: string) => request<import('./types').TermRef[]>(`/terms${taxonomy ? `?taxonomy=${encodeURIComponent(taxonomy)}` : ''}`),
    create: (data: { taxonomy: string; name: string; slug?: string; description?: string; parent_id?: string | null }) =>
      request<import('./types').TermRef>('/terms', { method: 'POST', body: JSON.stringify(data) }),
    update: (id: string, data: { name: string; slug?: string; description?: string; parent_id?: string | null }) =>
      request<import('./types').TermRef>(`/terms/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    delete: (id: string, reassignTo?: string) =>
      request<{ success: boolean }>(
        `/terms/${id}${reassignTo ? `?reassign_to=${encodeURIComponent(reassignTo)}` : ''}`,
        { method: 'DELETE' }
      ),
    merge: (id: string, targetId: string) =>
      request<import('./types').TermRef>(`/terms/${id}/merge`, {
        method: 'POST',
        body: JSON.stringify({ target_id: targetId }),
      }),
  },

  media: {
    list: (params?: { q?: string; kind?: string; folder_id?: string; page?: number; per_page?: number }) => {
      const q = new URLSearchParams();
      if (params?.q) q.set('q', params.q);
      if (params?.kind) q.set('kind', params.kind);
      if (params?.folder_id) q.set('folder_id', params.folder_id);
      if (params?.page) q.set('page', params.page.toString());
      if (params?.per_page) q.set('per_page', params.per_page.toString());
      const qs = q.toString();
      return request<{ items: Media[]; total: number; page: number; total_pages: number }>(
        `/media${qs ? `?${qs}` : ''}`
      );
    },
    upload: (file: File, folderId?: string | null) => {
      const form = new FormData();
      form.append('file', file);
      const qs = folderId ? `?folder_id=${encodeURIComponent(folderId)}` : '';
      return request<Media>(`/media${qs}`, {
        method: 'POST',
        body: form,
      });
    },
    update: (
      id: string,
      data: { alt?: string; caption?: string; folder_id?: string | null; clear_folder?: boolean }
    ) =>
      request<Media>(`/media/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    delete: (id: string, force = false) =>
      request<{ success: boolean }>(`/media/${id}?force=${force}`, {
        method: 'DELETE',
      }),
    listFolders: () => request<import('./types').MediaFolder[]>('/media/folders'),
    createFolder: (name: string, parentId?: string | null) =>
      request<import('./types').MediaFolder>('/media/folders', {
        method: 'POST',
        body: JSON.stringify({ name, parent_id: parentId ?? null }),
      }),
    deleteFolder: (id: string) =>
      request<{ success: boolean }>(`/media/folders/${id}`, {
        method: 'DELETE',
      }),
    rotate: (id: string, degrees: number) =>
      request<Media>(`/media/${id}/rotate`, {
        method: 'POST',
        body: JSON.stringify({ degrees }),
      }),
    restore: (id: string) =>
      request<Media>(`/media/${id}/restore`, {
        method: 'POST',
      }),
  },

  plugins: {
    list: () => request<PluginInfo[]>('/plugins'),
    rescan: () => request<PluginInfo[]>('/plugins/rescan', { method: 'POST' }),
    enable: (id: string) => request<PluginInfo>(`/plugins/${id}/enable`, { method: 'POST' }),
    disable: (id: string) => request<PluginInfo>(`/plugins/${id}/disable`, { method: 'POST' }),
  },

  settings: {
    getAll: () => request<Record<string, any>>('/settings'),
    update: (data: Record<string, any>) =>
      request<Record<string, any>>('/settings', {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
  },

  users: {
    list: (params?: { search?: string; role?: import('./types').Role; page?: number; per_page?: number }) => {
      const q = new URLSearchParams();
      if (params?.search) q.set('search', params.search);
      if (params?.role) q.set('role', params.role);
      if (params?.page) q.set('page', params.page.toString());
      if (params?.per_page) q.set('per_page', params.per_page.toString());
      const qs = q.toString();
      return request<{ items: User[]; total: number; page: number; total_pages: number }>(
        `/users${qs ? `?${qs}` : ''}`
      );
    },
    create: (data: {
      email: string;
      username: string;
      display_name?: string;
      password: string;
      role?: import('./types').Role;
    }) =>
      request<User>('/users', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    update: (
      id: string,
      data: {
        email?: string;
        display_name?: string;
        bio?: string;
        role?: import('./types').Role;
        password?: string;
      }
    ) =>
      request<User>(`/users/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    delete: (id: string, reassignTo?: string) =>
      request<{ success: boolean }>(
        `/users/${id}${reassignTo ? `?reassign_to=${encodeURIComponent(reassignTo)}` : ''}`,
        { method: 'DELETE' }
      ),
    listSessions: (id: string) =>
      request<import('./types').UserSession[]>(`/users/${id}/sessions`),
    revokeSessions: (id: string) =>
      request<{ success: boolean; revoked: number }>(`/users/${id}/sessions`, {
        method: 'DELETE',
      }),
  },

  ai: {
    plans: () => request<{ id: string; tool: string; arguments: Record<string, any>; status: string }[]>('/ai/plans'),
    approve: (id: string) => request<{ success: boolean; message: string; data?: any }>(`/ai/plans/${id}/approve`, { method: 'POST' }),
    tools: () => request<ToolDefinition[]>('/ai/tools'),
    execute: (tool_name: string, args: Record<string, any>) =>
      request<{ success: boolean; message: string; data?: any }>('/ai/execute', {
        method: 'POST',
        body: JSON.stringify({ tool_name, arguments: args }),
      }),
    status: () => request<AiStatus>('/ai/status'),
    chat: (messages: AiChatTurn[]) =>
      request<AiChatResponse>('/ai/chat', {
        method: 'POST',
        body: JSON.stringify({ messages }),
      }),
  },

  converter: {
    analyze: (file: File) => {
      const form = new FormData();
      form.append('file', file);
      return request<any>('/converter/analyze', {
        method: 'POST',
        body: form,
      });
    },
    convertTheme: (file: File, slug?: string) => {
      const form = new FormData();
      form.append('file', file);
      if (slug) form.append('slug', slug);
      return request<{
        success: boolean;
        theme_slug: string;
        theme_name: string;
        templates_count: number;
        target_directory: string;
        preview_url: string;
      }>('/converter/convert-theme', {
        method: 'POST',
        body: form,
      });
    },
    formsPlan: (slug: string, author_or_title?: string) =>
      request<any>('/converter/forms-plan', {
        method: 'POST',
        body: JSON.stringify({ slug, author_or_title }),
      }),
    applyPlan: (enable_modules: string[], activate_theme_slug?: string) =>
      request<{
        success: boolean;
        activated_modules: string[];
        active_theme: string;
        message: string;
      }>('/converter/apply-plan', {
        method: 'POST',
        body: JSON.stringify({ enable_modules, activate_theme_slug }),
      }),
    importWxr: (file: File) => {
      const form = new FormData();
      form.append('file', file);
      return request<{
        entries_imported: number;
        entries_skipped: number;
        terms_imported: number;
        media_detected: number;
        created_content_types: string[];
        legacy_fields_stored_in_vault: number;
        warnings: string[];
        execution_time_ms: number;
      }>('/converter/import-wxr', {
        method: 'POST',
        body: form,
      });
    },
  },

  setup: {
    status: () =>
      request<{
        is_installed: boolean;
        version: string;
        database_connected: boolean;
        site_title?: string;
      }>('/setup/status'),
    testDb: (database_url: string) =>
      request<{ success: boolean; message: string }>('/setup/test-db', {
        method: 'POST',
        body: JSON.stringify({ database_url }),
      }),
    install: (data: {
      setup_token: string;
      site_title: string;
      site_url?: string;
      admin_email: string;
      admin_username: string;
      admin_password: string;
      site_tagline?: string;
    }) =>
      request<{
        success: boolean;
        message: string;
        token: string;
        user: User;
      }>('/setup/install', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    restoreCapsule: (file: File, new_site_url?: string) => {
      const form = new FormData();
      form.append('file', file);
      if (new_site_url) form.append('new_site_url', new_site_url);
      return request<{
        success: boolean;
        message: string;
        manifest: any;
        entries_restored: number;
        terms_restored: number;
        media_extracted: number;
      }>('/setup/restore-capsule', {
        method: 'POST',
        body: form,
      });
    },
  },

  capsules: {
    downloadExportUrl: () => `${API_BASE}/capsules/export`,
    inspect: (file: File) => {
      const form = new FormData();
      form.append('file', file);
      return request<{ valid: boolean; manifest: any }>('/capsules/inspect', {
        method: 'POST',
        body: form,
      });
    },
    restore: (file: File, new_site_url?: string) => {
      const form = new FormData();
      form.append('file', file);
      if (new_site_url) form.append('new_site_url', new_site_url);
      return request<{
        success: boolean;
        message: string;
        manifest: any;
        entries_restored: number;
        terms_restored: number;
        media_extracted: number;
        vault_fields_stored: number;
        execution_time_ms: number;
      }>('/capsules/restore', {
        method: 'POST',
        body: form,
      });
    },
    exportBlob: async () => {
      const token = getToken();
      const res = await fetch(`${API_BASE}/capsules/export`, {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      });
      if (!res.ok) throw new Error("Erreur lors de l'exportation de la capsule");
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `rustforge-capsule-${new Date().toISOString().slice(0, 10)}.rfcapsule`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
    },
  },

  comments: {
    list: (params?: {
      entry_id?: string;
      status?: import('./types').CommentStatus;
      search?: string;
      page?: number;
      per_page?: number;
    }) => {
      const q = new URLSearchParams();
      if (params?.entry_id) q.set('entry_id', params.entry_id);
      if (params?.status) q.set('status', params.status);
      if (params?.search) q.set('search', params.search);
      if (params?.page) q.set('page', params.page.toString());
      if (params?.per_page) q.set('per_page', params.per_page.toString());
      const qs = q.toString();
      return request<import('./types').CommentListResponse>(`/comments${qs ? `?${qs}` : ''}`);
    },
    counts: () => request<import('./types').CommentCounts>('/comments/counts'),
    get: (id: string) => request<import('./types').CommentItem>(`/comments/${id}`),
    updateStatus: (id: string, status: import('./types').CommentStatus) =>
      request<import('./types').CommentItem>(`/comments/${id}/status`, {
        method: 'PUT',
        body: JSON.stringify({ status }),
      }),
    batchUpdate: (ids: string[], status: import('./types').CommentStatus) =>
      request<{ updated: number }>('/comments/batch', {
        method: 'POST',
        body: JSON.stringify({ ids, status }),
      }),
    update: (
      id: string,
      data: {
        content: string;
        author_name?: string;
        author_email?: string;
        author_url?: string;
      }
    ) =>
      request<import('./types').CommentItem>(`/comments/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    reply: (id: string, content: string) =>
      request<import('./types').CommentItem>(`/comments/${id}/reply`, {
        method: 'POST',
        body: JSON.stringify({ content }),
      }),
    delete: (id: string) =>
      request<{ deleted: boolean }>(`/comments/${id}`, {
        method: 'DELETE',
      }),
    emptyTrash: () =>
      request<{ deleted: number }>('/comments/empty-trash', {
        method: 'DELETE',
      }),
  },

  patterns: {
    list: (category?: string) =>
      request<import('./editor/patterns').SyncedPattern[]>(
        category ? `/patterns?category=${encodeURIComponent(category)}` : '/patterns'
      ),
    get: (id: string) => request<import('./editor/patterns').SyncedPattern>(`/patterns/${id}`),
    create: (data: {
      slug?: string;
      title: string;
      description?: string;
      category?: string;
      document: unknown;
      is_synced?: boolean;
    }) =>
      request<import('./editor/patterns').SyncedPattern>('/patterns', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    update: (
      id: string,
      data: {
        title?: string;
        description?: string;
        category?: string;
        document?: unknown;
        is_synced?: boolean;
      }
    ) =>
      request<import('./editor/patterns').SyncedPattern>(`/patterns/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      }),
    delete: (id: string) =>
      request<{ success: boolean }>(`/patterns/${id}`, {
        method: 'DELETE',
      }),
  },
};
