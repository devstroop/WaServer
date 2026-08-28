import type { ReactNode } from "react";
import { Outlet } from "react-router-dom";
import { Card, ThemeSwitcher } from "@devstroop/react-uikit";

export type AuthShellProps = {
  children?: ReactNode;
  title?: ReactNode;
};

export function AuthShell({ children, title }: AuthShellProps) {
  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--dt-space-4)",
        background: "var(--dt-color-background, var(--dt-color-surface))",
        position: "relative",
      }}
    >
      <div style={{ position: "absolute", top: "var(--dt-space-4)", right: "var(--dt-space-4)" }}>
        <ThemeSwitcher />
      </div>
      <Card
        variant="elevated"
        header={title ?? "WAS Admin"}
        style={{ width: "100%", maxWidth: "420px" }}
      >
        {children ?? <Outlet />}
      </Card>
    </div>
  );
}

export default AuthShell;
