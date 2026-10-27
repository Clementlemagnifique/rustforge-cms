export type Role = 'admin' | 'editor' | 'author' | 'subscriber';

export type EntryStatus = 'draft' | 'published' | 'archived' | 'trash';

export interface User {
  id: string;
  email: string;
  username: string;
  display_name: string;
  role: Role;
  bio?: string;
  created_at: string;
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
  name: string;
  slug: string;
  taxonomy: string;
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
  blocks: any[];
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
  created_at: string;
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
