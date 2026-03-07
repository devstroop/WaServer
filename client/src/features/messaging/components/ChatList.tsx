import { MessageSquare, Users } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/Avatar';
import { Badge } from '@/components/ui/Badge';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/shared/EmptyState';
import type { Chat } from '@/services/whatsapp.service';
import { formatRelativeTime } from '@/lib/utils';

interface ChatListProps {
  chats: Chat[] | undefined;
  isLoading: boolean;
  selectedChatId: string | null;
  onSelectChat: (chat: Chat) => void;
}

export function ChatList({ chats, isLoading, selectedChatId, onSelectChat }: ChatListProps) {
  if (isLoading) {
    return (
      <div className="space-y-2 p-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="flex items-center gap-3 p-3">
            <Skeleton className="h-12 w-12 rounded-full" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-3 w-48" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (!chats?.length) {
    return <EmptyState icon={MessageSquare} title="No chats yet" description="Start a conversation to see it here." />;
  }

  return (
    <div className="overflow-y-auto">
      {chats.map((chat) => (
        <button
          key={chat.id}
          onClick={() => onSelectChat(chat)}
          className={cn(
            'w-full flex items-center gap-3 p-3 text-left transition-colors hover:bg-accent',
            selectedChatId === chat.id && 'bg-accent'
          )}
        >
          <Avatar className="h-12 w-12">
            <AvatarImage src={chat.contact.profile_picture_url ?? undefined} alt={chat.contact.name} />
            <AvatarFallback>{chat.contact.is_group ? <Users className="h-5 w-5" /> : chat.contact.name.slice(0, 2).toUpperCase()}</AvatarFallback>
          </Avatar>
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between">
              <p className="font-medium truncate">{chat.contact.name}</p>
              {chat.last_message_time && (
                <span className="text-xs text-muted-foreground">{formatRelativeTime(chat.last_message_time)}</span>
              )}
            </div>
            <div className="flex items-center justify-between">
              <p className="text-sm text-muted-foreground truncate">{chat.last_message ?? 'No messages'}</p>
              {chat.unread_count > 0 && <Badge variant="default" className="ml-2">{chat.unread_count}</Badge>}
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}
