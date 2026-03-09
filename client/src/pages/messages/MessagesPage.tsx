import { useState } from 'react';
import { 
  Send, 
  FileUp, 
  Clock, 
  Smile, 
  RefreshCw, 
  AlertCircle, 
  CheckCircle2, 
  Loader2,
  Plus,
  ArrowRight,
  RotateCw
} from 'lucide-react';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Switch } from '@/components/ui/Switch';
import { Label } from '@/components/ui/Label';
import { messageTemplates } from '@/mock';

// Mock queue data
const messageQueue = [
  { id: 1, phone: '+1 415 555 ...', preview: 'Hey, just check...', status: 'pending', time: '2m ago' },
  { id: 2, phone: '+44 20 7946...', preview: 'Your verificatio...', status: 'retry', time: '5m ago' },
  { id: 3, phone: '+91 98765 432...', preview: 'Payment confirme...', status: 'sent', time: '12m ago' },
];

// Mock templates
const savedTemplates = [
  { 
    id: 1, 
    name: 'Welcome_User_V2', 
    status: 'Approved', 
    preview: '"Hello {{name}}, welcome to our platform! We\'re excited..."',
    lastUsed: '1h ago'
  },
  { 
    id: 2, 
    name: 'Booking_Confirm', 
    status: 'Pending', 
    preview: '"Your appointment for {{service}} is confirmed for {{date}}..."',
    lastUsed: '3h ago'
  },
];

