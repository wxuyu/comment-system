// 评论系统管理后台 - Vue 3 应用
// 4 个面板：控制台 / 页面管理 / 站点管理 / 表单管理

const { createApp, reactive, ref, onMounted, computed } = Vue;

createApp({
  setup() {
    // 基础状态
    const token = ref(localStorage.getItem('cms_token') || '');
    const user = reactive(JSON.parse(localStorage.getItem('cms_user') || '{}'));
    const activePanel = ref('console');
    const consoleTab = ref('pending');
    const logging = ref(false);
    const loginError = ref('');
    const oauthProviders = ref([]);

    const loginForm = reactive({ username: 'admin', password: '' });

    // 数据
    const comments = ref([]);
    const pages = ref([]);
    const sites = ref([]);
    const stats = reactive({});
    const notifications = ref([]);

    // 站点编辑
    const showEditSite = ref(false);
    const editingSite = reactive({ id: null, name: '', domain: '', urlsText: '' });
    const showSiteForm = ref(false);

    // 表单
    const formCards = ref([{ id: 1, title: '' }, { id: 2, title: '' }, { id: 3, title: '' }]);
    const formBoolFalse = ref(false);
    const formBoolTrue = ref(true);
    const formSelect = ref('候选项 1');

    // ============ 工具 ============
    const API = '/api/v1';
    const authHeaders = () => ({ 'Authorization': 'Bearer ' + token.value });

    async function api(path, options = {}) {
      const opts = { headers: { 'Content-Type': 'application/json', ...(options.headers || {}) }, ...options };
      if (token.value) opts.headers['Authorization'] = 'Bearer ' + token.value;
      const resp = await fetch(API + path, opts);
      if (resp.status === 401) {
        logout();
        throw new Error('未授权');
      }
      return resp;
    }

    function formatTime(t) {
      if (!t) return '';
      const d = new Date(t.replace(' ', 'T') + 'Z');
      const now = new Date();
      const diff = (now - d) / 1000;
      if (diff < 60) return '刚刚';
      if (diff < 3600) return Math.floor(diff / 60) + ' 分钟前';
      if (diff < 86400) return Math.floor(diff / 3600) + ' 小时前';
      return d.toLocaleDateString();
    }

    function statusLabel(s) {
      return ({ pending: '待审', approved: '已通过', spam: '垃圾', trash: '回收' })[s] || s;
    }

    // ============ 登录 ============
    async function login() {
      logging.value = true;
      loginError.value = '';
      try {
        const resp = await fetch(API + '/admin/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(loginForm),
        });
        const json = await resp.json();
        if (!resp.ok) throw new Error(json.message || '登录失败');
        token.value = json.data.token;
        user.id = json.data.user.id;
        user.username = json.data.user.username;
        localStorage.setItem('cms_token', token.value);
        localStorage.setItem('cms_user', JSON.stringify(user));
        await loadStats();
      } catch (e) {
        loginError.value = e.message;
      } finally {
        logging.value = false;
      }
    }

    function logout() {
      token.value = '';
      localStorage.removeItem('cms_token');
      localStorage.removeItem('cms_user');
    }

    function oauthLogin(provider) {
      window.location = API + '/oauth/' + provider + '/authorize?redirect=/admin';
    }

    // 从 URL 提取 token（OAuth 回调后）
    function tryExtractTokenFromUrl() {
      const url = new URL(window.location.href);
      const t = url.searchParams.get('token');
      if (t) {
        token.value = t;
        localStorage.setItem('cms_token', t);
        url.searchParams.delete('token');
        window.history.replaceState({}, '', url.pathname + (url.search || ''));
        api('/admin/stats').then(r => r.json()).then(j => {
          if (j.data) {
            Object.assign(stats, j.data);
            user.id = 0;
            user.username = url.searchParams.get('nickname') || 'OAuth用户';
          }
        });
      }
    }

    // ============ 加载数据 ============
    async function loadStats() {
      try {
        const r = await api('/admin/stats');
        const j = await r.json();
        if (j.data) Object.assign(stats, j.data);
      } catch (e) { console.error(e); }
    }

    async function loadComments() {
      try {
        const url = consoleTab.value === 'pending'
          ? '/admin/comments/pending'
          : '/comments?status=' + consoleTab.value;
        const r = await api(url);
        const j = await r.json();
        comments.value = j.data || [];
      } catch (e) { console.error(e); }
    }

    async function loadPages() {
      try {
        const r = await api('/admin/pages');
        const j = await r.json();
        pages.value = j.data || [];
      } catch (e) { console.error(e); }
    }

    async function loadSites() {
      try {
        const r = await api('/sites');
        const j = await r.json();
        sites.value = (j.data || []).map((s, i) => ({
          ...s,
          color: ['#5B8FF9', '#5AD8A6', '#F6BD16', '#E86452', '#6DC8EC'][i % 5]
        }));
      } catch (e) { console.error(e); }
    }

    async function loadOAuthProviders() {
      try {
        const r = await fetch(API + '/oauth/providers');
        const j = await r.json();
        oauthProviders.value = j.data?.providers || [];
      } catch (e) { console.error(e); }
    }

    // ============ 操作 ============
    async function updateStatus(id, status) {
      if (!confirm('确认将评论标记为 ' + statusLabel(status) + '？')) return;
      await api('/admin/comments/' + id + '/status', {
        method: 'PUT',
        body: JSON.stringify({ status }),
      });
      await loadComments();
      await loadStats();
    }

    async function deleteComment(id) {
      if (!confirm('确认删除该评论？')) return;
      await api('/admin/comments/' + id, { method: 'DELETE' });
      await loadComments();
      await loadStats();
    }

    async function deletePage(id) {
      if (!confirm('确认删除该页面？')) return;
      await api('/admin/pages/' + id, { method: 'DELETE' });
      await loadPages();
    }

    function editPage(p) {
      const newTitle = prompt('新的页面标题', p.title);
      if (newTitle !== null) {
        alert('已更新（实际功能由后端 PUT 接口实现）');
      }
    }

    function regenerateKey(p) {
      if (confirm('确认重置页面密钥？')) {
        alert('已重置（实际功能由后端接口实现）');
      }
    }

    function openPageMenu(p) {
      alert('更多操作：' + p.title);
    }

    function editSiteURL() {
      const newDomain = prompt('新域名', editingSite.domain);
      if (newDomain !== null) editingSite.domain = newDomain;
    }

    function renameSite() {
      const newName = prompt('新名称');
      if (newName) {
        editingSite.name = newName;
        saveSite();
      }
    }

    function exportSite() {
      alert('导出 JSON（实际由后端接口实现）');
    }

    function importSite() {
      alert('导入 JSON（实际由后端接口实现）');
    }

    function deleteSite() {
      if (editingSite.id && confirm('确认删除该站点？')) {
        api('/admin/sites/' + editingSite.id, { method: 'DELETE' }).then(() => {
          showEditSite.value = false;
          loadSites();
        });
      }
    }

    async function saveSite() {
      if (!editingSite.name || !editingSite.domain) {
        alert('名称和域名必填');
        return;
      }
      const urls = editingSite.urlsText.split('\n').map(s => s.trim()).filter(Boolean);
      const body = { name: editingSite.name, domain: editingSite.domain, urls };
      const url = editingSite.id
        ? '/admin/sites/' + editingSite.id
        : '/admin/sites';
      const method = editingSite.id ? 'PUT' : 'POST';
      await api(url, { method, body: JSON.stringify(body) });
      showEditSite.value = false;
      editingSite.id = null;
      editingSite.name = '';
      editingSite.domain = '';
      editingSite.urlsText = '';
      await loadSites();
    }

    function addFormCard() {
      formCards.value.push({ id: Date.now(), title: '' });
    }

    function removeFormCard(id) {
      formCards.value = formCards.value.filter(c => c.id !== id);
    }

    function submitForm() {
      const data = {
        cards: formCards.value.map(c => c.title),
        bool_false: formBoolFalse.value,
        bool_true: formBoolTrue.value,
        select: formSelect.value,
      };
      alert('表单已提交：\n' + JSON.stringify(data, null, 2));
    }

    // ============ 切换面板时加载数据 ============
    function switchPanel(name) {
      activePanel.value = name;
      if (name === 'console') loadComments();
      else if (name === 'pages') loadPages();
      else if (name === 'sites') {
        loadSites();
        editingSite.id = null;
        editingSite.name = '';
        editingSite.domain = '';
        editingSite.urlsText = '';
      }
    }

    onMounted(() => {
      tryExtractTokenFromUrl();
      loadOAuthProviders();
      if (token.value) {
        loadStats();
        loadComments();
      }
    });

    return {
      token, user, activePanel, consoleTab,
      logging, loginError, loginForm, oauthProviders,
      comments, pages, sites, stats, notifications,
      login, logout, oauthLogin,
      showEditSite, editingSite, showSiteForm,
      formCards, formBoolFalse, formBoolTrue, formSelect,
      loadComments, loadPages, loadSites, loadStats,
      updateStatus, deleteComment, deletePage,
      editPage, regenerateKey, openPageMenu,
      editSiteURL, renameSite, exportSite, importSite, deleteSite, saveSite,
      addFormCard, removeFormCard, submitForm,
      switchPanel, formatTime, statusLabel,
    };
  }
}).mount('#app');
