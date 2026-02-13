import { Settings } from "@/components/dashboard"

export function SettingsPage() {
  return (
    <div className="p-6 overflow-auto h-full">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-muted-foreground">
          Configure your WAS client settings
        </p>
      </div>
      <Settings />
    </div>
  )
}
