import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/Dialog';
import { useCreateInstance } from '../hooks/useInstances';
import { Plus } from 'lucide-react';

const createInstanceSchema = z.object({
  name: z.string().min(1, 'Name is required').max(50, 'Name must be 50 characters or less'),
});

type CreateInstanceForm = z.infer<typeof createInstanceSchema>;

export function CreateInstanceModal() {
  const [open, setOpen] = useState(false);
  const createInstance = useCreateInstance();

  const { register, handleSubmit, reset, formState: { errors } } = useForm<CreateInstanceForm>({
    resolver: zodResolver(createInstanceSchema),
  });

  const onSubmit = async (data: CreateInstanceForm) => {
    await createInstance.mutateAsync(data);
    reset();
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button><Plus className="mr-2 h-4 w-4" />New Instance</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Instance</DialogTitle>
          <DialogDescription>Create a new WhatsApp instance to connect your account.</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Instance Name</Label>
            <Input id="name" placeholder="My WhatsApp" {...register('name')} error={!!errors.name} />
            {errors.name && <p className="text-sm text-destructive">{errors.name.message}</p>}
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
            <Button type="submit" loading={createInstance.isPending}>Create</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
