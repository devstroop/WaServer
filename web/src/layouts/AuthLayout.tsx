import { type JSX } from "react";
import { Outlet } from "react-router-dom";

export default function AuthLayout(): JSX.Element {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[#F8FAFC] p-4">
      <div className="w-full max-w-sm">
        <Outlet />
      </div>
    </div>
  );
}
