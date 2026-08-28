import { useEffect, useState } from "react";

export function useResponsiveSidebar() {
  const [isOverlay, setIsOverlay] = useState(() => {
    if (typeof window === "undefined") return false;
    return window.matchMedia("(max-width: 767.98px)").matches;
  });
  const [expanded, setExpanded] = useState(() => {
    if (typeof window === "undefined") return true;
    return !window.matchMedia("(max-width: 767.98px)").matches;
  });

  useEffect(() => {
    const mql = window.matchMedia("(max-width: 767.98px)");
    const onChange = () => {
      const overlay = mql.matches;
      setIsOverlay(overlay);
      setExpanded(!overlay);
    };
    onChange();
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  return { isOverlay, expanded, setExpanded };
}
