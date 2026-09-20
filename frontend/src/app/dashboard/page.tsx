export default function DashboardPage() {
  return (
    <div className="flex flex-col gap-6">
      <h1 className="text-3xl font-heading font-bold text-foreground">Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="p-6 bg-card rounded-lg border border-border">
          <h2 className="text-xl font-semibold mb-2">Connected Accounts</h2>
          <p className="text-muted-foreground">Manage your Twitch, YouTube, and Kick integrations.</p>
        </div>
        <div className="p-6 bg-card rounded-lg border border-border">
          <h2 className="text-xl font-semibold mb-2">Widget Settings</h2>
          <p className="text-muted-foreground">Configure how your chat overlay looks on OBS.</p>
        </div>
      </div>
    </div>
  );
}
