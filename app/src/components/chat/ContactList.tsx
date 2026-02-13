import { useQuery } from "@tanstack/react-query"
import {
  ScrollArea,
  Avatar,
  AvatarFallback,
  AvatarImage,
  Input,
  Spinner,
} from "@/components/ui"
import { apiClient } from "@/lib/api"
import { cn, truncate } from "@/lib/utils"
import type { Contact } from "@/types"
import { Search, Users, User } from "lucide-react"
import { useState } from "react"

interface ContactListProps {
  selectedContact?: Contact
  onSelectContact: (contact: Contact) => void
}

export function ContactList({ selectedContact, onSelectContact }: ContactListProps) {
  const [searchQuery, setSearchQuery] = useState("")

  const contactsQuery = useQuery({
    queryKey: ["contacts"],
    queryFn: () => apiClient.getChats(),
    refetchInterval: 30000, // Refresh every 30 seconds
  })

  const filteredContacts = contactsQuery.data?.filter((contact) =>
    contact.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (contact.phone_number?.includes(searchQuery) ?? false) ||
    (contact.id?.includes(searchQuery) ?? false)
  ) || []

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2)
  }

  return (
    <div className="flex flex-col h-full border-r">
      {/* Search Header */}
      <div className="p-4 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search chats..."
            className="pl-9"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      {/* Contact List */}
      <ScrollArea className="flex-1">
        {contactsQuery.isLoading ? (
          <div className="flex items-center justify-center h-40">
            <Spinner size="lg" />
          </div>
        ) : contactsQuery.error ? (
          <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
            <p className="text-sm">Failed to load contacts</p>
          </div>
        ) : filteredContacts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
            <Users className="h-12 w-12 mb-2 opacity-50" />
            <p className="text-sm">
              {searchQuery ? "No contacts found" : "No chats yet"}
            </p>
          </div>
        ) : (
          <div className="divide-y">
            {filteredContacts.map((contact) => (
              <button
                key={contact.id}
                onClick={() => onSelectContact(contact)}
                className={cn(
                  "w-full flex items-center gap-3 p-4 hover:bg-accent transition-colors text-left",
                  selectedContact?.id === contact.id && "bg-accent"
                )}
              >
                <Avatar className="h-12 w-12">
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

                <div className="flex-1 min-w-0">
                  {/* Row 1: Name only - no timestamp (timestamp is actually status from backend) */}
                  <div className="flex items-center gap-2">
                    <p className="font-medium truncate flex-1 min-w-0">{contact.name}</p>
                  </div>
                  {/* Row 2: Last Message + Unread Badge */}
                  <div className="flex items-center gap-2 mt-0.5">
                    <p className="text-sm text-muted-foreground truncate flex-1 min-w-0">
                      {contact.last_message ? (
                        truncate(contact.last_message, 35)
                      ) : (
                        <span className="italic opacity-60">No messages</span>
                      )}
                    </p>
                    {contact.unread_count && contact.unread_count > 0 && (
                      <span className="flex-shrink-0 flex items-center justify-center h-5 min-w-5 px-1.5 text-xs font-medium bg-whatsapp text-white rounded-full">
                        {contact.unread_count > 99 ? "99+" : contact.unread_count}
                      </span>
                    )}
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  )
}
