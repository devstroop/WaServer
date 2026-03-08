import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/Card';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: string | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = { hasError: false, error: null, errorInfo: null };

  public static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.setState({ errorInfo: errorInfo.componentStack || null });
  }

  private handleRetry = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  private handleGoHome = () => {
    window.location.href = '/dashboard';
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="w-full max-w-lg border-destructive/20 shadow-lg">
            <CardHeader className="text-center pb-2">
              <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-destructive/10">
                <AlertTriangle className="h-10 w-10 text-destructive" />
              </div>
              <h2 className="text-2xl font-bold text-foreground">
                Oops! Something went wrong
              </h2>
              <p className="text-muted-foreground mt-2 text-sm">
                We encountered an unexpected error. Don't worry, your data is safe.
              </p>
            </CardHeader>
            <CardContent className="space-y-4">
              {this.state.error && (
                <div className="rounded-lg bg-muted/50 border border-border p-4">
                  <p className="text-sm font-medium text-foreground mb-1">Error Details</p>
                  <p className="text-sm font-mono text-muted-foreground break-all">
                    {this.state.error.message}
                  </p>
                </div>
              )}
              {import.meta.env.DEV && this.state.errorInfo && (
                <details className="group">
                  <summary className="text-sm text-muted-foreground cursor-pointer hover:text-foreground transition-colors flex items-center gap-2">
                    <span className="group-open:rotate-90 transition-transform">▶</span>
                    Show stack trace
                  </summary>
                  <pre className="mt-2 overflow-auto rounded-lg bg-muted/50 border border-border p-3 text-xs font-mono text-muted-foreground max-h-48">
                    {this.state.errorInfo}
                  </pre>
                </details>
              )}
            </CardContent>
            <CardFooter className="flex gap-3 justify-center pt-2">
              <Button variant="outline" onClick={this.handleGoHome}>
                <Home className="mr-2 h-4 w-4" />
                Dashboard
              </Button>
              <Button onClick={this.handleRetry}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Try Again
              </Button>
            </CardFooter>
          </Card>
        </div>
      );
    }
    return this.props.children;
  }
}

// Functional error fallback for route errors
export function RouteErrorFallback() {
  const handleRetry = () => {
    window.location.reload();
  };

  const handleGoHome = () => {
    window.location.href = '/dashboard';
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="w-full max-w-lg border-destructive/20 shadow-lg">
        <CardHeader className="text-center pb-2">
          <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-destructive/10">
            <AlertTriangle className="h-10 w-10 text-destructive" />
          </div>
          <h2 className="text-2xl font-bold text-foreground">
            Page Error
          </h2>
          <p className="text-muted-foreground mt-2 text-sm">
            This page encountered an error. Please try again or return to the dashboard.
          </p>
        </CardHeader>
        <CardFooter className="flex gap-3 justify-center pt-4">
          <Button variant="outline" onClick={handleGoHome}>
            <Home className="mr-2 h-4 w-4" />
            Dashboard
          </Button>
          <Button onClick={handleRetry}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Reload Page
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
