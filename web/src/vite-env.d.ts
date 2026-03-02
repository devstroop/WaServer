/// <reference types="vite/client" />

// CSS modules
declare module '*.css' {
  const content: { [className: string]: string };
  export default content;
}

// Image files
declare module '*.svg' {
  const content: string;
  export default content;
}

declare module '*.png' {
  const content: string;
  export default content;
}
