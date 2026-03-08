import { useState } from 'react';
import { Plus, Copy, Eye, EyeOff, MoreVertical, Trash2, RefreshCw } from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { DataTable, type Column } from '@/components/tables/DataTable';
import { StatusBadge } from '@/components/badges/StatusBadge';
import { InputField } from '@/components/forms/InputField';
import { Modal } from '@/components/modals/Modal';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/DropdownMenu';
import { apiKeys } from '@/mock';
import { formatDate } from '@/lib/utils';
import type { ApiKey } from '@/types';

export function ApiKeysPage() {
  const [showKey, setShowKey] = useState<string | null>(null);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');

  const columns: Column<ApiKey>[] = [
    { key: 'name', header: 'Name' },
    {
      key: 'key',
      header: 'API Key',
      render: (apiKey) => (
        <div className="flex items-center gap-2">
          <code className="px-2 py-1 bg-muted rounded text-sm font-mono">
            {showKey === apiKey.id ? apiKey.key : apiKey.maskedKey}
          </code>
          <Button
            variant="ghost"
            size="icon"
            onClick={(e) => {
              e.stopPropagation();
              setShowKey(showKey === apiKey.id ? null : apiKey.id);
            }}
          >
            {showKey === apiKey.id ? (
              <EyeOff className="h-4 w-4" />
            ) : (
              <Eye className="h-4 w-4" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={(e) => {
              e.stopPropagation();
              navigator.clipboard.writeText(apiKey.key);
            }}
          >
            <Copy className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (apiKey) => <StatusBadge status={apiKey.status} />,
    },
    {
      key: 'permissions',
      header: 'Permissions',
      render: (apiKey) => (
        <div className="flex gap-1">
          {apiKey.permissions.map((perm) => (
            <span
              key={perm}
              className="px-2 py-0.5 bg-secondary text-secondary-foreground text-xs rounded capitalize"
            >
              {perm}
            </span>
          ))}
        </div>
      ),
    },
    {
      key: 'lastUsedAt',
      header: 'Last Used',
      render: (apiKey) =>
        apiKey.lastUsedAt ? formatDate(apiKey.lastUsedAt, 'PP') : 'Never',
    },
    {
      key: 'createdAt',
      header: 'Created',
      render: (apiKey) => formatDate(apiKey.createdAt, 'PP'),
    },
    {
      key: 'actions',
      header: '',
      className: 'w-12',
      render: (apiKey) => (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>
              <RefreshCw className="h-4 w-4 mr-2" />
              Regenerate
            </DropdownMenuItem>
            {apiKey.status === 'active' && (
              <DropdownMenuItem className="text-destructive">
                <Trash2 className="h-4 w-4 mr-2" />
                Revoke
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      ),
    },
  ];

  return (
    <ContentContainer>
      <PageHeader
        title="API Keys"
        description="Manage your API keys for programmatic access"
        actions={
          <Button size="sm" onClick={() => setIsCreateModalOpen(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Generate Key
          </Button>
        }
      />

      <Card className="mb-6">
        <CardHeader>
          <CardTitle className="text-base">API Documentation</CardTitle>
          <CardDescription>
            Use these API keys to authenticate requests to the WhatsApp Server API.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="p-4 bg-muted rounded-lg">
            <p className="text-sm font-mono mb-2">Authorization Header:</p>
            <code className="text-sm bg-background px-3 py-2 rounded border block">
              Authorization: Bearer YOUR_API_KEY
            </code>
          </div>
        </CardContent>
      </Card>

      <DataTable
        data={apiKeys}
        columns={columns}
        emptyMessage="No API keys found. Generate a new key to get started."
      />

      <Modal
        open={isCreateModalOpen}
        onOpenChange={setIsCreateModalOpen}
        title="Generate API Key"
        description="Create a new API key for programmatic access"
        footer={
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setIsCreateModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={() => setIsCreateModalOpen(false)}>
              Generate Key
            </Button>
          </div>
        }
      >
        <div className="space-y-4">
          <InputField
            label="Key Name"
            placeholder="e.g., Production API Key"
            value={newKeyName}
            onChange={(e) => setNewKeyName(e.target.value)}
          />
          <div className="space-y-2">
            <p className="text-sm font-medium">Permissions</p>
            <div className="flex gap-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" defaultChecked className="rounded" />
                <span className="text-sm">Read</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" defaultChecked className="rounded" />
                <span className="text-sm">Write</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" className="rounded" />
                <span className="text-sm">Admin</span>
              </label>
            </div>
          </div>
        </div>
      </Modal>
    </ContentContainer>
  );
}
