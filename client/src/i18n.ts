// 国际化

export type Locale = 'zh-CN' | 'zh-TW' | 'en' | 'ja' | 'ko';

const messages: Record<Locale, Record<string, string>> = {
  'zh-CN': {
    comments: '评论',
    noComments: '暂无评论，来说点什么吧',
    writeComment: '写评论',
    reply: '回复',
    cancel: '取消',
    submit: '提交',
    submitting: '提交中...',
    nickname: '昵称',
    email: '邮箱',
    website: '网站（可选）',
    content: '评论内容',
    placeholder: '写下你的想法...',
    loadMore: '加载更多',
    loading: '加载中...',
    sortNewest: '最新',
    sortOldest: '最早',
    sortHottest: '最热',
    sortVotes: '最多投票',
    viaAdmin: '管理员',
    pending: '评论待审核',
    pendingReview: '你的评论已提交，审核通过后将显示',
    voteUp: '赞同',
    voteDown: '反对',
    views: '浏览',
    secondsAgo: '秒前',
    minutesAgo: '分钟前',
    hoursAgo: '小时前',
    daysAgo: '天前',
    justNow: '刚刚',
    imageUpload: '上传图片',
    preview: '预览',
    errorLoad: '加载评论失败',
    errorSubmit: '提交失败，请重试',
    nicknameRequired: '请输入昵称',
    emailRequired: '请输入有效邮箱',
    contentRequired: '请输入评论内容',
    toggleTheme: '切换夜间模式',
  },
  'en': {
    comments: 'Comments',
    noComments: 'No comments yet. Be the first!',
    writeComment: 'Write a comment',
    reply: 'Reply',
    cancel: 'Cancel',
    submit: 'Submit',
    submitting: 'Submitting...',
    nickname: 'Nickname',
    email: 'Email',
    website: 'Website (optional)',
    content: 'Content',
    placeholder: 'Write your thoughts...',
    loadMore: 'Load more',
    loading: 'Loading...',
    sortNewest: 'Newest',
    sortOldest: 'Oldest',
    sortHottest: 'Hottest',
    sortVotes: 'Most votes',
    viaAdmin: 'Admin',
    pending: 'Pending review',
    pendingReview: 'Your comment has been submitted and will appear after review.',
    voteUp: 'Upvote',
    voteDown: 'Downvote',
    views: 'views',
    secondsAgo: 's ago',
    minutesAgo: 'm ago',
    hoursAgo: 'h ago',
    daysAgo: 'd ago',
    justNow: 'just now',
    imageUpload: 'Upload image',
    preview: 'Preview',
    errorLoad: 'Failed to load comments',
    errorSubmit: 'Failed to submit, please retry',
    nicknameRequired: 'Nickname is required',
    emailRequired: 'Valid email is required',
    contentRequired: 'Content is required',
    toggleTheme: 'Toggle dark mode',
  },
};

export class I18n {
  constructor(private locale: Locale) {}

  t(key: string): string {
    return messages[this.locale]?.[key] ?? messages['zh-CN'][key] ?? key;
  }

  setLocale(locale: Locale) {
    this.locale = locale;
  }
}
