import{j as e}from"./query-BhuALcwA.js";import{u as h,h as g}from"./vendor-yaVYLZzK.js";import{c as r,M as j,B as d,q as n,H as u,N as b,R as i,X as p}from"./index-BDKGGG7P.js";import{T as m}from"./theme-toggle-Bq6uSTND.js";/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const N=r("ChevronLeft",[["path",{d:"m15 18-6-6 6-6",key:"1wnfg3"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const v=r("LayoutDashboard",[["rect",{width:"7",height:"9",x:"3",y:"3",rx:"1",key:"10lvy0"}],["rect",{width:"7",height:"5",x:"14",y:"3",rx:"1",key:"16une8"}],["rect",{width:"7",height:"9",x:"14",y:"12",rx:"1",key:"1hutg5"}],["rect",{width:"7",height:"5",x:"3",y:"16",rx:"1",key:"ldoo1y"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const y=r("LogOut",[["path",{d:"M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4",key:"1uf3rs"}],["polyline",{points:"16 17 21 12 16 7",key:"1gabdz"}],["line",{x1:"21",x2:"9",y1:"12",y2:"12",key:"1uyos4"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const k=r("Menu",[["line",{x1:"4",x2:"20",y1:"12",y2:"12",key:"1e0a9i"}],["line",{x1:"4",x2:"20",y1:"6",y2:"6",key:"1owob3"}],["line",{x1:"4",x2:"20",y1:"18",y2:"18",key:"yk5zj1"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const S=r("MessageSquare",[["path",{d:"M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",key:"1lielz"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const w=r("Server",[["rect",{width:"20",height:"8",x:"2",y:"2",rx:"2",ry:"2",key:"ngkwjq"}],["rect",{width:"20",height:"8",x:"2",y:"14",rx:"2",ry:"2",key:"iecqi9"}],["line",{x1:"6",x2:"6.01",y1:"6",y2:"6",key:"16zg32"}],["line",{x1:"6",x2:"6.01",y1:"18",y2:"18",key:"nzw8ys"}]]);/**
 * @license lucide-react v0.359.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const L=r("Settings",[["path",{d:"M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",key:"1qme2f"}],["circle",{cx:"12",cy:"12",r:"3",key:"1v7zrd"}]]),f=[{label:"Dashboard",href:i.DASHBOARD,icon:v},{label:"Instances",href:i.INSTANCES,icon:w},{label:"Messages",href:i.MESSAGES,icon:S},{label:"Settings",href:i.SETTINGS,icon:L}];function z({collapsed:s,onToggle:t}){const c=h(),o=u(a=>a.logout),l=()=>{o(),c(i.LOGIN)};return e.jsx("aside",{className:n("fixed left-0 top-0 z-40 h-screen bg-card border-r transition-all duration-300",s?"w-16":"w-64"),children:e.jsxs("div",{className:"flex h-full flex-col",children:[e.jsxs("div",{className:"flex h-16 items-center justify-between border-b px-4",children:[!s&&e.jsx("span",{className:"text-xl font-bold text-primary",children:"WAS"}),e.jsx(d,{variant:"ghost",size:"icon",onClick:t,className:"ml-auto",children:s?e.jsx(b,{className:"h-4 w-4"}):e.jsx(N,{className:"h-4 w-4"})})]}),e.jsx("nav",{className:"flex-1 space-y-1 p-2",children:f.map(a=>e.jsxs(g,{to:a.href,className:({isActive:x})=>n("flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",x?"bg-accent text-accent-foreground":"text-muted-foreground",s&&"justify-center"),children:[e.jsx(a.icon,{className:"h-5 w-5 shrink-0"}),!s&&e.jsx("span",{children:a.label})]},a.href))}),e.jsxs("div",{className:"border-t p-2 space-y-1",children:[e.jsx("div",{className:n("flex items-center",s?"justify-center":"px-3"),children:e.jsx(m,{})}),e.jsxs(d,{variant:"ghost",className:n("w-full justify-start gap-3",s&&"justify-center"),onClick:l,children:[e.jsx(y,{className:"h-5 w-5"}),!s&&e.jsx("span",{children:"Logout"})]})]})]})})}function M({isOpen:s,onClose:t}){const c=h(),o=u(a=>a.logout),l=()=>{o(),c(i.LOGIN),t()};return s?e.jsxs(e.Fragment,{children:[e.jsx("div",{className:"fixed inset-0 z-40 bg-background/80 backdrop-blur-sm lg:hidden",onClick:t}),e.jsxs("aside",{className:"fixed left-0 top-0 z-50 h-screen w-64 bg-card border-r lg:hidden",children:[e.jsxs("div",{className:"flex h-16 items-center justify-between border-b px-4",children:[e.jsx("span",{className:"text-xl font-bold text-primary",children:"WAS"}),e.jsx(d,{variant:"ghost",size:"icon",onClick:t,children:e.jsx(p,{className:"h-4 w-4"})})]}),e.jsx("nav",{className:"flex-1 space-y-1 p-2",children:f.map(a=>e.jsxs(g,{to:a.href,onClick:t,className:({isActive:x})=>n("flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent",x?"bg-accent text-accent-foreground":"text-muted-foreground"),children:[e.jsx(a.icon,{className:"h-5 w-5"}),e.jsx("span",{children:a.label})]},a.href))}),e.jsxs("div",{className:"border-t p-2",children:[e.jsx("div",{className:"px-3 mb-2",children:e.jsx(m,{})}),e.jsxs(d,{variant:"ghost",className:"w-full justify-start gap-3",onClick:l,children:[e.jsx(y,{className:"h-5 w-5"}),e.jsx("span",{children:"Logout"})]})]})]})]}):null}function T({children:s}){const{sidebarCollapsed:t,sidebarOpen:c,toggleSidebar:o,setSidebarOpen:l}=j();return e.jsxs("div",{className:"min-h-screen bg-background",children:[e.jsx("div",{className:"hidden lg:block",children:e.jsx(z,{collapsed:t,onToggle:o})}),e.jsx(M,{isOpen:c,onClose:()=>l(!1)}),e.jsxs("header",{className:"sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background px-4 lg:hidden",children:[e.jsx(d,{variant:"ghost",size:"icon",onClick:()=>l(!0),children:e.jsx(k,{className:"h-5 w-5"})}),e.jsx("span",{className:"text-xl font-bold text-primary",children:"WAS"})]}),e.jsx("main",{className:n("transition-all duration-300 p-6",t?"lg:ml-16":"lg:ml-64"),children:s})]})}function q({className:s,...t}){return e.jsx("div",{className:n("animate-pulse rounded-md bg-muted",s),...t})}export{y as L,T as M,w as S,q as a,S as b,L as c};
