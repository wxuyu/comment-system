// 评论组件核心 Widget

import { ApiClient, ApiConfig, Comment, PageResponse } from './api';
import { I18n, Locale } from './i18n';

export interface WidgetConfig extends ApiConfig {
  pageTitle?: string;
  darkMode?: boolean;
  locale?: Locale;
  maxNesting?: number;
}

export class CommentWidget {
  private api: ApiClient;
  private i18n: I18n;
  private container: HTMLElement;
  private config: WidgetConfig;
  private currentPage = 1;
  private sortBy = 'newest';
  private replyTo: number | null = null;

  constructor(container: string | HTMLElement, config: WidgetConfig) {
    this.container = typeof container === 'string'
      ? document.querySelector(container) as HTMLElement
      : container;

    if (!this.container) {
      throw new Error('CommentWidget: container not found');
    }

    this.config = {
      maxNesting: 3,
      ...config,
    };

    this.api = new ApiClient({
      apiBase: this.config.apiBase,
      siteId: this.config.siteId,
      pageUrl: this.config.pageUrl,
    });

    this.i18n = new I18n(this.config.locale || 'zh-CN');
  }

  async init() {
    this.renderBase();
    this.bindEvents();

    if (this.config.darkMode) {
      this.container.classList.add('cw-dark');
    }

    // 记录浏览
    try { await this.api.recordView(); } catch {}

    // 加载评论
    await this.loadComments();
  }

  private renderBase() {
    this.container.innerHTML = `
      <div class="cw-comment-system">
        <div class="cw-header">
          <h3 class="cw-title">
            <span class="cw-title-text">${this.i18n.t('comments')}</span>
            <span class="cw-count">(0)</span>
          </h3>
          <div class="cw-header-actions">
            <select class="cw-sort">
              <option value="newest">${this.i18n.t('sortNewest')}</option>
              <option value="oldest">${this.i18n.t('sortOldest')}</option>
              <option value="hottest">${this.i18n.t('sortHottest')}</option>
              <option value="votes">${this.i18n.t('sortVotes')}</option>
            </select>
            <button class="cw-theme-toggle" title="${this.i18n.t('toggleTheme')}">🌙</button>
          </div>
        </div>

        <div class="cw-form-container">
          <form class="cw-form">
            <div class="cw-form-row">
              <input type="text" class="cw-input cw-nickname" placeholder="${this.i18n.t('nickname')}" maxlength="50" required>
              <input type="email" class="cw-input cw-email" placeholder="${this.i18n.t('email')}" maxlength="100" required>
              <input type="url" class="cw-input cw-website" placeholder="${this.i18n.t('website')}" maxlength="200">
            </div>
            <textarea class="cw-textarea" placeholder="${this.i18n.t('placeholder')}" maxlength="5000" required rows="3"></textarea>
            <div class="cw-form-footer">
              <button type="button" class="cw-btn cw-btn-upload" title="${this.i18n.t('imageUpload')}">
                📷
                <input type="file" class="cw-file-input" accept="image/*" hidden>
              </button>
              <button type="submit" class="cw-btn cw-btn-submit">${this.i18n.t('submit')}</button>
            </div>
          </form>
          <div class="cw-pending-msg" style="display:none">${this.i18n.t('pendingReview')}</div>
        </div>

        <div class="cw-list">
          <div class="cw-loading">${this.i18n.t('loading')}</div>
        </div>

        <div class="cw-pagination" style="display:none">
          <button class="cw-btn cw-btn-loadmore">${this.i18n.t('loadMore')}</button>
        </div>
      </div>
    `;

    // 记住表单数据
    this.loadDraft();
  }

