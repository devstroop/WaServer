import { useState, useRef } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Send, Paperclip, X, Image, FileText } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Textarea } from '@/components/ui/Textarea';
import { useSendTextMessage, useSendMediaMessage } from '../hooks/useMessaging';
import { FILE_LIMITS } from '@/lib/constants';
import { formatBytes } from '@/lib/utils';
import { useToast } from '@/hooks/useToast';

const messageSchema = z.object({
  message: z.string().min(1, 'Message is required'),
});

type MessageForm = z.infer<typeof messageSchema>;

interface ComposeMessageProps {
  instanceId: string;
  chatId: string;
  disabled?: boolean;
}

export function ComposeMessage({ instanceId, chatId, disabled }: ComposeMessageProps) {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const sendText = useSendTextMessage();
  const sendMedia = useSendMediaMessage();
  const { toast } = useToast();

  const { register, handleSubmit, reset, formState: { errors } } = useForm<MessageForm>({
    resolver: zodResolver(messageSchema),
  });

  const onSubmit = async (data: MessageForm) => {
    if (selectedFile) {
      await sendMedia.mutateAsync({ instanceId, phone: chatId, file: selectedFile, caption: data.message });
      setSelectedFile(null);
    } else {
      await sendText.mutateAsync({ instanceId, phone: chatId, text: data.message });
    }
    reset();
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > FILE_LIMITS.MAX_SIZE) {
      toast({ title: 'File too large', description: `Maximum size is ${formatBytes(FILE_LIMITS.MAX_SIZE)}`, variant: 'destructive' });
      return;
    }
    setSelectedFile(file);
  };

  const isLoading = sendText.isPending || sendMedia.isPending;
  const isImage = selectedFile?.type.startsWith('image/');

  return (
    <div className="border-t p-4">
      {selectedFile && (
        <div className="flex items-center gap-2 mb-2 p-2 bg-muted rounded">
          {isImage ? <Image className="h-4 w-4" /> : <FileText className="h-4 w-4" />}
          <span className="text-sm flex-1 truncate">{selectedFile.name}</span>
          <span className="text-xs text-muted-foreground">{formatBytes(selectedFile.size)}</span>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setSelectedFile(null)}>
            <X className="h-4 w-4" />
          </Button>
        </div>
      )}
      <form onSubmit={handleSubmit(onSubmit)} className="flex items-end gap-2">
        <input type="file" ref={fileInputRef} className="hidden" onChange={handleFileSelect} accept="image/*,application/pdf,.doc,.docx,.txt" />
        <Button type="button" variant="ghost" size="icon" onClick={() => fileInputRef.current?.click()} disabled={disabled || isLoading}>
          <Paperclip className="h-5 w-5" />
        </Button>
        <div className="flex-1">
          <Textarea
            placeholder="Type a message..."
            className="min-h-[44px] max-h-32 resize-none"
            rows={1}
            disabled={disabled || isLoading}
            {...register('message')}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSubmit(onSubmit)();
              }
            }}
          />
          {errors.message && <p className="text-xs text-destructive mt-1">{errors.message.message}</p>}
        </div>
        <Button type="submit" size="icon" disabled={disabled || isLoading} loading={isLoading}>
          <Send className="h-5 w-5" />
        </Button>
      </form>
    </div>
  );
}
