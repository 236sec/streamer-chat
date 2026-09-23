"use client";

import React, { useEffect, useState } from "react";
import { X, Copy, Check, MonitorPlay } from "lucide-react";
import { Button } from "@/components/ui/button";

import { Input } from "@/components/ui/input";
import { useToast } from "@/components/ui/toast";

interface ObsSetupGuideModalProps {
  isOpen: boolean;
  onClose: () => void;
  widgetUrl?: string;
}

export function ObsSetupGuideModal({
  isOpen,
  onClose,
  widgetUrl = "",
}: ObsSetupGuideModalProps) {
  const { success } = useToast();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleCopy = async () => {
    if (!widgetUrl) return;
    try {
      await navigator.clipboard.writeText(widgetUrl);
      setCopied(true);
      success("Widget URL copied to clipboard");
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
    }
  };

  const steps = [
    {
      num: 1,
      title: "Add Browser Source in OBS",
      description:
        "In OBS Studio, find the Sources panel, click the + button, and select Browser.",
    },
    {
      num: 2,
      title: "Name the Source",
      description:
        'Enter a recognizable name such as "StreamSync Chat" and click OK.',
    },
    {
      num: 3,
      title: "Paste Widget URL",
      description: "Paste your widget overlay URL into the URL field in OBS.",
      hasUrl: true,
    },
    {
      num: 4,
      title: "Set Dimensions",
      description:
        "Set Width to 400 (or your preferred width) and Height to 600 (or custom height).",
    },
    {
      num: 5,
      title: "Enable Performance Settings",
      description:
        'Check both "Shutdown source when not visible" and "Refresh browser when scene becomes active" for optimal performance.',
    },
  ];

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="obs-guide-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="bg-card text-card-foreground border border-border rounded-lg shadow-xl w-full max-w-xl max-h-[90vh] flex flex-col overflow-hidden animate-in fade-in-0 zoom-in-95 duration-150">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-border">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-md bg-secondary text-primary">
              <MonitorPlay className="h-5 w-5" />
            </div>
            <div>
              <h2 id="obs-guide-title" className="text-xl font-semibold">
                OBS Setup Guide
              </h2>
              <p className="text-sm text-muted-foreground">
                Step-by-step instructions to embed chat into OBS Studio
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close dialog"
            className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto space-y-5">
          <ol className="space-y-4">
            {steps.map((step) => (
              <li
                key={step.num}
                className="flex items-start gap-4 p-3.5 rounded-md bg-background/50 border border-border/60"
              >
                <span className="flex items-center justify-center shrink-0 w-7 h-7 rounded-full bg-primary/20 text-primary text-sm font-bold">
                  {step.num}
                </span>
                <div className="flex-1 min-w-0 space-y-1">
                  <div className="text-sm font-semibold text-foreground">
                    {step.title}
                  </div>
                  <div className="text-xs text-muted-foreground leading-relaxed">
                    {step.description}
                  </div>
                  {step.hasUrl && widgetUrl && (
                    <div className="mt-2.5 flex items-center gap-2">
                      <Input
                        readOnly
                        value={widgetUrl}
                        className="font-mono text-xs h-8 bg-background"
                      />
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={handleCopy}
                        className="h-8 shrink-0 px-3 text-xs"
                      >
                        {copied ? (
                          <>
                            <Check className="h-3.5 w-3.5 mr-1 text-primary" />
                            Copied
                          </>
                        ) : (
                          <>
                            <Copy className="h-3.5 w-3.5 mr-1" />
                            Copy
                          </>
                        )}
                      </Button>
                    </div>
                  )}
                </div>
              </li>
            ))}
          </ol>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-4 border-t border-border bg-card">
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </div>
  );
}
