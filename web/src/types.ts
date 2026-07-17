export interface CookedComment {
  id: number
  content: string
  contentRendered: string
  page_key: string
  site_name: string
  user_id: number
  name: string
  email: string
  link: string
  badge_name: string
  badge_color: string
  is_admin: boolean
  is_verified: boolean
  ua: string
  ip: string
  rid: number
  rootID: number
  is_collapsed: boolean
  is_pending: boolean
  is_pinned: boolean
  vote_up: number
  vote_down: number
  createdAt: string
  updatedAt: string
  children?: CookedComment[]
  ipRegion?: string | null
}

export interface CookedPage {
  key: string
  title: string
  admin_only: boolean
  site_name: string
  pv: number
  vote_up: number
  vote_down: number
  createdAt: string
  updatedAt: string
}

export interface CommentListResp {
  comments: CookedComment[]
  count: number
  roots_count: number
  page?: CookedPage
}

export interface User {
  id: number
  name: string
  email: string
  link: string
  is_admin: boolean
  badge_name: string
  badge_color: string
}

export interface StatResp {
  comments: number
  pages: number
  users: number
  sites: number
}
