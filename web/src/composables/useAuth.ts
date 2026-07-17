import { ref } from 'vue'
import { api } from '../api'
import type { User } from '../types'

const token = ref<string | null>(localStorage.getItem('bc_token'))
const user = ref<User | null>(null)

export function useAuth() {
  function isLoggedIn() {
    return !!token.value
  }

  async function login(name: string, email: string, password: string) {
    const res = await api.login(name, email, password)
    token.value = res.token
    user.value = res.user
    localStorage.setItem('bc_token', res.token)
  }

  async function register(name: string, email: string, password: string, link?: string, isAdmin?: boolean) {
    const res = await api.register(name, email, password, link, isAdmin)
    token.value = res.token
    user.value = res.user
    localStorage.setItem('bc_token', res.token)
  }

  function logout() {
    token.value = null
    user.value = null
    localStorage.removeItem('bc_token')
  }

  return { token, user, isLoggedIn, login, register, logout }
}
