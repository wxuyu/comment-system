<template>
  <div class="bc-item" :class="{ 'bc-admin': comment.is_admin, 'bc-pinned': comment.is_pinned }">
    <div class="bc-avatar" :style="{ background: avatarColor }">{{ comment.name.charAt(0).toUpperCase() }}</div>
    <div class="bc-body">
      <div class="bc-meta">
        <span class="bc-name" :class="{ 'bc-badge': comment.badge_name }" :style="badgeStyle">{{ comment.name }}</span>
        <span v-if="comment.is_admin" class="bc-tag bc-tag-admin">管理员</span>
        <span class="bc-time">{{ formatTime(comment.createdAt) }}</span>
        <span v-if="comment.is_pending" class="bc-tag bc-tag-pending">待审核</span>
      </div>
      <div class="bc-content" v-html="comment.content_rendered"></div>
      <div class="bc-actions">
        <button class="bc-btn-link" @click="toggleReply">{{ replying ? '取消' : '回复' }}</button>
        <button class="bc-btn-link" @click="voteUp">{{ voted ? '已赞' : '赞' }} ({{ comment.vote_up }})</button>
        <template v-if="canDelete">
          <button class="bc-btn-link bc-danger" @click="doDelete">删除</button>
          <button class="bc-btn-link" @click="toggleCollapse">{{ comment.is_collapsed ? '展开' : '折叠' }}</button>
        </template>
      </div>

      <CommentEditor
        v-if="replying"
        :pageKey="pageKey"
        :siteName="siteName"
        :rid="comment.id"
        :replyTo="comment.name"
        @created="onReplied"
      />

      <div class="bc-children" v-if="comment.children && comment.children.length">
        <CommentItem
          v-for="child in comment.children"
          :key="child.id"
          :comment="child"
          :siteName="siteName"
          :pageKey="pageKey"
          @updated="$emit('updated')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import CommentEditor from './CommentEditor.vue'
import { api } from '../api'
import type { CookedComment } from '../types'
import { useAuth } from '../composables/useAuth'

const props = defineProps<{ comment: CookedComment; pageKey: string; siteName: string }>()
const emit = defineEmits<{ (e: 'updated'): void }>()

const auth = useAuth()
const replying = ref(false)
const voted = ref(false)

const canDelete = computed(() => auth.user.value?.is_admin || auth.user.value?.id === props.comment.user_id)

const avatarColor = computed(() => {
  const colors = ['#f56a00', '#7265e6', '#00a2ae', '#1890ff', '#2f54eb']
  let sum = 0
  for (const ch of props.comment.name) sum += ch.charCodeAt(0)
  return colors[sum % colors.length]
})

const badgeStyle = computed(() => props.comment.badge_color ? { color: props.comment.badge_color } : {})

function formatTime(t: string) {
  if (!t) return ''
  const d = new Date(t)
  return d.toLocaleString('zh-CN')
}

function toggleReply() { replying.value = !replying.value }
function toggleCollapse() { emit('updated') }

async function voteUp() {
  try {
    await api.vote(props.comment.id, 'comment_up')
    voted.value = true
    props.comment.vote_up++
  } catch (e: any) {
    if (String(e.message).includes('401')) alert('请先登录')
  }
}

async function doDelete() {
  if (!confirm('确定删除该评论？')) return
  await api.deleteComment(props.comment.id)
  emit('updated')
}

function onReplied() {
  replying.value = false
  emit('updated')
}
</script>
