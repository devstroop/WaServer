import { useEffect, useRef } from 'react';
import { Check, CheckCheck, Clock, AlertCircle, Image, FileText, Mic, Video, Sticker } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/shared/EmptyState';
import type { Message } from '@/services/messaging.service';
import { formatDate } from '@/lib/utils';

interface MessageListProps {
  messages: Message[] | undefined;
  isLoading: boolean;
}

function getMessageIcon(type: Message['message_type']) {
  switch (type) {
    case 'image': return <Image className="h-4 w-4" />;
    case 'video': return <Video className="h-4 w-4" />;
    case 'audio': return <Mic className="h-4 w-4" />;
    case 'document': return <FileText className="h-4 w-4" />;
    case 'sticker': return <Sticker className="h-4 w-4" />;
    default: return null;
  }
}

function getStatusIcon(status: Message['status']) {
  switch (status) {
    case 'pending': return <Clock className="h-3 w-3 text-muted-foreground" />;
    case 'sent': return <Check className="h-3 w-3 text-muted-foreground" />;
    case 'delivered': return <CheckCheck className="h-3 w-3 text-muted-foreground" />;
    case 'read': return <CheckCheck className="h-3 w-3 text-blue-500" />;
    case 'failed': return <AlertCircle className="h-3 w-3 text-destructive" />;
  }
}

export function MessageList({ messages, isLoading }: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  if (isLoading) {
    return (
      <div className="flex-1 p-4 space-y-4">
        {Array.from({ length: 8 }).map((_, i) => (
          <div key={i} className={cn('flex', i % 2 === 0 ? 'justify-start' : 'justify-end')}>
            <Skeleton className={cn('h-12 rounded-lg', i % 2 === 0 ? 'w-48' : 'w-64')} />
          </div>
        ))}
      </div>
    );
  }

  if (!messages?.length) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <EmptyState title="No messages" description="Send a message to start the conversation." />
      </div>
    );
  }

  let lastDate = '';

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-2">
      {messages.map((message) => {
        const messageDate = formatDate(message.timestamp, 'date');
        const showDateSeparator = messageDate !== lastDate;
        lastDate = messageDate;

        return (
          <div key={message.id}>
            {showDateSeparator && (
              <div className="flex justify-center my-4">
                <span className="text-xs bg-muted px-3 py-1 rounded-full text-muted-foreground">{messageDate}</span>
              </div>
            )}
            <div className={cn('flex', message.from_me ? 'justify-end' : 'justify-start')}>
              <div className={cn('max-w-[70%] rounded-lg px-3 py-2', message.from_me ? 'bg-primary text-primary-foreground' : 'bg-muted')}>
                {message.message_type !== 'text' && (
                  <div className="flex items-center gap-1 mb-1 text-xs opacity-70">
                    {getMessageIcon(message.message_type)}
                    <span className="capitalize">{message.message_type}</span>
                  </div>
                )}
                {message.media_url && message.message_type === 'image' && (
                  <img src={message.media_url} alt="Media" className="rounded max-h-48 mb-2" />
                )}
                <p className="text-sm whitespace-pre-wrap break-words">{message.content}</p>
                <div className={cn('flex items-center gap-1 mt-1', message.from_me ? 'justify-end' : 'justify-start')}>
                  <span className="text-xs opacity-70">{formatDate(message.timestamp, 'time')}</span>
                  {message.from_me && getStatusIcon(message.status)}
                </div>
              </div>
            </div>
          </div>
        );
      })}
      <div ref={bottomRef} />
    </div>
  );
}
