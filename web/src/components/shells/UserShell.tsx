import type { CSSProperties, ReactNode } from "react";
import { Link, NavLink, Outlet } from "react-router-dom";
import { Body, Button, Header, Layout, Sidebar, ThemeSwitcher } from "@devstroop/react-uikit";
import { useResponsiveSidebar } from "./useResponsiveSidebar";

export type UserShellProps = {
  children?: ReactNode;
  username?: string;
  onLogout?: () => void | Promise<void>;
};

const linkStyle = ({ isActive }: { isActive: boolean }): CSSProperties => ({
  display: "block",
  padding: "var(--dt-space-2) var(--dt-space-3)",
  borderRadius: "var(--dt-radius-sm)",
  textDecoration: "none",
  fontSize: "var(--dt-font-size-sm)",
  fontWeight: 500,
  color: isActive ? "var(--dt-color-primary)" : "var(--dt-color-text)",
  background: isActive ? "var(--dt-color-surface-hover)" : "transparent",
});

export function UserShell({ children, username, onLogout }: UserShellProps) {
  const { isOverlay, expanded, setExpanded } = useResponsiveSidebar();

  return (
    <Layout>
      <Header>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            width: "100%",
            gap: "var(--dt-space-3)",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "var(--dt-space-3)" }}>
            <button
              type="button"
              data-se-sidebar-toggle
              aria-label="Toggle sidebar"
              aria-expanded={expanded}
              onClick={() => setExpanded((v) => !v)}
              style={{
                display: "inline-flex",
                alignItems: "center",
                justifyContent: "center",
                width: "32px",
                height: "32px",
                borderRadius: "var(--dt-radius-sm)",
                border: "1px solid var(--dt-color-border)",
                background: "var(--dt-color-surface)",
                cursor: "pointer",
              }}
            >
              ☰
            </button>
            <Link
              to="/app"
              style={{
                fontWeight: 700,
                letterSpacing: "-0.02em",
                textDecoration: "none",
                color: "var(--dt-color-text)",
              }}
            >
              WAS <span style={{ color: "var(--dt-color-primary)" }}>Admin</span>
            </Link>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--dt-space-3)" }}>
            <ThemeSwitcher />
            {username ? (
              <span style={{ fontSize: "var(--dt-font-size-sm)", color: "var(--dt-color-text-muted)" }}>
                {username}
              </span>
            ) : null}
            {onLogout ? (
              <Button variant="secondary" size="sm" onClick={() => void onLogout()}>
                Sign out
              </Button>
            ) : null}
          </div>
        </div>
      </Header>
      <Sidebar
        position="left"
        expanded={expanded}
        overlay={isOverlay}
        responsive={isOverlay}
        onClose={() => setExpanded(false)}
      >
        <nav aria-label="User navigation" style={{ display: "flex", flexDirection: "column", gap: "var(--dt-space-1)" }}>
          <NavLink to="/app" end style={linkStyle}>
            Dashboard
          </NavLink>
          <NavLink to="/app/instances" style={linkStyle}>
            Instances
          </NavLink>
          <NavLink to="/app/api-keys" style={linkStyle}>
            API Keys
          </NavLink>
        </nav>
      </Sidebar>
      <Body>{children ?? <Outlet />}</Body>
    </Layout>
  );
}

export default UserShell;
