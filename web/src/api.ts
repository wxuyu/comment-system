import type { CommentListResp, CookedComment, User, StatResp } from './types'

const API_BASE = (import.meta.env.VITE_API_BASE as string) || ''

function getToken(): string | null {
  return localStorage.getItem('bc_token')
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {})
  }
  const token = getToken()
  if (token) headers['Authorization'] = `Bearer ${token}`
  const res = await fetch(`${API_BASE}${path}`, { ...options, headers })
  if (!res.ok) {
    let msg = `HTTP ${res.status}`
    try {
      const body = await res.json()
      msg = body.msg || msg
    } catch {
      /* ignore */
    }
    throw new Error(msg)
  }
  if (res.status === 204) return undefined as T
  return res.json() as Promise<T>
}

export type VoteType = 'comment_up' | 'comment_down' | 'page_up' | 'page_down'

export const api = {
  // Comments
  listComments: (pageKey: string, siteName: string, opts: { limit?: number; offset?: number; flat?: boolean; sortBy?: string } = {}) => {
    const params = new URLSearchParams({ page_key: pageKey, site_name: siteName })
    if (opts.limit) params.set('limit', String(opts.limit))
    if (opts.offset) params.set('offset', String(opts.offset))
    if (opts.flat) params.set('flat_mode', 'true')
    if (opts.sortBy) params.set('sort_by', opts.sortBy)
    return request<CommentListResp>(`/api/v2/comments?${params.toString()}`)
  },
  createComment: (data: {
    name: string; email: string; link?: string; content: string; rid?: number
    page_key: string; page_title?: string; site_name: string; ua?: string
  }) => request<CookedComment>(`/api/v2/comments`, { method: 'POST', body: JSON.stringify(data) }),
  updateComment: (id: number, data: Partial<{ content: string; is_collapsed: boolean; is_pending: boolean; is_pinned: boolean }>) =>
    request<CookedComment>(`/api/v2/comment/${id}`, { method: 'POST', body: JSON.stringify(data) }),
  deleteComment: (id: number) => request<void>(`/api/v2/comment/${id}`, { method: 'DELETE' }),

  // Vote
  vote: (targetId: number, type: VoteType) =>
    request<{ vote_up: number; vote_down: number }>(`/api/v2/votes`, { method: 'POST', body: JSON.stringify({ target_id: targetId, type_: type }) }),
  getVote: (targetId: number, type: VoteType) =>
    request<{ vote_up: number; vote_down: number }>(`/api/v2/vote?target_id=${targetId}&type_=${type}`),

  // Page PV
  pagePV: (pageKey: string, siteName: string) =>
    request<{ page_key: string; pv: number }>(`/api/v2/pages/pv`, { method: 'POST', body: JSON.stringify({ page_key: pageKey, site_name: siteName }) }),

  // Stat
  stat: () => request<StatResp>(`/api/v2/stat`),

  // Config
  conf: () => request<{ front_end: any; version: string }>(`/api/v2/conf`),

  // Auth
  login: (name: string, email: string, password: string) =>
    request<{ token: string; user: User }>(`/api/v2/auth/login`, { method: 'POST', body: JSON.stringify({ email, password }) }),
  register: (name: string, email: string, password: string, link?: string, is_admin?: boolean) =>
    request<{ token: string; user: User }>(`/api/v2/auth/register`, { method: 'POST', body: JSON.stringify({ name, email, password, link, is_admin }) }),

  // Admin
  listSites: () => request<any[]>(`/api/v2/sites`),
  createSite: (name: string, urls: string) => request<any>(`/api/v2/sites`, { method: 'POST', body: JSON.stringify({ name, urls }) }),
  listUsers: () => request<User[]>(`/api/v2/users`),
  createUser: (data: { name: string; email: string; password?: string; link?: string; is_admin?: boolean }) =>
    request<User>(`/api/v2/users`, { method: 'POST', body: JSON.stringify(data) }),
  updateUser: (id: number, data: Partial<{ name: string; email: string; link: string; is_admin: boolean; password: string }>) =>
    request<User>(`/api/v2/user/${id}`, { method: 'POST', body: JSON.stringify(data) }),
  deleteUser: (id: number) => request<void>(`/api/v2/user/${id}`, { method: 'DELETE' }),

  // Notify
  notifies: () => request<any[]>(`/api/v2/notify`),
  readAll: () => request<void>(`/api/v2/notify/read-all`, { method: 'POST' }),
}

