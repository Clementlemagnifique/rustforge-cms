export type Role = 'admin' | 'editor' | 'author' | 'subscriber';

export interface AdminMenuItem {
  id: string; parent: string | null; label: string; label_key: string; route: string | null;
  module: string | null; roles: Role[]; state: 'ready' | 'partial' | 'planned';
  design: string; features: string[]; tasks: string[]; source: string;
}
export interface AdminNavigation {
  schema_version: number; items: AdminMenuItem[]; catalog: AdminMenuItem[];
  modules: Record<string, boolean>; supported_modules: string[];
}

export type EntryStatus = 'draft' | 'published' | 'archived' | 'trash';

export interface User {
  id: string;
  email: string;
  username: string;
  display_name: string;
  role: Role;
  bio?: string;
  created_at: string;
  updated_at?: string;
  last_login_at?: string | null;
  entry_count?: number;
}

export interface UserSession {
  id: string;
  user_id: string;
  created_at: string;
  expires_at: string;
}

export interface ContentType {
  id: string;
  slug: string;
  name: string;
  name_singular: string;
  description: string;
  icon: string;
  hierarchical: boolean;
  has_revisions: boolean;
  is_system: boolean;
  fields: any[];
  entry_count: number;
}

export interface TermRef {
  id: string;
  taxonomy_id?: string;
  parent_id?: string | null;
  name: string;
  slug: string;
  taxonomy: string;
  description?: string;
  entry_count?: number;
  created_at?: string;
}

export interface Entry {
  id: string;
  content_type_id: string;
  content_type: string;
  parent_id?: string;
  author_id?: string;
  author_name?: string;
  title: string;
  slug: string;
  excerpt: string;
  body: string;
  body_html: string;
  blocks: unknown;
  intent: Record<string, any>;
  design_tokens: Record<string, any>;
  status: EntryStatus;
  fields: Record<string, any>;
  featured_media_id?: string;
  featured_media_url?: string;
  published_at?: string;
  scheduled_unpublish_at?: string | null;
  created_at: string;
  updated_at: string;
  terms: TermRef[];
}

export interface MediaFolder {
  id: string;
  parent_id?: string | null;
  name: string;
  slug: string;
  created_at: string;
}

export interface Media {
  id: string;
  path: string;
  url: string;
  original_name: string;
  mime_type: string;
  size_bytes: number;
  width?: number;
  height?: number;
  alt: string;
  caption: string;
  folder_id?: string | null;
  webp_path?: string | null;
  webp_url?: string | null;
  created_at: string;
}

export interface PublicMenuItem {
  id: string;
  parent_id?: string | null;
  label: string;
  url: string;
  target?: '_self' | '_blank';
  type?: 'custom' | 'page' | 'category' | 'tag';
  object_id?: string | null;
  badge?: string;
  subtitle?: string;
  children?: PublicMenuItem[];
}

export interface PublicMenus {
  header: PublicMenuItem[];
  footer: PublicMenuItem[];
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  hooks: string[];
  capabilities: string[];
  granted: string[];
  enabled: boolean;
  file: string;
  error?: string;
}

export interface DashboardStats {
  content_types: {
    slug: string;
    name: string;
    icon: string;
    published: number;
    drafts: number;
    scheduled: number;
  }[];
  media: number;
  media_bytes: number;
  users: number;
  terms: number;
  recent: Entry[];
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: any;
}

export interface AiStatus {
  provider: string;
  model: string;
  endpoint?: string;
  reachable: boolean;
  model_available: boolean;
  models: string[];
  error?: string;
}

export interface AiChatTurn {
  role: 'user' | 'assistant';
  content: string;
}

export interface AiAction {
  tool: string;
  arguments: Record<string, any>;
  success: boolean;
  message: string;
  data?: any;
}

export interface AiChatResponse {
  reply: string;
  actions: AiAction[];
  model: string;
  duration_ms: number;
}
export interface MigrationInspection {
  source_url: string; entries: number; terms: number; media_references: number;
  by_type: Record<string, number>; sample: { title: string; slug: string; content_type: string; status: string }[];
  warnings: string[]; blocking_conflicts: number; seal: string;
}
export interface MigrationImportResult {
  status: 'content_imported';
  report: { entries_imported: number; terms_imported: number; warnings: string[] };
  checks_remaining: string[];
}

export type CommentStatus = 'pending' | 'approved' | 'spam' | 'trash';

export interface CommentItem {
  id: string;
  entry_id: string;
  parent_id?: string | null;
  author_id?: string | null;
  author_name: string;
  author_email?: string | null;
  author_url?: string | null;
  author_ip?: string | null;
  author_user_agent?: string | null;
  content: string;
  content_html: string;
  status: CommentStatus;
  created_at: string;
  updated_at: string;
}

export interface CommentCounts {
  all: number;
  pending: number;
  approved: number;
  spam: number;
  trash: number;
}

export interface CommentListResponse {
  items: CommentItem[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}


export interface JobSummary {
  id: string; kind: string; state: string; payload_version: number;
  attempts: number; max_attempts: number; available_at: string;
  lease_until: string | null; error_code: string | null;
  created_at: string; updated_at: string; finished_at: string | null;
  capture_available: boolean;
}
export interface JobsPage {
  items: JobSummary[]; total: number; page: number; pages: number;
  counts: Record<string, number>; mail_mode: 'capture_only';
}
export interface JobCapture { recipient: string; subject: string; text: string }
