(() => {
  let activeDialog = null;
  let returnFocus = null;

  function isModifiedClick(event) {
    return event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey;
  }

  function focusables(dialog) {
    return Array.from(dialog.querySelectorAll(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    )).filter((element) => {
      const rect = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      return rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
    });
  }

  document.addEventListener('click', (event) => {
    if (isModifiedClick(event) || !(event.target instanceof Element)) return;

    const worldAction = event.target.closest('[data-world-sheet] a.world-primary-action');
    if (worldAction) {
      const url = new URL(worldAction.href, location.href);
      const match = url.pathname.match(/\/(?:inspace\/)?spaces\/([^/]+)(?:\/chat)?$/);
      if (match) {
        const runtime = worldAction.closest('[data-world-runtime]');
        const panel = url.pathname.endsWith('/chat')
          ? 'discussion'
          : ({
              '#space-intro': 'intro',
              '#space-host': 'host',
              '#space-traces': 'story',
              '#space-capsules': 'capsules',
              '#space-guides': 'guides'
            }[url.hash] || 'wall');
        const bridge = runtime?.querySelector(`.world-space-modal-bridge--${panel}`);
        if (bridge) {
          event.preventDefault();
          event.stopPropagation();
          returnFocus = worldAction;
          runtime?.querySelector('[data-world-sheet-close]')?.click();
          bridge.click();
          return;
        }
      }
    }

    const trigger = event.target.closest('a[href*="/inspace/spaces/"]');
    if (trigger && !trigger.classList.contains('world-space-modal-bridge') && !trigger.closest('.space-experience-dialog')) {
      returnFocus = trigger;
    }
  }, true);

  document.addEventListener('keydown', (event) => {
    const dialog = document.querySelector('.space-experience-dialog');
    if (!dialog) return;

    if (event.key === 'Escape') {
      event.preventDefault();
      dialog.querySelector('.space-experience-close')?.click();
      return;
    }

    if (event.key !== 'Tab') return;
    const items = focusables(dialog);
    if (!items.length) return;
    const first = items[0];
    const last = items[items.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  });

  function scan() {
    const dialog = document.querySelector('.space-experience-dialog');
    if (dialog && dialog !== activeDialog) {
      activeDialog = dialog;
      requestAnimationFrame(() => {
        const target = dialog.querySelector('.space-experience-close') || focusables(dialog)[0];
        target?.focus({ preventScroll: true });
      });
    } else if (!dialog && activeDialog) {
      activeDialog = null;
      if (returnFocus && document.contains(returnFocus)) {
        returnFocus.focus({ preventScroll: true });
      }
      returnFocus = null;
    }
  }

  new MutationObserver(scan).observe(document.documentElement, { childList: true, subtree: true });
  scan();
})();
