<template>
  <div class="bc-root">
    <header class="bc-header" v-if="showHeader">
      <div class="bc-title">
        <span class="bc-count">{{ total }} 条评论</span>
      </div>
      <div class="bc-auth">
        <template v-if="auth.isLoggedIn() && auth.user.value">
          <span class="bc-user">{{ auth.user.value.name }}</span>
          <button class="bc-btn-link" @click="auth.logout()">退出</button>
        </template>
        <template v-else>
          <button class="bc-btn-link" @click="showLogin = !showLogin">{{ showLogin ? '取消' : '登录' }}</button>
        </template>
      </div>
    </header>

    <div class="bc-login" v-if="showLogin && !auth.isLoggedIn()">
      <input v-model="loginForm.name" placeholder="昵称" />
      <input v-model="loginForm.email" placeholder="邮箱" />
      <input v-model="loginForm.password" type="password" placeholder="密码" />
      <button class="bc-btn" @click="doLogin">登录</button>
    </div>

    <CommentEditor
      :pageKey="pageKey"
      :siteName="siteName"
      @created="onCreated"
    />

    <CommentList
      ref="listRef"
      :pageKey="pageKey"
      :siteName="siteName"
      @need-login="showLogin = true"
    />

    <AdminPanel v-if="auth.user.value?.is_admin" :siteName="siteName" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import CommentEditor from './components/CommentEditor.vue'
import CommentList from './components/CommentList.vue'
import AdminPanel from './components/AdminPanel.vue'
import { useAuth } from './composables/useAuth'
import { api } from './api'

const props = withDefaults(defineProps<{
  pageKey?: string
  siteName?: string
  showHeader?: boolean
}>(), {
  pageKey: () => (window.location.pathname || 'default'),
  siteName: 'Default Site',
  showHeader: true
})

const auth = useAuth()
const showLogin = ref(false)
const loginForm = ref({ name: '', email: '', password: '' })
const listRef = ref<InstanceType<typeof CommentList> | null>(null)
const total = ref(0)

onMounted(async () => {
  try {
    const stat = await api.stat()
    total.value = stat.comments
  } catch (e) {
    total.value = 0
  }
})

async function doLogin() {
  try {
    await auth.login(loginForm.value.name, loginForm.value.email, loginForm.value.password)
    showLogin.value = false
    loginForm.value = { name: '', email: '', password: '' }
  } catch (e: any) {
    alert('登录失败: ' + e.message)
  }
}

function onCreated() {
  listRef.value?.reload()
}
</script>