export function MessagesPage() {
  const [recipient, setRecipient] = useState('1234567890');
  const [message, setMessage] = useState('');
  const [scheduleEnabled, setScheduleEnabled] = useState(false);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <Loader2 className="h-4 w-4 text-primary animate-spin" />;
      case 'retry':
        return <AlertCircle className="h-4 w-4 text-destructive" />;
      case 'sent':
        return <CheckCircle2 className="h-4 w-4 text-success" />;
      default:
        return null;
    }
  };

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      pending: 'bg-primary/20 text-primary',
      retry: 'bg-destructive/20 text-destructive',
      sent: 'bg-success/20 text-success',
    };
    return (
      <span className={`px-2 py-0.5 text-xs font-medium rounded uppercase ${styles[status]}`}>
        {status}
      </span>
    );
  };

  const getTemplateBadge = (status: string) => {
    const styles: Record<string, string> = {
      Approved: 'bg-success/20 text-success',
      Pending: 'bg-warning/20 text-warning',
    };
    return (
      <span className={`px-2 py-0.5 text-xs font-medium rounded ${styles[status]}`}>
        {status}
      </span>
    );
  };

  return (
    <ContentContainer>
      <Tabs defaultValue="new" className="space-y-6">
        <TabsList className="bg-primary/10 border-0">
          <TabsTrigger 
            value="new" 
            className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
          >
            New Message
          </TabsTrigger>
          <TabsTrigger value="bulk">Bulk Messaging</TabsTrigger>
          <TabsTrigger value="templates">Templates</TabsTrigger>
          <TabsTrigger value="queue">Queue</TabsTrigger>
        </TabsList>

        <TabsContent value="new">
          <div className="grid gap-6 lg:grid-cols-3">
            {/* Left Panel - Compose */}
            <div className="lg:col-span-2">
              <Card className="border-2 border-primary/20">
                <CardHeader className="pb-4">
                  <CardTitle className="flex items-center gap-2 text-lg">
                    <Send className="h-5 w-5 text-primary" />
                    Compose New Message
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-6">
                  {/* Recipient */}
                  <div className="space-y-2">
                    <Label className="text-muted-foreground">Recipient Phone Number</Label>
                    <div className="relative">
                      <span className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">+</span>
                      <input
                        type="text"
                        value={recipient}
                        onChange={(e) => setRecipient(e.target.value)}
                        className="w-full pl-7 pr-4 py-3 bg-background border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary"
                        placeholder="1234567890"
                      />
                    </div>
                  </div>

                  {/* Message Content */}
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <Label className="text-muted-foreground">Message Content</Label>
                      <div className="flex items-center gap-2 text-sm">
                        <button className="text-primary hover:underline">Use Template</button>
                        <span className="text-muted-foreground">|</span>
                        <button className="text-primary hover:underline">Variables</button>
                      </div>
                    </div>
                    <div className="relative">
                      <textarea
                        value={message}
                        onChange={(e) => setMessage(e.target.value)}
                        className="w-full h-32 p-4 bg-background border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary resize-none"
                        placeholder="Type your message here..."
                      />
                      <button className="absolute bottom-3 right-3 text-muted-foreground hover:text-foreground">
                        <Smile className="h-5 w-5" />
                      </button>
                    </div>
                  </div>

                  {/* Media Upload */}
                  <div className="space-y-2">
                    <Label className="text-muted-foreground">Media & Attachments</Label>
                    <div className="border-2 border-dashed border-border rounded-lg p-8 text-center hover:border-primary/50 transition-colors cursor-pointer">
                      <FileUp className="h-10 w-10 mx-auto text-muted-foreground mb-3" />
                      <p className="text-sm font-medium">Click or drag images/docs to upload</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        Max size: 16MB. Supported: JPG, PNG, PDF, CSV
                      </p>
                    </div>
                  </div>

                  {/* Schedule Toggle */}
                  <div className="flex items-center justify-between p-4 bg-card/50 rounded-lg border border-border">
                    <div className="flex items-center gap-3">
                      <Clock className="h-5 w-5 text-primary" />
                      <div>
                        <p className="font-medium">Schedule for later</p>
                        <p className="text-sm text-muted-foreground">Pick a specific time to send this message</p>
                      </div>
                    </div>
                    <Switch checked={scheduleEnabled} onCheckedChange={setScheduleEnabled} />
                  </div>

                  {/* Action Buttons */}
                  <div className="flex gap-3">
                    <Button className="flex-1 h-12 text-base">
                      <Send className="h-4 w-4 mr-2" />
                      Send Message Now
                    </Button>
                    <Button variant="outline" className="h-12 px-6">
                      Save Draft
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Right Panel - Queue & Templates */}
            <div className="space-y-6">
              {/* Message Queue */}
              <Card>
                <CardHeader className="pb-3">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-lg">Message Queue</CardTitle>
                    <span className="flex items-center justify-center w-6 h-6 bg-primary text-primary-foreground text-xs font-bold rounded-full">
                      {messageQueue.length}
                    </span>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  {messageQueue.map((item) => (
                    <div key={item.id} className="flex items-center gap-3 p-3 bg-card/50 rounded-lg border border-border">
                      <div className="flex-shrink-0">
                        {getStatusIcon(item.status)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <p className="font-medium truncate">{item.phone}</p>
                          <span className="text-xs text-muted-foreground">{item.time}</span>
                        </div>
                        <div className="flex items-center justify-between mt-1">
                          <p className="text-sm text-muted-foreground truncate">{item.preview}</p>
                          {getStatusBadge(item.status)}
                        </div>
                      </div>
                    </div>
                  ))}
                  <button className="w-full text-center text-primary text-sm hover:underline py-2">
                    View Full Queue
                  </button>
                </CardContent>
              </Card>

              {/* Saved Templates */}
              <Card>
                <CardHeader className="pb-3">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-lg">Saved Templates</CardTitle>
                    <Button variant="ghost" size="icon" className="h-8 w-8">
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  {savedTemplates.map((template) => (
                    <div key={template.id} className="p-4 bg-card/50 rounded-lg border border-border hover:border-primary/50 transition-colors cursor-pointer group">
                      <div className="flex items-center justify-between mb-2">
                        <p className="font-medium">{template.name}</p>
                        {getTemplateBadge(template.status)}
                      </div>
                      <p className="text-sm text-muted-foreground italic line-clamp-2">
                        {template.preview}
                      </p>
                      <div className="flex items-center justify-between mt-3">
                        <span className="text-xs text-muted-foreground">
                          Last used: {template.lastUsed}
                        </span>
                        <ArrowRight className="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="bulk">
          <Card>
            <CardContent className="p-12 text-center">
              <FileUp className="h-16 w-16 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-xl font-semibold mb-2">Bulk Messaging</h3>
              <p className="text-muted-foreground mb-6">
                Upload a CSV file with phone numbers and send messages in bulk
              </p>
              <Button>
                <FileUp className="h-4 w-4 mr-2" />
                Upload CSV File
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="templates">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {messageTemplates.map((template) => (
              <Card key={template.id} className="hover:border-primary/50 transition-colors cursor-pointer">
                <CardHeader className="pb-3">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-base">{template.name}</CardTitle>
                    <span className="text-xs text-muted-foreground capitalize px-2 py-0.5 bg-muted rounded">
                      {template.category}
                    </span>
                  </div>
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground line-clamp-3">{template.content}</p>
                  <div className="flex flex-wrap gap-1 mt-3">
                    {template.variables.map((v) => (
                      <span key={v} className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded">
                        {`{{${v}}}`}
                      </span>
                    ))}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="queue">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Message Queue</CardTitle>
                <Button variant="outline" size="sm">
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Refresh
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {[...messageQueue, ...messageQueue].map((item, idx) => (
                  <div key={idx} className="flex items-center justify-between p-4 bg-card/50 rounded-lg border border-border">
                    <div className="flex items-center gap-4">
                      {getStatusIcon(item.status)}
                      <div>
                        <p className="font-medium">{item.phone}</p>
                        <p className="text-sm text-muted-foreground">{item.preview}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <span className="text-sm text-muted-foreground">{item.time}</span>
                      {getStatusBadge(item.status)}
                      {item.status === 'retry' && (
                        <Button variant="ghost" size="sm">
                          <RotateCw className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </ContentContainer>
  );
}
