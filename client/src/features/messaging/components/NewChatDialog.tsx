import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Textarea } from '@/components/ui/Textarea';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/Dialog';
import { useSendTextMessage } from '../hooks/useMessaging';

const newChatSchema = z.object({
  phoneNumber: z.string().min(10, 'Invalid phone number').regex(/^[0-9+\-\s]+$/, 'Invalid phone number format'),
  message: z.string().min(1, 'Message is required'),
});

type NewChatForm = z.infer<typeof newChatSchema>;

interface NewChatDialogProps {
  instanceId: string;
}

export function NewChatDialog({ instanceId }: NewChatDialogProps) {
  const [open, setOpen] = useState(false);
  const sendMessage = useSendTextMessage();

  const { register, handleSubmit, reset, formState: { errors } } = useForm<NewChatForm>({
    resolver: zodResolver(newChatSchema),
  });

  const onSubmit = async (data: NewChatForm) => {
    const cleanNumber = data.phoneNumber.replace(/[\s\-]/g, '');
    await sendMessage.mutateAsync({ instanceId, phone: cleanNumber, text: data.message });
    reset();
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="icon" variant="outline"><Plus className="h-4 w-4" /></Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New Conversation</DialogTitle>
          <DialogDescription>Start a conversation with a new contact.</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="phoneNumber">Phone Number</Label>
            <Input id="phoneNumber" placeholder="+1 234 567 8900" {...register('phoneNumber')} error={!!errors.phoneNumber} />
            {errors.phoneNumber && <p className="text-sm text-destructive">{errors.phoneNumber.message}</p>}
          </div>
          <div className="space-y-2">
            <Label htmlFor="message">Message</Label>
            <Textarea id="message" placeholder="Type your message..." {...register('message')} />
            {errors.message && <p className="text-sm text-destructive">{errors.message.message}</p>}
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
            <Button type="submit" loading={sendMessage.isPending}>Send</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
