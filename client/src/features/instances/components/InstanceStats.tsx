import { Server, Zap, Power, AlertTriangle } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Skeleton } from '@/components/ui/Skeleton';
import { useInstanceStats } from '../hooks/useInstances';

export function InstanceStats() {
  const { data: stats, isLoading } = useInstanceStats();

  const items = [
    { label: 'Total Instances', value: stats?.total ?? 0, icon: Server, color: 'text-blue-500' },
    { label: 'Active', value: stats?.active ?? 0, icon: Zap, color: 'text-green-500' },
    { label: 'Inactive', value: stats?.inactive ?? 0, icon: Power, color: 'text-gray-500' },
    { label: 'Errors', value: stats?.error ?? 0, icon: AlertTriangle, color: 'text-red-500' },
  ];

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {items.map((item) => (
        <Card key={item.label}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{item.label}</CardTitle>
            <item.icon className={`h-4 w-4 ${item.color}`} />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <div className="text-2xl font-bold">{item.value.toLocaleString()}</div>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
