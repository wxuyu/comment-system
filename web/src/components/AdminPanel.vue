<template>
  <div class="bc-admin-panel" v-if="visible">
    <h3 class="bc-admin-title">管理面板</h3>
    <div class="bc-admin-section">
      <h4>站点</h4>
      <ul>
        <li v-for="s in sites" :key="s.id">{{ s.name }} <small>{{ s.urls }}</small></li>
      </ul>
      <div class="bc-admin-add">
        <input v-model="newSite.name" placeholder="站点名" />
        <input v-model="newSite.urls" placeholder="URLs（逗号分隔）" />
        <button class="bc-btn" @click="addSite">新建站点</button>
      </div>
    </div>
    <div class="bc-admin-section">
      <h4>用户</h4>
      <ul>
        <li v-for="u in users" :key="u.id">
          {{ u.name }} ({{ u.email }}) <span v-if="u.is_admin" class="bc-tag bc-tag-admin">管理员</span>
        </li>
      </ul>
      <div class="bc-admin-add">
        <input v-model="newUser.name" placeholder="昵称" />
        <input v-model="newUser.email" placeholder="邮箱" />
        <input v-model="newUser.password" type="password" placeholder="密码" />
        <label><input type="checkbox" v-model="newUser.is_admin" /> 管理员</label>
        <button class="bc-btn" @click="addUser">新建用户</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '../api'
import type { Site, User } from '../types'

defineProps<{ siteName: string }>()

const visible = ref(false)
const sites = ref<Site[]>([])
const users = ref<User[]>([])
const newSite = ref({ name: '', urls: '' })
const newUser = ref({ name: '', email: '', password: '', is_admin: false })

onMounted(load)

async function load() {
  try {
    sites.value = await api.listSites()
    users.value = await api.listUsers()
    visible.value = true
  } catch (e) {
    visible.value = false
  }
}

async function addSite() {
  if (!newSite.value.name) return
  await api.createSite(newSite.value.name, newSite.value.urls)
  newSite.value = { name: '', urls: '' }
  sites.value = await api.listSites()
}

async function addUser() {
  if (!newUser.value.name || !newUser.value.email) return
  await api.createUser({ ...newUser.value })
  newUser.value = { name: '', email: '', password: '', is_admin: false }
  users.value = await api.listUsers()
}
</script>
