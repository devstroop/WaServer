import { useEffect, useRef, useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import {
  ScrollArea,
  Avatar,
  AvatarFallback,
  AvatarImage,
  Input,
  Button,
  Spinner,
  toast,
} from "@/components/ui"
import { apiClient } from "@/lib/api"
import { cn } from "@/lib/utils"
import type { Contact } from "@/types"
import { Send, User, Users, Check, CheckCheck } from "lucide-react"

interface ChatViewProps {
  contact: Contact
}

export function ChatView({ contact }: ChatViewProps) {
  const [message, setMessage] = useState("")
  const scrollRef = useRef<HTMLDivElement>(null)
  const queryClient = useQueryClient()

  // Extract phone number from contact
  const getPhoneForSend = (): string => {
    // If phone_number is set, use it
    if (contact.phone_number) {
      return contact.phone_number
    }
    // If id is a JID (contains @c.us), extract the phone part
    if (contact.id.includes("@c.us")) {
      return contact.id.split("@")[0]
    }
    // If id is a JID (contains @g.us for groups), can't send directly
    if (contact.id.includes("@g.us")) {
      return contact.id.split("@")[0]
    }
    // If id starts with name:, we can't send - need phone number
    // Return empty and let backend handle the error
    if (contact.id.startsWith("name:")) {
      return ""
    }
    // Otherwise assume id is the phone number
    return contact.id
  }

  const messagesQuery = useQuery({
    queryKey: ["messages", contact.id],
    queryFn: () => apiClient.getMessages(contact.id),
    refetchInterval: 5000, // Refresh every 5 seconds
  })

  const sendMutation = useMutation({
    mutationFn: (text: string) => {
      const phone = getPhoneForSend()
      if (!phone) {
        return Promise.reject(new Error("Cannot send message: no phone number available for this contact"))
      }
      return apiClient.sendMessage({ phone, text })
    },
    onSuccess: () => {
      setMessage("")
      queryClient.invalidateQueries({ queryKey: ["messages", contact.id] })
      queryClient.invalidateQueries({ queryKey: ["contacts"] })
      toast({
        title: "Message sent",
        variant: "success",
      })
    },
    onError: (error: Error) => {
      toast({
        title: "Failed to send message",
        description: error.message,
        variant: "destructive",
      })
    },
  })

  // Scroll to bottom on new messages
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [messagesQuery.data])

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault()
    if (message.trim()) {
      sendMutation.mutate(message.trim())
    }
  }

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2)
  }

  const messages = messagesQuery.data || []

  const renderStatus = (status?: string) => {
    switch (status) {
      case "sent":
        return <Check className="h-3 w-3 text-muted-foreground" />
      case "delivered":
        return <CheckCheck className="h-3 w-3 text-muted-foreground" />
      case "read":
        return <CheckCheck className="h-3 w-3 text-blue-500" />
      default:
        return null
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Chat Header */}
      <div className="flex items-center gap-3 p-4 border-b bg-card">
        <Avatar className="h-10 w-10">
          {contact.avatar_url ? (
            <AvatarImage src={contact.avatar_url} alt={contact.name} />
          ) : null}
          <AvatarFallback className={contact.is_group ? "bg-whatsapp/20" : "bg-primary/20"}>
            {contact.is_group ? (
              <Users className="h-5 w-5" />
            ) : (
              getInitials(contact.name) || <User className="h-5 w-5" />
            )}
          </AvatarFallback>
        </Avatar>
        <div>
          <h2 className="font-semibold">{contact.name}</h2>
          {(() => {
            // Show phone number if available and looks like a phone number
            if (contact.phone_number && /^\+?\d[\d\s-]+$/.test(contact.phone_number)) {
              return <p className="text-xs text-muted-foreground">{contact.phone_number}</p>
            }
            // Extract phone from JID format (e.g., "919501005734@c.us")
            if (contact.id.includes("@c.us")) {
              const phone = contact.id.split("@")[0]
              if (/^\d+$/.test(phone)) {
                return <p className="text-xs text-muted-foreground">+{phone}</p>
              }
            }
            // For groups, show "Group" label
            if (contact.is_group || contact.id.includes("@g.us")) {
              return <p className="text-xs text-muted-foreground">Group</p>
            }
            // For contacts without phone, show "Contact" 
            return <p className="text-xs text-muted-foreground">Contact</p>
          })()}
        </div>
      </div>

      {/* Messages Area */}
      <ScrollArea ref={scrollRef} className="flex-1 p-4">
        {messagesQuery.isLoading ? (
          <div className="flex items-center justify-center h-full">
            <Spinner size="lg" />
          </div>
        ) : messagesQuery.error ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            <p>Failed to load messages</p>
          </div>
        ) : messages.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            <p>No messages yet. Start a conversation!</p>
          </div>
        ) : (
          <div className="space-y-2">
            {messages.map((msg) => (
              <div
                key={msg.id}
                className={cn(
                  "flex",
                  msg.from_me ? "justify-end" : "justify-start"
                )}
              >
                <div
                  className={cn(
                    "max-w-[70%] px-4 py-2 rounded-lg",
                    msg.from_me
                      ? "bg-whatsapp text-white rounded-br-none"
                      : "bg-muted rounded-bl-none"
                  )}
                >
                  {!msg.from_me && contact.is_group && msg.sender && (
                    <p className="text-xs font-medium text-whatsapp-dark mb-1">
                      {msg.sender}
                    </p>
                  )}
                  <p className="text-sm whitespace-pre-wrap break-words">
                    {msg.text || (msg.message_type !== "chat" ? `[${msg.message_type}]` : "[Media]")}
                  </p>
                  <div className={cn(
                    "flex items-center justify-end gap-1 mt-1",
                    msg.from_me ? "text-white/70" : "text-muted-foreground"
                  )}>
                    <span className="text-[10px]">
                      {msg.timestamp || ""}
                    </span>
                    {msg.from_me && renderStatus(msg.status)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>

      {/* Message Input */}
      <form onSubmit={handleSend} className="p-4 border-t bg-card">
        <div className="flex gap-2">
          <Input
            placeholder="Type a message..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            disabled={sendMutation.isPending}
            className="flex-1"
          />
          <Button
            type="submit"
            variant="whatsapp"
            size="icon"
            disabled={!message.trim() || sendMutation.isPending}
          >
            {sendMutation.isPending ? (
              <Spinner size="sm" className="text-white" />
            ) : (
              <Send className="h-5 w-5" />
            )}
          </Button>
        </div>
      </form>
    </div>
  )
}
