(()=>{'use strict';
// Same rule as the original footer: Stale when the oldest known observation is
// over the threshold, Unknown when any date is missing or in the future.
const parse=s=>{if(/^\d{4}-\d{2}-\d{2}$/.test(s))s+='T00:00:00Z';s=s.replace(/(\.\d{3})\d+(?=Z|[+-])/,'$1');const n=Date.parse(s);return Number.isFinite(n)?n:null};
const span=ms=>{const m=Math.floor(Math.max(0,ms)/6e4);if(m<1)return'now';if(m<60)return m+'m';const h=Math.floor(m/60);return h<48?h+'h':Math.floor(h/24)+'d'};
const text=(el,sel,t)=>{const n=el.querySelector(sel);if(n&&n.textContent!==t)n.textContent=t};
// Dates arrive as [label, evidence-source count] pairs, labels exactly as supplied.
const facts=el=>{const now=Date.now(),rows=JSON.parse(el.dataset.dates).map(([raw,n])=>({raw,n,time:parse(raw)}));
  const known=rows.filter(d=>d.time!==null&&d.time<=now).sort((a,b)=>a.time-b.time);
  const records=rows.reduce((t,d)=>t+d.n,0);
  return{now,known,records,unknown:records-known.reduce((t,d)=>t+d.n,0),hours:+el.dataset.hours}};
function refresh(){
  for(const el of document.querySelectorAll('[data-fresh]')){
    const{now,known,unknown,hours}=facts(el),built=parse(el.dataset.built);
    const status=known.length&&now-known[0].time>=hours*36e5?'Stale':unknown||!known.length?'Unknown':'Fresh';
    el.dataset.status=status;
    text(el,'.w',status);text(el,'.built',built===null?'?':span(now-built));
    text(el,'.data',known.length?span(now-known[0].time)+(unknown?'+':''):'?');
    const tile=el.closest('.map');
    if(tile){tile.dataset.status=status;const fill=tile.querySelector('.age>i');
      if(fill)fill.style.transform='scaleX('+(status==='Fresh'?Math.max(0,1-(now-known[0].time)/(hours*36e5)):0).toFixed(3)+')'}
  }
}
// Original date labels, as supplied, behind Basis.
function dates(){
  for(const box of document.querySelectorAll('.basis .dates')){
    box.replaceChildren();
    for(const el of document.querySelectorAll('[data-fresh][data-name]')){
      const{known,records,unknown}=facts(el),rows=[['Built',el.dataset.built],['Oldest',known.length?known[0].raw:'None'],['Newest',known.length?known[known.length-1].raw:'None'],['Evidence',String(records)]];
      if(unknown)rows.push(['Unknown',String(unknown)]);
      const h=document.createElement('h4'),dl=document.createElement('dl');h.textContent=el.dataset.name;
      for(const[k,v]of rows){const row=document.createElement('div'),dt=document.createElement('dt'),dd=document.createElement('dd');dt.textContent=k;dd.textContent=v;row.append(dt,dd);dl.append(row)}
      box.append(h,dl);
    }
  }
}
refresh();dates();setInterval(refresh,6e4);document.addEventListener('visibilitychange',refresh);addEventListener('pageshow',refresh);
const close=()=>{for(const b of document.querySelectorAll('.basis[open]')){b.open=false;return b}};
document.addEventListener('keydown',e=>{if(e.key==='Escape'){const b=close();if(b)b.querySelector('summary').focus()}});
document.addEventListener('click',e=>{if(!e.target.closest('.basis'))close()});
})();
