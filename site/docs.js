(() => {
  // Deep links must reveal their destination even inside progressive disclosure.
  // Browser-native links, details and text remain usable without this helper.
  function reveal() {
    let id; try { id = decodeURIComponent(location.hash.slice(1)); } catch (_) { return; }
    const target = document.getElementById(id);
    if (!target) return;
    for (let element = target; element; element = element.parentElement) {
      if (element instanceof HTMLDetailsElement) element.open = true;
    }
    target.scrollIntoView({ block: 'start' });
  }
  const contents = document.querySelector('.doc-toc details');
  if (contents && matchMedia('(max-width:1180px)').matches) contents.open = false;
  window.addEventListener('hashchange', reveal);
  if (location.hash) reveal();
})();
