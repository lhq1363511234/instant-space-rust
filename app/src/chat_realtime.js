(() => {
  const controllers = new Map();
  let hydrated = false;

  function localeIsZh() {
    return (document.documentElement.lang || '').toLowerCase().startsWith('zh');
  }

  function text(zh, en) {
    return localeIsZh() ? zh : en;
  }

  function socketUrl(spaceId) {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const prefix = location.pathname.startsWith('/inspace/') ? '/inspace' : '';
    return `${protocol}//${location.host}${prefix}/ws/spaces/${encodeURIComponent(spaceId)}`;
  }

  function appendMessage(list, message) {
    if (!list || !message || !message.id) return;
    if (list.querySelector(`[data-message-id="${CSS.escape(String(message.id))}"]`)) return;

    const empty = list.querySelector('.empty-state');
    if (empty) empty.remove();

    const article = document.createElement('article');
    article.className = 'chat-message';
    article.dataset.messageId = String(message.id);

    const sender = document.createElement('strong');
    sender.textContent = message.sender || text('访客', 'Guest');
    const body = document.createElement('p');
    body.textContent = message.body || '';
    const time = document.createElement('time');
    let parsed;
    if (Array.isArray(message.created_at) && message.created_at.length >= 5) {
      const [year, ordinalDay, hour, minute, second = 0] = message.created_at;
      parsed = new Date(Date.UTC(year, 0, ordinalDay, hour, minute, second));
    } else {
      parsed = new Date(message.created_at);
    }
    time.textContent = Number.isNaN(parsed.getTime())
      ? ''
      : parsed.toLocaleString([], {
          year: 'numeric', month: '2-digit', day: '2-digit',
          hour: '2-digit', minute: '2-digit',
        });

    article.append(sender, body, time);
    list.appendChild(article);
    list.scrollTop = list.scrollHeight;
  }

  function mount(root) {
    if (!root || root.dataset.realtimeMounted === 'true') return;
    const spaceId = root.dataset.spaceId;
    const shell = root.closest('.chat-shell');
    const form = shell?.querySelector('[data-chat-form="true"]');
    const input = form?.querySelector('[data-chat-input="true"]');
    const status = shell?.querySelector('[data-realtime-status]');
    const online = shell?.querySelector('[data-realtime-online]');
    const reverify = shell?.querySelector('[data-private-reverify]');
    if (!spaceId || !form || !input) return;

    root.dataset.realtimeMounted = 'true';
    let ws = null;
    let closedForAccess = false;
    let reconnectTimer = null;
    let attempts = 0;

    const setStatus = (state, label) => {
      if (!status) return;
      status.dataset.realtimeStatus = state;
      status.textContent = label;
    };

    const showError = (message) => {
      let error = form.querySelector('.chat-realtime-error');
      if (!error) {
        error = document.createElement('p');
        error.className = 'error chat-realtime-error';
        form.appendChild(error);
      }
      error.textContent = message;
    };

    const clearError = () => {
      form.querySelector('.chat-realtime-error')?.remove();
    };

    const connect = () => {
      if (!document.body.contains(root) || closedForAccess) return;
      setStatus('connecting', text('正在连接实时房间…', 'Connecting to realtime room…'));
      ws = new WebSocket(socketUrl(spaceId));

      ws.addEventListener('open', () => {
        attempts = 0;
        clearError();
        setStatus('connected', text('实时连接已建立', 'Realtime connected'));
      });

      ws.addEventListener('message', (event) => {
        let payload;
        try { payload = JSON.parse(event.data); } catch (_) { return; }
        if (payload.type === 'history') {
          (payload.messages || []).forEach((message) => appendMessage(root, message));
          root.scrollTop = root.scrollHeight;
        } else if (payload.type === 'message') {
          appendMessage(root, payload.message);
        } else if (payload.type === 'presence') {
          const count = Number(payload.online_count || 0);
          if (online) {
            online.dataset.realtimeOnline = String(count);
            online.textContent = localeIsZh() ? `${count} 人在线` : `${count} online`;
          }
        } else if (payload.type === 'error') {
          showError(payload.message || text('实时消息发送失败', 'Realtime message failed'));
          if (payload.code === 'password_changed' || payload.code === 'access_expired') {
            closedForAccess = true;
            setStatus('access-required', text('需要重新验证密码', 'Password verification required'));
            if (reverify) reverify.style.display = '';
          }
        }
      });

      ws.addEventListener('close', () => {
        if (!document.body.contains(root) || closedForAccess) return;
        attempts += 1;
        const delay = Math.min(1000 * (2 ** Math.min(attempts, 4)), 15000);
        setStatus('reconnecting', text('连接中断，正在重连…', 'Disconnected, reconnecting…'));
        reconnectTimer = window.setTimeout(connect, delay);
      });

      ws.addEventListener('error', () => {
        setStatus('error', text('实时连接暂时不可用', 'Realtime connection unavailable'));
      });
    };

    const submit = (event) => {
      const body = String(input.value || '').trim();
      if (!body) return;

      // WebSocket is the fast path. If it is still connecting or temporarily
      // unavailable, let the hydrated Leptos submit handler use the regular
      // server function instead of blocking the user.
      if (!ws || ws.readyState !== WebSocket.OPEN) {
        clearError();
        return;
      }

      event.preventDefault();
      event.stopImmediatePropagation();
      ws.send(JSON.stringify({ type: 'message', body }));
      input.value = '';
      input.dispatchEvent(new Event('input', { bubbles: true }));
      clearError();
    };

    form.addEventListener('submit', submit, true);
    connect();

    controllers.set(root, {
      close() {
        form.removeEventListener('submit', submit, true);
        if (reconnectTimer) window.clearTimeout(reconnectTimer);
        if (ws && ws.readyState < WebSocket.CLOSING) ws.close(1000, 'route changed');
      },
    });
  }

  function scan() {
    if (!hydrated) return;
    for (const [root, controller] of controllers) {
      if (!document.body.contains(root)) {
        controller.close();
        controllers.delete(root);
      }
    }
    document.querySelectorAll('[data-realtime-messages="true"]').forEach(mount);
  }

  function boot() {
    if (hydrated) return;
    hydrated = true;
    scan();
    new MutationObserver(scan).observe(document.body, { childList: true, subtree: true });
  }

  if (window.__instantSpaceHydrated) boot();
  else window.addEventListener('instant-space-hydrated', boot, { once: true });
  // Non-hydrated emergency fallback. Like map_boot, never mutate early.
  window.setTimeout(() => {
    if (!window.__instantSpaceHydrated) return;
    boot();
  }, 5000);
})();
