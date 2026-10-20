(()=>{'use strict';
// Keys follow the on-screen numbers. Navigation and toggles are instant.
addEventListener('keydown',e=>{
  if(e.defaultPrevented||e.metaKey||e.ctrlKey||e.altKey||e.shiftKey||e.target.closest('input,textarea,select,[contenteditable]'))return;
  const el=document.querySelector('[data-key="'+e.key+'"]');if(!el)return;
  e.preventDefault();
  if(el.tagName==='A'){location.assign(el.href);return}
  el.classList.add('instant');el.open=!el.open;if(el.open)el.querySelector('summary').focus();
  requestAnimationFrame(()=>requestAnimationFrame(()=>el.classList.remove('instant')));
});
})();
