/**
 * Error Boundary Component
 * Catches React component crashes and displays a friendly fallback UI
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Box, Paper, Button, Typography, Stack, Alert } from '@mui/material';
import { RefreshOutlined, BugReport } from '@mui/icons-material';

interface Props {
  children: ReactNode;
  /** Optional fallback UI */
  fallback?: ReactNode;
  /** Callback when error occurs */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary to catch and handle React component errors gracefully
 *
 * Usage:
 * ```tsx
 * <ErrorBoundary>
 *   <MyComponent />
 * </ErrorBoundary>
 * ```
 *
 * Or with custom fallback:
 * ```tsx
 * <ErrorBoundary fallback={<div>Custom error message</div>}>
 *   <MyComponent />
 * </ErrorBoundary>
 * ```
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error to console
    console.error('[ErrorBoundary] Caught error:', error, errorInfo);

    // Update state with error info
    this.setState({
      error,
      errorInfo,
    });

    // Call optional error callback
    this.props.onError?.(error, errorInfo);
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  handleReload = () => {
    window.location.reload();
  };

  handleReportBug = () => {
    const { error, errorInfo } = this.state;
    const bugReportUrl = 'https://github.com/anthropics/claude-code/issues';

    const title = `Bug: ${error?.message || 'Unknown error'}`;
    const body = `
## Error Details
**Message**: ${error?.message || 'Unknown'}
**Stack**:
\`\`\`
${error?.stack || 'No stack trace'}
\`\`\`

**Component Stack**:
\`\`\`
${errorInfo?.componentStack || 'No component stack'}
\`\`\`

## Environment
- Browser: ${navigator.userAgent}
- Timestamp: ${new Date().toISOString()}
    `.trim();

    const url = `${bugReportUrl}/new?title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
    window.open(url, '_blank');
  };

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default fallback UI
      return (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: '400px',
            p: 3,
          }}
        >
          <Paper sx={{ p: 4, maxWidth: 600 }}>
            <Stack spacing={3}>
              <Box sx={{ textAlign: 'center' }}>
                <BugReport sx={{ fontSize: 64, color: 'error.main', mb: 2 }} />
                <Typography variant="h5" gutterBottom>
                  Something went wrong
                </Typography>
                <Typography variant="body1" color="text.secondary">
                  The application encountered an unexpected error. Don't worry, your work is auto-saved.
                </Typography>
              </Box>

              <Alert severity="error" sx={{ textAlign: 'left' }}>
                <Typography variant="subtitle2" gutterBottom>
                  Error: {this.state.error?.message || 'Unknown error'}
                </Typography>
                {process.env.NODE_ENV === 'development' && this.state.error?.stack && (
                  <Box
                    component="pre"
                    sx={{
                      mt: 1,
                      p: 1,
                      bgcolor: 'rgba(0,0,0,0.1)',
                      borderRadius: 1,
                      fontSize: 11,
                      overflow: 'auto',
                      maxHeight: 200,
                    }}
                  >
                    {this.state.error.stack}
                  </Box>
                )}
              </Alert>

              <Stack direction="row" spacing={2} justifyContent="center">
                <Button
                  variant="outlined"
                  startIcon={<RefreshOutlined />}
                  onClick={this.handleReset}
                >
                  Try Again
                </Button>
                <Button
                  variant="contained"
                  startIcon={<RefreshOutlined />}
                  onClick={this.handleReload}
                >
                  Reload Page
                </Button>
                <Button
                  variant="outlined"
                  color="secondary"
                  startIcon={<BugReport />}
                  onClick={this.handleReportBug}
                >
                  Report Bug
                </Button>
              </Stack>

              <Typography variant="caption" color="text.secondary" sx={{ textAlign: 'center' }}>
                Your annotations are automatically saved every 30 seconds. You can reload the page to continue where you left off.
              </Typography>
            </Stack>
          </Paper>
        </Box>
      );
    }

    return this.props.children;
  }
}

/**
 * Hook-based error boundary wrapper for functional components
 * (Note: Error boundaries must be class components, but this provides a convenient wrapper)
 */
export const withErrorBoundary = <P extends object>(
  Component: React.ComponentType<P>,
  fallback?: ReactNode,
  onError?: (error: Error, errorInfo: ErrorInfo) => void
) => {
  return (props: P) => (
    <ErrorBoundary fallback={fallback} onError={onError}>
      <Component {...props} />
    </ErrorBoundary>
  );
};

export default ErrorBoundary;
