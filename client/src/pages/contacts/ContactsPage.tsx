import { useState } from 'react';
import { Plus, Upload, Download, MoreVertical, Edit, Trash2, Tag, MessageSquare } from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { DataTable, type Column } from '@/components/tables/DataTable';
import { StatusBadge } from '@/components/badges/StatusBadge';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/DropdownMenu';
import { contacts, contactTags } from '@/mock';
import { formatDate, formatPhoneNumber } from '@/lib/utils';
import type { Contact } from '@/types';

const contactColumns: Column<Contact>[] = [
  {
    key: 'name',
    header: 'Name',
    render: (contact) => (
      <div>
        <p className="font-medium">{contact.name}</p>
        <p className="text-xs text-muted-foreground">
          {formatPhoneNumber(contact.phoneNumber)}
        </p>
      </div>
    ),
  },
  {
    key: 'email',
    header: 'Email',
    render: (contact) => contact.email || '-',
  },
  {
    key: 'tags',
    header: 'Tags',
    render: (contact) => (
      <div className="flex flex-wrap gap-1">
        {contact.tags.slice(0, 3).map((tag) => (
          <Badge key={tag} variant="secondary" className="text-xs">
            {tag}
          </Badge>
        ))}
        {contact.tags.length > 3 && (
          <Badge variant="outline" className="text-xs">
            +{contact.tags.length - 3}
          </Badge>
        )}
      </div>
    ),
  },
  {
    key: 'status',
    header: 'Status',
    render: (contact) => <StatusBadge status={contact.status} />,
  },
  {
    key: 'messagesCount',
    header: 'Messages',
    render: (contact) => contact.messagesCount.toLocaleString(),
  },
  {
    key: 'lastMessageAt',
    header: 'Last Message',
    render: (contact) =>
      contact.lastMessageAt ? formatDate(contact.lastMessageAt, 'PP') : '-',
  },
  {
    key: 'actions',
    header: '',
    className: 'w-12',
    render: () => (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon">
            <MoreVertical className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem>
            <MessageSquare className="h-4 w-4 mr-2" />
            Send Message
          </DropdownMenuItem>
          <DropdownMenuItem>
            <Edit className="h-4 w-4 mr-2" />
            Edit
          </DropdownMenuItem>
          <DropdownMenuItem>
            <Tag className="h-4 w-4 mr-2" />
            Manage Tags
          </DropdownMenuItem>
          <DropdownMenuItem className="text-destructive">
            <Trash2 className="h-4 w-4 mr-2" />
            Delete
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    ),
  },
];

export function ContactsPage() {
  const [selectedTags, setSelectedTags] = useState<string[]>([]);

  const filteredContacts =
    selectedTags.length > 0
      ? contacts.filter((contact) =>
          selectedTags.some((tag) => contact.tags.includes(tag))
        )
      : contacts;

  const toggleTag = (tagName: string) => {
    setSelectedTags((prev) =>
      prev.includes(tagName)
        ? prev.filter((t) => t !== tagName)
        : [...prev, tagName]
    );
  };

  return (
    <ContentContainer>
      <PageHeader
        title="Contacts"
        description="Manage your contact list and tags"
        actions={
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm">
              <Upload className="h-4 w-4 mr-2" />
              Import
            </Button>
            <Button variant="outline" size="sm">
              <Download className="h-4 w-4 mr-2" />
              Export
            </Button>
            <Button size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Add Contact
            </Button>
          </div>
        }
      />

      <div className="grid gap-6 lg:grid-cols-4 mb-6">
        <div className="lg:col-span-3">
          <Card className="mb-4">
            <CardContent className="p-4">
              <div className="flex flex-wrap gap-2">
                <span className="text-sm text-muted-foreground mr-2">Filter by tags:</span>
                {contactTags.map((tag) => (
                  <button
                    key={tag.id}
                    onClick={() => toggleTag(tag.name)}
                    className={`px-2 py-1 text-xs rounded-full border transition-colors ${
                      selectedTags.includes(tag.name)
                        ? 'bg-primary text-primary-foreground border-primary'
                        : 'bg-secondary text-secondary-foreground border-border hover:bg-accent'
                    }`}
                  >
                    {tag.name} ({tag.contactCount})
                  </button>
                ))}
                {selectedTags.length > 0 && (
                  <button
                    onClick={() => setSelectedTags([])}
                    className="px-2 py-1 text-xs text-muted-foreground hover:text-foreground"
                  >
                    Clear all
                  </button>
                )}
              </div>
            </CardContent>
          </Card>

          <DataTable
            data={filteredContacts}
            columns={contactColumns}
            emptyMessage="No contacts found."
          />
        </div>

        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Tags</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {contactTags.map((tag) => (
                <div
                  key={tag.id}
                  className="flex items-center justify-between p-2 rounded-lg hover:bg-accent cursor-pointer"
                  onClick={() => toggleTag(tag.name)}
                >
                  <div className="flex items-center gap-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: tag.color }}
                    />
                    <span className="text-sm">{tag.name}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {tag.contactCount}
                  </span>
                </div>
              ))}
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Quick Stats</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Total Contacts</span>
                <span className="font-medium">{contacts.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Active</span>
                <span className="font-medium">
                  {contacts.filter((c) => c.status === 'active').length}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-muted-foreground">Blocked</span>
                <span className="font-medium">
                  {contacts.filter((c) => c.status === 'blocked').length}
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </ContentContainer>
  );
}
