import{j as e}from"./query-BhuALcwA.js";import{r as i}from"./vendor-yaVYLZzK.js";import{c as n,U as c,D as m,g as d,B as l,h as x,i as t}from"./index-BDKGGG7P.js";/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const p=n("Monitor",[["rect",{width:"20",height:"14",x:"2",y:"3",rx:"2",key:"48i651"}],["line",{x1:"8",x2:"16",y1:"21",y2:"21",key:"1svkeh"}],["line",{x1:"12",x2:"12",y1:"17",y2:"21",key:"vw1qmm"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const r=n("Moon",[["path",{d:"M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z",key:"a7tn18"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const h=n("Sun",[["circle",{cx:"12",cy:"12",r:"4",key:"4exip2"}],["path",{d:"M12 2v2",key:"tus03m"}],["path",{d:"M12 20v2",key:"1lh1kg"}],["path",{d:"m4.93 4.93 1.41 1.41",key:"149t6j"}],["path",{d:"m17.66 17.66 1.41 1.41",key:"ptbguv"}],["path",{d:"M2 12h2",key:"1t8f8n"}],["path",{d:"M20 12h2",key:"1q8mjw"}],["path",{d:"m6.34 17.66-1.41 1.41",key:"1m8zz5"}],["path",{d:"m19.07 4.93-1.41 1.41",key:"1shlcs"}]]);function u(){const o=i.useContext(c);if(!o)throw new Error("ThemeToggle must be used within ThemeProvider");const{setTheme:s,resolvedTheme:a}=o;return e.jsxs(m,{children:[e.jsx(d,{asChild:!0,children:e.jsxs(l,{variant:"ghost",size:"icon",children:[a==="dark"?e.jsx(r,{className:"h-5 w-5"}):e.jsx(h,{className:"h-5 w-5"}),e.jsx("span",{className:"sr-only",children:"Toggle theme"})]})}),e.jsxs(x,{align:"end",children:[e.jsxs(t,{onClick:()=>s("light"),children:[e.jsx(h,{className:"mr-2 h-4 w-4"}),"Light"]}),e.jsxs(t,{onClick:()=>s("dark"),children:[e.jsx(r,{className:"mr-2 h-4 w-4"}),"Dark"]}),e.jsxs(t,{onClick:()=>s("system"),children:[e.jsx(p,{className:"mr-2 h-4 w-4"}),"System"]})]})]})}export{u as T};
