<template>
  <form class="bc-editor" @submit.prevent="submit">
    <div class="bc-editor-row" v-if="!isReply">
      <input v-model="form.name" placeholder="昵称 *" required />
      <input v-model="form.email" placeholder="邮箱 *" required />
      <input v-model="form.link" placeholder="网址（可选）" />
    </div>
    <div class="bc-editor-row bc-reply-hint" v-else>
      回复 @{{ replyTo }}
    </div>
    <textarea v-model="form.content" :placeholder="isReply ? `回复 ${replyTo}…` : '说点什么吧…'" required></textarea>
    <div class="bc-editor-actions">
      <button class="bc-btn" type="submit" :disabled="submitting">
        {{ submitting ? '提交中…' : (isReply ? '回复' : '发送') }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { api } from '../api'
import { useAuth } from '../composables/useAuth'

const props = withDefaults(defineProps<{
  pageKey: string
  siteName: string
  rid?: number
  replyTo?: string
}>(), { rid: 0, replyTo: '' })

const emit = defineEmits<{ (e: 'created'): void }>()

const auth = useAuth()
const isReply = props.rid !== 0
const submitting = ref(false)

const form = reactive({
  name: auth.user.value?.name || '',
  email: auth.user.value?.email || '',
  link: auth.user.value?.link || '',
  content: ''
})

function computedIsReply() {
  return () => props.rid !== 0
}

async function submit() {
  if (!form.content.trim()) return
  submitting.value = true
  try {
    await api.createComment({
      name: form.name,
      email: form.email,
      link: form.link,
      content: form.content,
      rid: props.rid,
      page_key: props.pageKey,
      site_name: props.siteName
    })
    form.content = ''
    emit('created')
  } catch (e: any) {
    alert('提交失败: ' + e.message)
  } finally {
    submitting.value = false
  }
}
</script>
