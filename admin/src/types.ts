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

export interface MediaVariant {
  path: string;
  url: string;
  width: number;
  height: number;
  size_bytes: number;
  mime_type: string;
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
  metadata?: {
    is_webp_optimized?: boolean;
    variants?: Record<string, MediaVariant>;
    original_path?: string;
  };
}

export interface PublicMenuItem {
  id: string;
  parent_id?: string | null;
  label: string;
  url: string;
  target?: '_self' | '_blank';
  type?: 'custom' | 'page' | 'post' | 'category' | 'tag';
  object_id?: string | null;
  classes?: string;
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
  counts: Record<string, number>; mail_mode: string;
}
export interface JobCapture { recipient: string; subject: string; text: string }

export interface SiteIconInfo {
  url?: string;
  icon_32?: string;
  icon_180?: string;
  icon_192?: string;
  icon_512?: string;
}

export interface FormField {
  name: string;
  label: string;
  type: 'text' | 'email' | 'tel' | 'textarea' | 'select' | 'checkbox';
  required?: boolean;
  placeholder?: string;
  options?: string[];
}

export interface FormItem {
  id: string;
  name: string;
  slug: string;
  description: string;
  fields: Record<string, unknown> | FormField[];
  settings: {
    success_message?: string;
    notify_email?: string;
    honeypot?: boolean;
    [key: string]: unknown;
  };
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface FormSubmissionItem {
  id: string;
  form_id: string;
  data: Record<string, unknown>;
  ip_hash: string;
  user_agent?: string | null;
  is_spam: boolean;
  is_read: boolean;
  created_at: string;
}

export interface ProductItem {
  id: string;
  title: string;
  slug: string;
  excerpt: string;
  description: string;
  price_cents: number;
  featured_media_id: string | null;
  file_path?: string | null;
  file_name?: string | null;
  download_limit: number;
  course_id?: string | null;
  status: 'draft' | 'published' | 'archived';
  sales_count: number;
  created_at: string;
  updated_at: string;
}

export interface ProductInput {
  title: string;
  slug?: string;
  excerpt?: string;
  description?: string;
  price_cents: number;
  featured_media_id?: string | null;
  file_path?: string | null;
  file_name?: string | null;
  download_limit?: number;
  course_id?: string | null;
  status?: 'draft' | 'published' | 'archived';
}

export interface DiscountItem {
  id: string;
  code: string;
  discount_type: 'percentage' | 'fixed_amount';
  amount: number;
  max_uses: number | null;
  uses_count: number;
  expires_at: string | null;
  is_active: boolean;
  created_at: string;
}

export interface DiscountInput {
  code: string;
  discount_type?: 'percentage' | 'fixed_amount';
  amount: number;
  max_uses?: number | null;
  expires_at?: string | null;
  is_active?: boolean;
}

export interface OrderItem {
  id: string;
  order_number: string;
  customer_email: string;
  customer_name: string;
  user_id?: string | null;
  subtotal_cents: number;
  discount_cents: number;
  total_cents: number;
  currency: string;
  gateway: string;
  gateway_tx_id?: string | null;
  status: 'pending' | 'completed' | 'refunded' | 'failed';
  discount_id?: string | null;
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface OrderLineItem {
  id: string;
  order_id: string;
  product_id: string;
  title: string;
  price_cents: number;
  download_count: number;
  created_at: string;
}

export interface OrderWithItems {
  order: OrderItem;
  items: OrderLineItem[];
}

export interface CommerceSettings {
  currency: string;
  currency_symbol: string;
  gateway_mode: string;
  stripe_public_key: string;
  stripe_secret_key: string;
  tax_rate_percent: number;
  require_login: boolean;
  default_download_limit: number;
}

export interface CourseItem {
  id: string;
  title: string;
  slug: string;
  description: string;
  content: string;
  featured_media_id: string | null;
  price_cents: number;
  is_published: boolean;
  created_by?: string | null;
  product_id?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CourseInput {
  title: string;
  slug?: string;
  description?: string;
  content?: string;
  featured_media_id?: string | null;
  price_cents?: number;
  is_published?: boolean;
  product_id?: string | null;
}

export interface CourseModuleItem {
  id: string;
  course_id: string;
  title: string;
  description: string;
  sort_order: number;
  created_at: string;
}

export interface LessonItem {
  id: string;
  course_id: string;
  module_id: string | null;
  title: string;
  slug: string;
  content: string;
  video_url?: string | null;
  duration_min: number;
  is_preview: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface LessonInput {
  module_id?: string | null;
  title: string;
  slug?: string;
  content?: string;
  video_url?: string | null;
  duration_min?: number;
  is_preview?: boolean;
  sort_order?: number;
}

export interface EnrollmentDetails {
  id: string;
  course_id: string;
  user_id: string;
  username: string;
  email: string;
  display_name: string;
  progress_pct: number;
  completed_at: string | null;
  created_at: string;
}

