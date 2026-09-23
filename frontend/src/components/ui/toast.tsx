"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import { CheckCircle2, AlertCircle, Info, X } from "lucide-react";
import { cn } from "@/lib/utils";


export type ToastType = "success" | "error" | "info";

export interface ToastItem {
  id: string;
  type: ToastType;
  title?: string;
  message: string;
}

interface ToastOptions {
  message: string;
  title?: string;
  type?: ToastType;
  duration?: number;
}

interface ToastContextValue {
  toast: (options: ToastOptions) => string;
  success: (message: string, title?: string) => string;
  error: (message: string, title?: string) => string;
  info: (message: string, title?: string) => string;
  dismiss: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const toast = useCallback(
    ({ message, title, type = "info", duration = 4000 }: ToastOptions) => {
      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
      const newToast: ToastItem = { id, type, title, message };

      setToasts((prev) => [...prev, newToast]);

      if (duration > 0) {
        setTimeout(() => {
          dismiss(id);
        }, duration);
      }

      return id;
    },
    [dismiss]
  );

  const success = useCallback(
    (message: string, title?: string) => toast({ message, title, type: "success" }),
    [toast]
  );

  const error = useCallback(
    (message: string, title?: string) => toast({ message, title, type: "error" }),
    [toast]
  );

  const info = useCallback(
    (message: string, title?: string) => toast({ message, title, type: "info" }),
    [toast]
  );

  return (
    <ToastContext.Provider value={{ toast, success, error, info, dismiss }}>
      {children}
      <div
        aria-live="polite"
        aria-atomic="true"
        className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full pointer-events-none p-2 sm:p-0"
      >
        {toasts.map((t) => (
          <div
            key={t.id}
            role="alert"
            className={cn(
              "pointer-events-auto flex items-start gap-3 p-4 rounded-lg border bg-card text-card-foreground shadow-lg transition-all duration-200",
              t.type === "error" && "border-destructive/60",
              t.type === "success" && "border-primary/60",
              t.type === "info" && "border-border"
            )}
          >
            <div className="shrink-0 mt-0.5">
              {t.type === "success" && <CheckCircle2 className="h-5 w-5 text-primary" />}
              {t.type === "error" && <AlertCircle className="h-5 w-5 text-destructive" />}
              {t.type === "info" && <Info className="h-5 w-5 text-muted-foreground" />}
            </div>
            <div className="flex-1 text-sm min-w-0">
              {t.title && <div className="font-semibold text-foreground mb-0.5">{t.title}</div>}
              <div className="text-muted-foreground break-words leading-relaxed">{t.message}</div>
            </div>
            <button
              type="button"
              onClick={() => dismiss(t.id)}
              aria-label="Dismiss toast"
              className="shrink-0 rounded p-1 text-muted-foreground hover:text-foreground transition-colors"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}
