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
    const prefix = (location.pathname === '/inspace' || location.pathname.startsWith('/inspace/')) ? '/inspace' : '';
    return `${protocol}//${location.host}${prefix}/ws/spaces/${encodeURIComponent(spaceId)}`;
  }

  function monogram(name) {
    const trimmed = String(name || '').trim();
    return trimmed ? Array.from(trimmed)[0].toUpperCase() : '?';
  }

  // Only auto-scroll when the reader is already following the live edge, so
  // scrolling back through history is not yanked forward by every new message.
  function isPinnedToBottom(list) {
    return list.scrollHeight - list.scrollTop - list.clientHeight < 90;
  }

  function appendMessage(list, message) {
    if (!list || !message || !message.id) return;
    if (list.querySelector(`[data-message-id="${CSS.escape(String(message.id))}"]`)) return;

    const pinned = isPinnedToBottom(list);
    list.querySelector('.chat-empty')?.remove();
    list.querySelector('.empty-state')?.remove();

    const article = document.createElement('article');
    const kindClass = {
      system: 'chat-message--system',
      help: 'chat-message--help',
      help_resolved: 'chat-message--help-resolved',
    }[message.kind] || '';
    article.className = ['chat-message', kindClass].filter(Boolean).join(' ');
    article.dataset.messageId = String(message.id);

    const senderName = message.sender || text('访客', 'Guest');
    const avatar = document.createElement('span');
    avatar.className = 'chat-avatar';
    avatar.setAttribute('aria-hidden', 'true');
    avatar.textContent = monogram(senderName);

    const bodyWrap = document.createElement('div');
    bodyWrap.className = 'chat-message-body';

    const meta = document.createElement('p');
    meta.className = 'chat-message-meta';
    const sender = document.createElement('strong');
    sender.textContent = senderName;

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
      : `${String(parsed.getHours()).padStart(2, '0')}:${String(parsed.getMinutes()).padStart(2, '0')}`;
    meta.append(sender, time);

    const body = document.createElement('p');
    body.className = 'chat-message-text';
    body.textContent = message.body || '';

    bodyWrap.append(meta, body);
    article.append(avatar, bodyWrap);
    list.appendChild(article);
    if (pinned) list.scrollTop = list.scrollHeight;
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
      setStatus('connecting', text('连接中', 'Connecting'));
      ws = new WebSocket(socketUrl(spaceId));

      ws.addEventListener('open', () => {
        attempts = 0;
        clearError();
        setStatus('connected', text('实时在线', 'Live'));
      });

      ws.addEventListener('message', (event) => {
        let payload;
        try { payload = JSON.parse(event.data); } catch (_) { return; }
        if (payload.type === 'history') {
          (payload.messages || []).forEach((message) => appendMessage(root, message));
          root.scrollTop = root.scrollHeight;
          root.dataset.chatReady = 'true';
        } else if (payload.type === 'message') {
          appendMessage(root, payload.message);
        } else if (payload.type === 'presence') {
          const count = Number(payload.online_count || 0);
          if (online) {
            online.dataset.realtimeOnline = String(count);
            online.textContent = localeIsZh() ? `${count} 人在场` : `${count} here`;
          }
        } else if (payload.type === 'error') {
          showError(payload.message || text('实时消息发送失败', 'Realtime message failed'));
          if (payload.code === 'password_changed' || payload.code === 'access_expired') {
            closedForAccess = true;
            setStatus('access-required', text('需重新验证', 'Verify again'));
            if (reverify) reverify.style.display = '';
          }
        }
      });

      ws.addEventListener('close', () => {
        if (!document.body.contains(root) || closedForAccess) return;
        attempts += 1;
        const delay = Math.min(1000 * (2 ** Math.min(attempts, 4)), 15000);
        setStatus('reconnecting', text('重新连接中', 'Reconnecting'));
        reconnectTimer = window.setTimeout(connect, delay);
      });

      ws.addEventListener('error', () => {
        setStatus('error', text('连接不可用', 'Offline'));
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
      root.scrollTop = root.scrollHeight;
      clearError();
    };

    // Enter sends, Shift+Enter breaks the line; the composer grows with content.
    const autogrow = () => {
      if (input.dataset.chatAutogrow !== 'true') return;
      input.style.height = 'auto';
      input.style.height = `${Math.min(input.scrollHeight, 148)}px`;
    };
    const onKeydown = (event) => {
      if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
        event.preventDefault();
        if (String(input.value || '').trim()) {
          form.requestSubmit ? form.requestSubmit() : form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }));
        }
      }
    };

    input.addEventListener('input', autogrow);
    input.addEventListener('keydown', onKeydown);
    form.addEventListener('submit', submit, true);
    connect();
    autogrow();

    controllers.set(root, {
      close() {
        form.removeEventListener('submit', submit, true);
        input.removeEventListener('input', autogrow);
        input.removeEventListener('keydown', onKeydown);
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
