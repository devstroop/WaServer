import { useState, useEffect } from 'react';
import { MessageSquare, Users } from 'lucide-react';
import { MainLayout } from '@/layouts';
import { Card } from '@/components/ui/Card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { EmptyState } from '@/components/shared/EmptyState';
import { ChatList, MessageList, ComposeMessage, NewChatDialog } from '@/features/messaging';
import { useInstances } from '@/features/instances/hooks/useInstances';
import { useChats } from '@/features/instances/hooks/useWhatsApp';
import { useMessages, useMarkAsRead } from '@/features/messaging/hooks/useMessaging';
import type { Chat } from '@/services/whatsapp.service';
import type { Instance } from '@/services/instance.service';
import { INSTANCE_STATUS } from '@/lib/constants';

export function MessagesPage() {
  const { data: instances, isLoading: instancesLoading } = useInstances();
  const [selectedInstanceId, setSelectedInstanceId] = useState<string | null>(null);
  const [selectedChat, setSelectedChat] = useState<Chat | null>(null);

  const activeInstances = instances?.instances?.filter((i: Instance) => i.status === INSTANCE_STATUS.ACTIVE && i.authorized) ?? [];

  useEffect(() => {
    if (activeInstances.length > 0 && !selectedInstanceId) {
      setSelectedInstanceId(activeInstances[0]?.id ?? null);
    }
  }, [activeInstances, selectedInstanceId]);

  const { data: chats, isLoading: chatsLoading } = useChats(selectedInstanceId ?? '', !!selectedInstanceId);
  const { data: messages, isLoading: messagesLoading } = useMessages({
    instanceId: selectedInstanceId ?? '',
    phone: selectedChat?.id ?? '',
    limit: 50,
  });
  const markAsRead = useMarkAsRead();

  const handleSelectChat = (chat: Chat) => {
    setSelectedChat(chat);
    if (selectedInstanceId && chat.unread_count > 0) {
      markAsRead.mutate({ instanceId: selectedInstanceId, phone: chat.id });
    }
  };

  if (!instancesLoading && activeInstances.length === 0) {
    return (
      <MainLayout>
        <div className="space-y-6">
          <h1 className="text-3xl font-bold">Messages</h1>
          <EmptyState
            icon={MessageSquare}
            title="No active instances"
            description="You need at least one connected WhatsApp instance to send messages."
          />
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h1 className="text-3xl font-bold">Messages</h1>
          <Select value={selectedInstanceId ?? ''} onValueChange={setSelectedInstanceId}>
            <SelectTrigger className="w-48">
              <SelectValue placeholder="Select instance" />
            </SelectTrigger>
            <SelectContent>
              {activeInstances.map((instance: Instance) => (
                <SelectItem key={instance.id} value={instance.id}>{instance.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 h-[calc(100vh-12rem)]">
          <Card className="lg:col-span-1 flex flex-col overflow-hidden">
            <div className="p-4 border-b flex items-center justify-between">
              <h2 className="font-semibold">Chats</h2>
              {selectedInstanceId && <NewChatDialog instanceId={selectedInstanceId} />}
            </div>
            <ChatList
              chats={chats}
              isLoading={chatsLoading}
              selectedChatId={selectedChat?.id ?? null}
              onSelectChat={handleSelectChat}
            />
          </Card>

          <Card className="lg:col-span-2 flex flex-col overflow-hidden">
            {selectedChat ? (
              <>
                <div className="p-4 border-b flex items-center gap-3">
                  <div className="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center">
                    {selectedChat.contact.is_group ? (
                      <Users className="h-5 w-5 text-primary" />
                    ) : (
                      <span className="text-primary font-medium">{selectedChat.contact.name.slice(0, 2).toUpperCase()}</span>
                    )}
                  </div>
                  <div>
                    <h2 className="font-semibold">{selectedChat.contact.name}</h2>
                    <p className="text-sm text-muted-foreground">{selectedChat.contact.phone_number}</p>
                  </div>
                </div>
                <MessageList messages={messages?.messages} isLoading={messagesLoading} />
                <ComposeMessage instanceId={selectedInstanceId ?? ''} chatId={selectedChat.id} />
              </>
            ) : (
              <div className="flex-1 flex items-center justify-center">
                <EmptyState icon={MessageSquare} title="Select a chat" description="Choose a conversation from the list to view messages." />
              </div>
            )}
          </Card>
        </div>
      </div>
    </MainLayout>
  );
}
