import React from "react";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { Button } from "@/components/ui/button";

interface Props {
  children: React.ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("UI error:", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div className={cn("flex h-screen items-center justify-center p-6", text.body)}>
          <div className={cn(glass.panelStrong, "max-w-md space-y-4 p-8 text-center")}>
            <h2 className={cn("text-lg font-semibold", text.primary)}>界面出现异常</h2>
            <p className={cn("text-sm", text.muted)}>{this.state.error.message}</p>
            <Button onClick={() => window.location.reload()}>重新加载</Button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