  private bindEvents() {
    const form = this.container.querySelector('.cw-form') as HTMLFormElement;
    const sortSelect = this.container.querySelector('.cw-sort') as HTMLSelectElement;
    const themeBtn = this.container.querySelector('.cw-theme-toggle') as HTMLButtonElement;
    const loadMore = this.container.querySelector('.cw-btn-loadmore') as HTMLButtonElement;
    const uploadBtn = this.container.querySelector('.cw-btn-upload') as HTMLButtonElement;
    const fileInput = this.container.querySelector('.cw-file-input') as HTMLInputElement;

    form?.addEventListener('submit', (e) => {
      e.preventDefault();
      this.handleSubmit();
    });

    // 自动保存草稿
    const textarea = this.container.querySelector('.cw-textarea') as HTMLTextAreaElement;
    const nickname = this.container.querySelector('.cw-nickname') as HTMLInputElement;
    const email = this.container.querySelector('.cw-email') as HTMLInputElement;
    const website = this.container.querySelector('.cw-website') as HTMLInputElement;

    [textarea, nickname, email, website].forEach(el => {
      el?.addEventListener('input', () => this.saveDraft());
    });

    sortSelect?.addEventListener('change', () => {
      this.sortBy = sortSelect.value;
      this.currentPage = 1;
      this.loadComments();
    });

    themeBtn?.addEventListener('click', () => {
      this.container.classList.toggle('cw-dark');
      themeBtn.textContent = this.container.classList.contains('cw-dark') ? '☀️' : '🌙';
    });

    loadMore?.addEventListener('click', () => {
      this.currentPage++;
      this.loadComments(true);
    });

    uploadBtn?.addEventListener('click', () => fileInput?.click());
    fileInput?.addEventListener('change', () => this.handleUpload(fileInput));
  }

  private async loadComments(append = false) {
    const list = this.container.querySelector('.cw-list') as HTMLElement;
    const loading = this.container.querySelector('.cw-loading') as HTMLElement;
    const pagination = this.container.querySelector('.cw-pagination') as HTMLElement;
    const count = this.container.querySelector('.cw-count') as HTMLElement;

    loading.style.display = 'block';
    pagination.style.display = 'none';

    try {
      const data = await this.api.getComments(this.currentPage, 20, this.sortBy);

      count.textContent = `(${data.total})`;

      if (!append) {
        list.innerHTML = '';
      } else {
        loading.remove();
      }

      if (data.items.length === 0 && !append) {
        list.innerHTML = `<div class="cw-empty">${this.i18n.t('noComments')}</div>`;
        return;
      }

      data.items.forEach(comment => {
        const el = this.renderComment(comment);
        list.appendChild(el);
      });

      // 显示加载更多
      if (this.currentPage < data.total_pages) {
        pagination.style.display = 'flex';
        pagination.innerHTML = `<button class="cw-btn cw-btn-loadmore">${this.i18n.t('loadMore')}</button>`;
        pagination.querySelector('.cw-btn-loadmore')?.addEventListener('click', () => {
          this.currentPage++;
          this.loadComments(true);
        });
      }
    } catch (e) {
      list.innerHTML = `<div class="cw-error">${this.i18n.t('errorLoad')}</div>`;
    } finally {
      loading.style.display = 'none';
    }
  }

  private renderComment(comment: Comment, depth = 0): HTMLElement {
    const div = document.createElement('div');
    div.className = `cw-comment ${comment.is_pinned ? 'cw-pinned' : ''}`;
    div.dataset.id = String(comment.id);

    const timeAgo = this.formatTime(comment.created_at);
    const maxDepth = this.config.maxNesting || 3;

    div.innerHTML = `
      <div class="cw-comment-inner">
        <div class="cw-comment-avatar">
          ${this.getAvatarHtml(comment.nickname, comment.is_admin)}
        </div>
        <div class="cw-comment-body">
          <div class="cw-comment-meta">
            <span class="cw-comment-author">
              ${this.escapeHtml(comment.nickname)}
              ${comment.is_admin ? `<span class="cw-badge">${this.i18n.t('viaAdmin')}</span>` : ''}
            </span>
            ${comment.ip_region ? `<span class="cw-region">${comment.ip_region}</span>` : ''}
            <span class="cw-time">${timeAgo}</span>
            ${comment.is_pinned ? '<span class="cw-pin-badge">📌 置顶</span>' : ''}
          </div>
          <div class="cw-comment-content">${comment.content_html || this.escapeHtml(comment.content)}</div>
          <div class="cw-comment-actions">
            <button class="cw-vote-btn" data-action="up">👍 <span>${comment.vote_up}</span></button>
            <button class="cw-vote-btn" data-action="down">👎 <span>${comment.vote_down}</span></button>
            ${depth < maxDepth ? `<button class="cw-reply-btn">${this.i18n.t('reply')}</button>` : ''}
          </div>
          ${depth < maxDepth ? '<div class="cw-reply-form" style="display:none"></div>' : ''}
        </div>
      </div>
      <div class="cw-replies"></div>
    `;

    // 递归渲染回复
    if (comment.replies && comment.replies.length > 0) {
      const repliesContainer = div.querySelector('.cw-replies') as HTMLElement;
      comment.replies.forEach(reply => {
        repliesContainer.appendChild(this.renderComment(reply, depth + 1));
      });
    }

    // 绑定事件
    this.bindCommentEvents(div, comment);

    return div;
  }

