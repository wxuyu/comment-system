<template>
  <div class="bc-list">
    <div v-if="loading" class="bc-loading">加载中…</div>
    <div v-else-if="comments.length === 0" class="bc-empty">暂无评论，来抢沙发吧～</div>
    <CommentItem
      v-for="c in comments"
      :key="c.id"
      :comment="c"
      :siteName="siteName"
      :pageKey="pageKey"
      @updated="reload"
    />
    <button v-if="hasMore" class="bc-btn bc-loadmore" @click="loadMore">加载更多</button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import CommentItem from './CommentItem.vue'
import { api } from '../api'
import type { CookedComment } from '../types'

const props = defineProps<{ pageKey: string; siteName: string }>()
const emit = defineEmits<{ (e: 'need-login'): void }>()

const comments = ref<CookedComment[]>([])
const loading = ref(false)
const offset = ref(0)
const limit = 20
const hasMore = ref(false)

async function fetchComments(reset = false) {
  loading.value = true
  try {
    const res = await api.listComments(props.pageKey, props.siteName, {
      limit,
      offset: reset ? 0 : offset.value
    })
    if (reset) {
      comments.value = res.comments
      offset.value = res.comments.length
    } else {
      comments.value.push(...res.comments)
      offset.value += res.comments.length
    }
    hasMore.value = offset.value < res.count
  } catch (e: any) {
    if (String(e.message).includes('401') || String(e.message).includes('Unauthorized')) {
      emit('need-login')
    }
  } finally {
    loading.value = false
  }
}

function reload() {
  offset.value = 0
  fetchComments(true)
}

function loadMore() {
  fetchComments(false)
}

defineExpose({ reload })

onMounted(() => reload())
</script>
