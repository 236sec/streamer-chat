"use client";

import { useState } from "react";
import { WidgetClient } from "@/components/widget/WidgetClient";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function WidgetSettingsPage() {
  const [theme, setTheme] = useState("dark");
  const [fontSize, setFontSize] = useState("16px");
  const [backgroundColor, setBackgroundColor] = useState("transparent");

  const widgetUrl = `http://localhost:3000/widget/test-user-id?theme=${theme}&fontSize=${encodeURIComponent(fontSize)}&backgroundColor=${encodeURIComponent(backgroundColor)}`;

  return (
    <div className="flex flex-col gap-6 h-full">
      <h1 className="text-3xl font-heading font-bold text-foreground">Widget Settings</h1>
      
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
        {/* Settings Panel */}
        <div className="p-6 bg-card rounded-lg border border-border flex flex-col gap-6 lg:col-span-1 overflow-y-auto">
          <div>
            <h2 className="text-xl font-semibold mb-2">Appearance</h2>
            <p className="text-muted-foreground text-sm mb-4">Configure how your chat overlay looks on OBS.</p>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="theme">Theme</Label>
              <select
                id="theme"
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="fontSize">Font Size</Label>
              <Input 
                id="fontSize" 
                value={fontSize} 
                onChange={(e) => setFontSize(e.target.value)} 
                placeholder="e.g. 16px, 1.2rem"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="bgColor">Background Color</Label>
              <Input 
                id="bgColor" 
                value={backgroundColor} 
                onChange={(e) => setBackgroundColor(e.target.value)} 
                placeholder="transparent or hex code"
              />
            </div>
          </div>

          <div className="mt-8">
            <h2 className="text-xl font-semibold mb-2">Widget URL</h2>
            <p className="text-muted-foreground text-sm mb-4">Copy this URL and add it as a Browser Source in OBS.</p>
            <div className="flex gap-2">
              <Input readOnly value={widgetUrl} className="font-mono text-xs" />
              <Button onClick={() => navigator.clipboard.writeText(widgetUrl)}>
                Copy
              </Button>
            </div>
          </div>
        </div>

        {/* Preview Panel */}
        <div className="p-6 bg-card rounded-lg border border-border lg:col-span-2 flex flex-col min-h-[500px]">
          <h2 className="text-xl font-semibold mb-4">Live Preview</h2>
          <div className="flex-1 rounded-md border border-dashed border-border overflow-hidden relative bg-black/5 flex items-center justify-center">
            {/* Checkerboard background for transparent preview */}
            <div 
              className="absolute inset-0 z-0 opacity-10" 
              style={{
                backgroundImage: 'linear-gradient(45deg, #808080 25%, transparent 25%), linear-gradient(-45deg, #808080 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #808080 75%), linear-gradient(-45deg, transparent 75%, #808080 75%)',
                backgroundSize: '20px 20px',
                backgroundPosition: '0 0, 0 10px, 10px -10px, -10px 0px'
              }}
            />
            <div className="relative z-10 w-full h-full max-w-md mx-auto shadow-2xl border border-border/50">
              <WidgetClient 
                widgetId="test-user-id"
                theme={theme}
                fontSize={fontSize}
                backgroundColor={backgroundColor}
                mock={true}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
