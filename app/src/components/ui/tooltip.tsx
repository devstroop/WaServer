import * as React from "react"
import { cn } from "@/lib/utils"

interface TooltipContextType {
  open: boolean
  setOpen: (open: boolean) => void
}

const TooltipContext = React.createContext<TooltipContextType | undefined>(undefined)

export function TooltipProvider({ children }: { children: React.ReactNode }) {
  return <>{children}</>
}

export function Tooltip({ children, className }: { children: React.ReactNode; className?: string }) {
  const [open, setOpen] = React.useState(false)

  return (
    <TooltipContext.Provider value={{ open, setOpen }}>
      <div className={cn("relative", className)}>
        {children}
      </div>
    </TooltipContext.Provider>
  )
}

export function TooltipTrigger({ 
  children, 
  asChild 
}: { 
  children: React.ReactNode
  asChild?: boolean 
}) {
  const context = React.useContext(TooltipContext)
  if (!context) throw new Error("TooltipTrigger must be used within Tooltip")

  const handleMouseEnter = () => context.setOpen(true)
  const handleMouseLeave = () => context.setOpen(false)

  if (asChild && React.isValidElement(children)) {
    return React.cloneElement(children as React.ReactElement<{
      onMouseEnter?: () => void
      onMouseLeave?: () => void
    }>, {
      onMouseEnter: handleMouseEnter,
      onMouseLeave: handleMouseLeave,
    })
  }

  return (
    <span onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave}>
      {children}
    </span>
  )
}

interface TooltipContentProps {
  children: React.ReactNode
  className?: string
  side?: "top" | "bottom" | "left" | "right"
  sideOffset?: number
}

export function TooltipContent({ 
  children, 
  className,
  side = "right",
  sideOffset = 8
}: TooltipContentProps) {
  const context = React.useContext(TooltipContext)
  if (!context) throw new Error("TooltipContent must be used within Tooltip")

  if (!context.open) return null

  const positionClasses = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
    left: "right-full top-1/2 -translate-y-1/2 mr-2",
    right: "left-full top-1/2 -translate-y-1/2",
  }

  return (
    <div
      className={cn(
        "absolute z-50 px-3 py-1.5 text-xs font-medium text-popover-foreground bg-popover rounded-md shadow-md border",
        "animate-in fade-in-0 zoom-in-95",
        positionClasses[side],
        className
      )}
      style={{ marginLeft: side === "right" ? sideOffset : undefined }}
    >
      {children}
    </div>
  )
}
