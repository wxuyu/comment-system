// API 客户端

export interface ApiConfig {
  apiBase: string;
  siteId: number;
  pageUrl: string;
}

export interface Comment {
  id: number;
  site_id: number;
  page_id: number;
  parent_id: number | null;
  root_id: number | null;
  nickname: string;
  email_hash: string;
  website: string | null;
  content: string;
  content_html: string;
  ip_region: string | null;
  status: 'pending' | 'approved' | 'spam' | 'trash';
  is_pinned: boolean;
  is_admin: boolean;
  vote_up: number;
  vote_down: number;
  created_at: string;
  updated_at: string;
  replies: Comment[] | null;
}

export interface PageResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

export class ApiClient {
  constructor(private config: ApiConfig) {}

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.config.apiBase}${path}`;
    const res = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.message || `HTTP ${res.status}`);
    }

    const json: ApiResponse<T> = await res.json();
    if (json.code !== 0) {
      throw new Error(json.message);
    }

    return json.data as T;
  }

  async getComments(page = 1, pageSize = 20, sort = 'newest'): Promise<PageResponse<Comment>> {
    const params = new URLSearchParams({
      site_id: String(this.config.siteId),
      page_url: this.config.pageUrl,
      page_num: String(page),
      page_size: String(pageSize),
      sort_by: sort,
    });
    return this.request<PageResponse<Comment>>(`/comments?${params}`);
  }

  async getComment(id: number): Promise<Comment> {
    return this.request<Comment>(`/comments/${id}`);
  }

  async createComment(data: {
    nickname: string;
    email: string;
    website?: string;
    content: string;
    parent_id?: number;
  }): Promise<Comment> {
    return this.request<Comment>('/comments', {
      method: 'POST',
      body: JSON.stringify({
        ...data,
        site_id: this.config.siteId,
        page_id: 0, // will be resolved server-side
        page_url: this.config.pageUrl,
      }),
    });
  }

  async voteComment(commentId: number, voteType: 'up' | 'down'): Promise<void> {
    await this.request(`/comments/${commentId}/vote`, {
      method: 'POST',
      body: JSON.stringify({ comment_id: commentId, vote_type: voteType }),
    });
  }

  async recordView(): Promise<void> {
    await this.request('/pages/view', {
      method: 'POST',
      body: JSON.stringify({
        site_id: this.config.siteId,
        page_url: this.config.pageUrl,
        page_title: document.title,
      }),
    });
  }

  async uploadImage(file: File): Promise<{ url: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`${this.config.apiBase}/upload`, {
      method: 'POST',
      body: formData,
    });
    const json: ApiResponse<{ url: string }> = await res.json();
    if (json.code !== 0) throw new Error(json.message);
    return json.data!;
  }

  async getPageInfo(): Promise<{ view_count: number; comment_count: number }> {
    const params = new URLSearchParams({
      site_id: String(this.config.siteId),
      page_url: this.config.pageUrl,
    });
    return this.request(`/pages/info?${params}`);
  }
}
