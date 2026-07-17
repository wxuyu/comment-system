// 评论组件主入口

import { CommentWidget } from './widget';
import './style.css';

// 挂载到全局
(window as any).CommentWidget = CommentWidget;

// 自动初始化
function autoInit() {
  const containers = document.querySelectorAll('[data-comment-widget]');
  containers.forEach((container) => {
    const el = container as HTMLElement;
    const apiBase = el.dataset.commentWidget || '/api/v1';
    const siteId = parseInt(el.dataset.siteId || '1');
    const pageUrl = el.dataset.pageUrl || window.location.pathname;
    const pageTitle = el.dataset.pageTitle || document.title;
    const darkMode = el.dataset.darkMode === 'true';
    const locale = el.dataset.locale || 'zh-CN';

    const widget = new CommentWidget(container, {
      apiBase,
      siteId,
      pageUrl,
      pageTitle,
      darkMode,
      locale,
    });
    widget.init();
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', autoInit);
} else {
  autoInit();
}
