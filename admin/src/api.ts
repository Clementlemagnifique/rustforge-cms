import type { User, Entry, ContentType, Media, PluginInfo, DashboardStats, ToolDefinition, AiStatus, AiChatTurn, AiChatResponse } from './types';

const API_BASE = '/api/v1';

function getToken(): string | null {
  return localStorage.getItem('rustforge_token') || localStorage.getItem('trustpress_token');
}

export function setToken(token: string | null) {
  if (token) {
    localStorage.setItem('rustforge_token', token);
    localStorage.setItem('trustpress_token', token);
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
    throw new Error(errorJson.error?.message || `Erreur HTTP ${res.status}`);
  }

  return res.json();
}

export const api = {
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
    list: (params?: { content_type?: string; q?: string; page?: number; per_page?: number }) => {
      const q = new URLSearchParams();
      if (params?.content_type) q.set('content_type', params.content_type);
      if (params?.q) q.set('q', params.q);
      if (params?.page) q.set('page', params.page.toString());
      if (params?.per_page) q.set('per_page', params.per_page.toString());
      return request<{ items: Entry[]; total: number; page: number; total_pages: number }>(`/entries?${q}`);
    },
    get: (id: string) => request<Entry>(`/entries/${id}`),
    create: (contentType: string, data: Partial<Entry>) =>
      request<Entry>(`/entries?content_type=${contentType}`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    update: (id: string, data: Partial<Entry>) =>
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

  media: {
    list: () => request<{ items: Media[]; total: number }>('/media'),
    upload: (file: File) => {
      const form = new FormData();
      form.append('file', file);
      return request<Media>('/media', {
        method: 'POST',
        body: form,
      });
    },
    delete: (id: string) =>
      request<{ success: boolean }>(`/media/${id}`, {
        method: 'DELETE',
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

  ai: {
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
      site_title: string;
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
};

