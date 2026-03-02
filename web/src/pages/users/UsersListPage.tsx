import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  Users as UsersIcon, 
  Plus, 
  Search, 
  Trash2,
  Shield,
  User as UserIcon
} from 'lucide-react';
import { Header } from '@/components/layout';
import { Card, CardContent, Input, Button, Badge, SkeletonTable } from '@/components/ui';
import { usersApi, type User } from '@/api/users';

export function UsersListPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      const data = await usersApi.listUsers();
      setUsers(data);
    } catch (error) {
      console.error('Failed to load users:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string, username: string) => {
    if (!confirm(`Are you sure you want to delete user "${username}"?`)) {
      return;
    }

    try {
      await usersApi.deleteUser(id);
      setUsers(users.filter(u => u.id !== id));
    } catch (error) {
      console.error('Failed to delete user:', error);
    }
  };

  const filteredUsers = users.filter(user =>
    user.username.toLowerCase().includes(search.toLowerCase()) ||
    (user.email && user.email.toLowerCase().includes(search.toLowerCase()))
  );

  return (
    <>
      <Header 
        title="Users" 
        description="Manage user accounts and permissions"
        actions={
          <Button asChild>
            <Link to="/users/new">
              <Plus className="h-4 w-4 mr-2" />
              New User
            </Link>
          </Button>
        }
      />

      <div className="p-6">
        <Card>
          <CardContent className="p-4">
            {/* Search */}
            <div className="mb-4">
              <Input
                placeholder="Search users..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                leftIcon={<Search className="h-4 w-4" />}
              />
            </div>

            {/* Table */}
            {loading ? (
              <SkeletonTable rows={5} />
            ) : filteredUsers.length === 0 ? (
              <div className="text-center py-12">
                <UsersIcon className="h-12 w-12 mx-auto text-text-muted-light dark:text-text-muted-dark mb-3" />
                <p className="text-text-muted-light dark:text-text-muted-dark mb-3">
                  {search ? 'No users found matching your search' : 'No users yet'}
                </p>
                {!search && (
                  <Button asChild>
                    <Link to="/users/new">
                      <Plus className="h-4 w-4 mr-2" />
                      Create User
                    </Link>
                  </Button>
                )}
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-border-light dark:border-border-dark">
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        User
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Role
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Status
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Created
                      </th>
                      <th className="text-right py-3 px-4 font-medium text-text-muted-light dark:text-text-muted-dark">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredUsers.map((user) => (
                      <tr 
                        key={user.username}
                        className="border-b border-border-light dark:border-border-dark last:border-0 hover:bg-bg-surface-light dark:hover:bg-bg-surface-dark"
                      >
                        <td className="py-3 px-4">
                          <div className="flex items-center gap-3">
                            <div className="h-10 w-10 rounded-full bg-primary-500/10 flex items-center justify-center">
                              {user.role === 'admin' ? (
                                <Shield className="h-5 w-5 text-primary-500" />
                              ) : (
                                <UserIcon className="h-5 w-5 text-primary-500" />
                              )}
                            </div>
                            <span className="font-medium text-text-light dark:text-text-dark">
                              {user.username}
                            </span>
                          </div>
                        </td>
                        <td className="py-3 px-4">
                          <Badge variant={user.role === 'admin' ? 'primary' : 'default'}>
                            {user.role}
                          </Badge>
                        </td>
                        <td className="py-3 px-4">
                          <Badge variant={user.is_active ? 'success' : 'error'}>
                            {user.is_active ? 'Active' : 'Inactive'}
                          </Badge>
                        </td>
                        <td className="py-3 px-4 text-text-muted-light dark:text-text-muted-dark">
                          {user.created_at ? new Date(user.created_at).toLocaleDateString() : '—'}
                        </td>
                        <td className="py-3 px-4">
                          <div className="flex items-center justify-end gap-1">
                            <Button 
                              variant="ghost" 
                              size="icon"
                              onClick={() => handleDelete(user.id, user.username)}
                              title="Delete User"
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
