import{c as o,T as u,B as c,I as l,S as j}from"./index-BDKGGG7P.js";import{j as e}from"./query-BhuALcwA.js";import{D as m,f as p,g as v,h as T,i as g,j as y}from"./Dialog-CJKvLMIU.js";import{B as i}from"./Badge-CUPLjVMs.js";/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const A=o("RotateCw",[["path",{d:"M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8",key:"1p45f6"}],["path",{d:"M21 3v5h-5",key:"1q7to0"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const w=o("Trash2",[["path",{d:"M3 6h18",key:"d0wm0j"}],["path",{d:"M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6",key:"4alrt4"}],["path",{d:"M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2",key:"v07s0e"}],["line",{x1:"10",x2:"10",y1:"11",y2:"17",key:"1uufr5"}],["line",{x1:"14",x2:"14",y1:"11",y2:"17",key:"xtxkd"}]]);function I({open:a,onOpenChange:t,title:s,description:d,confirmText:x="Confirm",cancelText:h="Cancel",variant:r="default",onConfirm:f,loading:n=!1}){return e.jsx(m,{open:a,onOpenChange:t,children:e.jsxs(p,{children:[e.jsxs(v,{children:[e.jsxs(T,{className:"flex items-center gap-2",children:[r==="destructive"&&e.jsx(u,{className:"h-5 w-5 text-destructive"}),s]}),e.jsx(g,{children:d})]}),e.jsxs(y,{children:[e.jsx(c,{variant:"outline",onClick:()=>t(!1),disabled:n,children:h}),e.jsx(c,{variant:r==="destructive"?"destructive":"default",onClick:f,loading:n,children:x})]})]})})}function B({status:a,authorized:t}){if(a===l.ACTIVE&&t)return e.jsx(i,{variant:"success",children:"Connected"});if(a===l.ACTIVE&&!t)return e.jsx(i,{variant:"warning",children:"Awaiting QR"});const s=j[a];return e.jsx(i,{variant:s.variant,children:s.label})}export{I as C,A as R,B as S,w as T};