  private bindCommentEvents(el: HTMLElement, comment: Comment) {
    // 投票
    el.querySelectorAll('.cw-vote-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        const action = (btn as HTMLElement).dataset.action as 'up' | 'down';
        try {
          await this.api.voteComment(comment.id, action);
          const span = btn.querySelector('span');
          if (span) {
            span.textContent = String(parseInt(span.textContent || '0') + 1);
          }
          btn.classList.add('cw-voted');
        } catch {}
      });
    });

    // 回复按钮
    const replyBtn = el.querySelector('.cw-reply-btn');
    replyBtn?.addEventListener('click', () => {
      const formContainer = el.querySelector('.cw-reply-form') as HTMLElement;
      const isVisible = formContainer.style.display !== 'none';

      // 关闭所有其他回复表单
      this.container.querySelectorAll('.cw-reply-form').forEach(f => {
        (f as HTMLElement).style.display = 'none';
      });

      if (!isVisible) {
        this.replyTo = comment.id;
        formContainer.style.display = 'block';
        formContainer.innerHTML = this.renderReplyForm(comment.nickname);
        this.bindReplyForm(formContainer, comment.id);
      } else {
        this.replyTo = null;
      }
    });
  }

  private renderReplyForm(replyToName: string): string {
    return `
      <div class="cw-reply-form-inner">
        <div class="cw-reply-to">@${this.escapeHtml(replyToName)}</div>
        <div class="cw-form-row">
          <input type="text" class="cw-input cw-r-nickname" placeholder="${this.i18n.t('nickname')}" maxlength="50" required>
          <input type="email" class="cw-input cw-r-email" placeholder="${this.i18n.t('email')}" maxlength="100" required>
        </div>
        <textarea class="cw-textarea" placeholder="${this.i18n.t('placeholder')}" maxlength="5000" required rows="2"></textarea>
        <div class="cw-form-footer">
          <button type="button" class="cw-btn cw-btn-cancel">${this.i18n.t('cancel')}</button>
          <button type="button" class="cw-btn cw-btn-submit">${this.i18n.t('submit')}</button>
        </div>
      </div>
    `;
  }

  private bindReplyForm(container: HTMLElement, parentId: number) {
    const cancelBtn = container.querySelector('.cw-btn-cancel');
    const submitBtn = container.querySelector('.cw-btn-submit');

    cancelBtn?.addEventListener('click', () => {
      container.style.display = 'none';
      this.replyTo = null;
    });

    submitBtn?.addEventListener('click', async () => {
      const nickname = (container.querySelector('.cw-r-nickname') as HTMLInputElement).value.trim();
      const email = (container.querySelector('.cw-r-email') as HTMLInputElement).value.trim();
      const content = (container.querySelector('.cw-textarea') as HTMLTextAreaElement).value.trim();

      if (!nickname) { alert(this.i18n.t('nicknameRequired')); return; }
      if (!email || !email.includes('@')) { alert(this.i18n.t('emailRequired')); return; }
      if (!content) { alert(this.i18n.t('contentRequired')); return; }

      submitBtn.textContent = this.i18n.t('submitting');
      submitBtn.setAttribute('disabled', 'true');

      try {
        await this.api.createComment({
          nickname,
          email,
          content,
          parent_id: parentId,
        });
        container.style.display = 'none';
        this.replyTo = null;
        this.currentPage = 1;
        await this.loadComments();
      } catch {
        alert(this.i18n.t('errorSubmit'));
        submitBtn.textContent = this.i18n.t('submit');
        submitBtn.removeAttribute('disabled');
      }
    });
  }

  private async handleSubmit() {
    const nickname = (this.container.querySelector('.cw-nickname') as HTMLInputElement).value.trim();
    const email = (this.container.querySelector('.cw-email') as HTMLInputElement).value.trim();
    const website = (this.container.querySelector('.cw-website') as HTMLInputElement).value.trim();
    const content = (this.container.querySelector('.cw-textarea') as HTMLTextAreaElement).value.trim();
    const submitBtn = this.container.querySelector('.cw-btn-submit') as HTMLButtonElement;
    const form = this.container.querySelector('.cw-form') as HTMLFormElement;

    if (!nickname) { alert(this.i18n.t('nicknameRequired')); return; }
    if (!email || !email.includes('@')) { alert(this.i18n.t('emailRequired')); return; }
    if (!content) { alert(this.i18n.t('contentRequired')); return; }

    submitBtn.textContent = this.i18n.t('submitting');
    submitBtn.setAttribute('disabled', 'true');

    try {
      await this.api.createComment({
        nickname,
        email,
        website: website || undefined,
        content: content.replace(/\n/g, '\n\n'),
      });

      // 清空表单
      const textarea = this.container.querySelector('.cw-textarea') as HTMLTextAreaElement;
      textarea.value = '';
      this.clearDraft();

      this.currentPage = 1;
      await this.loadComments();

      // 显示待审核提示
      const pendingMsg = this.container.querySelector('.cw-pending-msg') as HTMLElement;
      pendingMsg.style.display = 'block';
      setTimeout(() => { pendingMsg.style.display = 'none'; }, 5000);
    } catch {
      alert(this.i18n.t('errorSubmit'));
    } finally {
      submitBtn.textContent = this.i18n.t('submit');
      submitBtn.removeAttribute('disabled');
    }
  }

  private async handleUpload(input: HTMLInputElement) {
    const file = input.files?.[0];
    if (!file) return;

    try {
      const result = await this.api.uploadImage(file);
      const textarea = this.container.querySelector('.cw-textarea') as HTMLTextAreaElement;
      textarea.value += `\n![image](${result.url})\n`;
    } catch {
      alert('图片上传失败');
    } finally {
      input.value = '';
    }
  }

  private formatTime(dateStr: string): string {
    const date = new Date(dateStr + 'Z');
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diff < 60) return this.i18n.t('justNow');
    if (diff < 3600) return `${Math.floor(diff / 60)} ${this.i18n.t('minutesAgo')}`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} ${this.i18n.t('hoursAgo')}`;
    if (diff < 2592000) return `${Math.floor(diff / 86400)} ${this.i18n.t('daysAgo')}`;
    return date.toLocaleDateString();
  }

  private getAvatarHtml(name: string, isAdmin: boolean): string {
    const initial = name.charAt(0).toUpperCase();
    const hue = name.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 360;
    const bg = `hsl(${hue}, 60%, ${isAdmin ? '45%' : '65%'})`;
    return `<div class="cw-avatar" style="background:${bg}">${initial}</div>`;
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  private saveDraft() {
    const key = `cw_draft_${this.config.siteId}_${this.config.pageUrl}`;
    const nickname = (this.container.querySelector('.cw-nickname') as HTMLInputElement)?.value || '';
    const email = (this.container.querySelector('.cw-email') as HTMLInputElement)?.value || '';
    const website = (this.container.querySelector('.cw-website') as HTMLInputElement)?.value || '';
    const content = (this.container.querySelector('.cw-textarea') as HTMLTextAreaElement)?.value || '';

    try {
      localStorage.setItem(key, JSON.stringify({ nickname, email, website, content }));
    } catch {}
  }

  private loadDraft() {
    const key = `cw_draft_${this.config.siteId}_${this.config.pageUrl}`;
    try {
      const draft = localStorage.getItem(key);
      if (draft) {
        const data = JSON.parse(draft);
        const nickname = this.container.querySelector('.cw-nickname') as HTMLInputElement;
        const email = this.container.querySelector('.cw-email') as HTMLInputElement;
        const website = this.container.querySelector('.cw-website') as HTMLInputElement;
        const content = this.container.querySelector('.cw-textarea') as HTMLTextAreaElement;
        if (nickname) nickname.value = data.nickname || '';
        if (email) email.value = data.email || '';
        if (website) website.value = data.website || '';
        if (content) content.value = data.content || '';
      }
    } catch {}
  }

  private clearDraft() {
    const key = `cw_draft_${this.config.siteId}_${this.config.pageUrl}`;
    try { localStorage.removeItem(key); } catch {}
  }
}
