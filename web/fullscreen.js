(() => {
  'use strict';
  const button = document.querySelector('[data-fullscreen-toggle]');
  if (!button) return;
  // The host expands its existing iframe, preserving the embedded map's state.
  // The iframe does not add a second, competing fullscreen control.
  if (window.self !== window.top) {
    button.hidden = true;
    document.addEventListener('keydown', event => {
      try {
        if (event.key === 'Escape' && parent.document.querySelector('[data-fullscreen-active]')) {
          event.preventDefault(); event.stopImmediatePropagation();
          parent.postMessage({ type: 'topo:exit-fullscreen' }, location.origin);
        }
      } catch (_) { /* An unrelated cross-origin embed owns its own controls. */ }
    }, true);
    return;
  }
  const surface = document.querySelector('[data-fullscreen-surface]') || document.documentElement;
  const frame = surface.querySelector('iframe');
  const outside = [];
  let active = false, pending = false, native = false;
  let oldOverflow = '', returnFocus = null;
  const currentFullscreen = () => document.fullscreenElement || document.webkitFullscreenElement;
  function resize() {
    window.dispatchEvent(new Event('resize'));
    try { frame?.contentWindow.dispatchEvent(new Event('resize')); } catch (_) {}
  }
  function label() {
    const text = active ? 'Exit fullscreen' : 'Fullscreen';
    button.setAttribute('aria-label', text);
    button.setAttribute('aria-pressed', String(active));
    button.title = active ? 'Exit fullscreen (Escape)' : 'Fullscreen';
    const visible = button.querySelector('[data-fullscreen-label]');
    if (visible) visible.textContent = text;
  }
  function restore() {
    active = false; native = false;
    surface.removeAttribute('data-fullscreen-active');
    document.body.style.overflow = oldOverflow;
    for (const [element, inert] of outside.splice(0)) element.inert = inert;
    label(); resize();
    if (returnFocus?.isConnected) returnFocus.focus({ preventScroll: true });
  }
  async function exit() {
    if (!active || pending) return;
    if (currentFullscreen() === surface) {
      try { await (document.exitFullscreen || document.webkitExitFullscreen).call(document); }
      catch (_) { /* Retain a working page-level exit if the browser denies it. */ }
      if (currentFullscreen() === surface) return;
    }
    if (active) restore();
  }
  async function enter() {
    if (active || pending) return;
    active = true; pending = true;
    returnFocus = button; oldOverflow = document.body.style.overflow;
    surface.setAttribute('data-fullscreen-active', '');
    document.body.style.overflow = 'hidden';
    // Keep keyboard focus in the expanded map without changing hidden/disabled
    // state of the rest of the marketing page. Restore prior inert values.
    for (let child = surface; child.parentElement; child = child.parentElement) {
      for (const sibling of child.parentElement.children) {
        if (sibling !== child && sibling instanceof HTMLElement && !['SCRIPT', 'STYLE', 'LINK'].includes(sibling.tagName)) {
          outside.push([sibling, sibling.inert]); sibling.inert = true;
        }
      }
    }
    label(); resize();
    const request = surface.requestFullscreen || surface.webkitRequestFullscreen;
    try {
      if (request && (document.fullscreenEnabled || document.webkitFullscreenEnabled)) {
        await request.call(surface);
        native = currentFullscreen() === surface;
      }
    } catch (_) { /* Unsupported/denied native fullscreen retains the in-page expansion. */ }
    finally { pending = false; button.focus({ preventScroll: true }); resize(); }
  }
  button.addEventListener('click', () => active ? exit() : enter());
  for (const event of ['fullscreenchange', 'webkitfullscreenchange']) {
    document.addEventListener(event, () => {
      if (currentFullscreen() === surface) native = true;
      else if (native && active) restore();
    });
  }
  document.addEventListener('keydown', event => {
    if (!active) return;
    if (event.key === 'Escape') {
      event.preventDefault(); event.stopImmediatePropagation(); exit();
    } else if (event.key === 'Tab' && surface !== document.documentElement) {
      const controls = [...surface.querySelectorAll('button, a[href], input, select, [tabindex="0"], iframe')]
        .filter(element => !element.hidden && !element.disabled && element.getClientRects().length);
      const first = controls[0], last = controls.at(-1);
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
    }
  }, true);
  window.addEventListener('message', event => {
    if (active && frame && event.source === frame.contentWindow && event.origin === location.origin && event.data?.type === 'topo:exit-fullscreen') exit();
  });
  label();
})();
