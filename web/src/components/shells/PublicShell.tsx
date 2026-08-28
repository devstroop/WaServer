import type { ReactNode } from "react";
import { Link, Outlet } from "react-router-dom";
import { Body, Footer, Header, Layout, ThemeSwitcher } from "@devstroop/react-uikit";

export type PublicShellProps = {
  children?: ReactNode;
};

export function PublicShell({ children }: PublicShellProps) {
  return (
    <Layout>
      <Header>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            width: "100%",
            gap: "var(--dt-space-4)",
          }}
        >
          <Link
            to="/"
            style={{
              fontWeight: 700,
              letterSpacing: "-0.02em",
              textDecoration: "none",
              color: "var(--dt-color-text)",
            }}
          >
            WAS <span style={{ color: "var(--dt-color-primary)" }}>Admin</span>
          </Link>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--dt-space-3)" }}>
            <ThemeSwitcher />
          </div>
        </div>
      </Header>
      <Body>{children ?? <Outlet />}</Body>
      <Footer>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            width: "100%",
            fontSize: "var(--dt-font-size-sm)",
            color: "var(--dt-color-text-muted)",
          }}
        >
          <span>© {new Date().getFullYear()} WAS — WhatsApp Admin Service</span>
          <a
            href="/api-docs/"
            target="_blank"
            rel="noreferrer"
            style={{ color: "inherit", textDecoration: "none" }}
          >
            API Docs
          </a>
        </div>
      </Footer>
    </Layout>
  );
}

export default PublicShell;
