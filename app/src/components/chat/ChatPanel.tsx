import { useState } from "react"
import { ContactList } from "./ContactList"
import { ChatView } from "./ChatView"
import type { Contact } from "@/types"
import { MessageSquare } from "lucide-react"
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui"

export function ChatPanel() {
  const [selectedContact, setSelectedContact] = useState<Contact | undefined>()

  return (
    <ResizablePanelGroup orientation="horizontal" className="h-full">
      {/* Contact List - Resizable */}
      <ResizablePanel id="contacts" defaultSize="25%" minSize="200px" maxSize="40%">
        <ContactList
          selectedContact={selectedContact}
          onSelectContact={setSelectedContact}
        />
      </ResizablePanel>

      <ResizableHandle withHandle />

      {/* Chat View - Flexible width */}
      <ResizablePanel id="chat" defaultSize="75%" minSize="40%">
        {selectedContact ? (
          <ChatView contact={selectedContact} />
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground bg-muted/30">
            <MessageSquare className="h-16 w-16 mb-4 opacity-50" />
            <h2 className="text-xl font-semibold mb-2">WhatsApp Web</h2>
            <p className="text-sm">Select a chat to start messaging</p>
          </div>
        )}
      </ResizablePanel>
    </ResizablePanelGroup>
  )
}
