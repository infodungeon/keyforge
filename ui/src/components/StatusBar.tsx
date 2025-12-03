import { Wifi, WifiOff, Activity, Server, Database } from "lucide-react"; // CLEANED IMPORTS

interface Props {
  hiveUrl: string;
  hiveStatus: 'connected' | 'disconnected' | 'error';
  localWorkerEnabled: boolean;
  isSyncing: boolean;
  activeJobId: string | null;
}

export function StatusBar({ hiveUrl, hiveStatus, localWorkerEnabled, isSyncing, activeJobId }: Props) {

  // Parse simple hostname for display
  const displayHost = hiveUrl.replace(/^https?:\/\//, '').replace(/\/$/, '');

  return (
    <div className="h-8 bg-slate-950 border-t border-slate-800 flex items-center px-4 justify-between text-[10px] text-slate-500 shrink-0 select-none">

      {/* Left: Connection Status */}
      <div className="flex items-center gap-4">
        <div className={`flex items-center gap-1.5 ${hiveStatus === 'connected' ? "text-green-400" :
            hiveStatus === 'error' ? "text-red-400" : "text-slate-500"
          }`}>
          {hiveStatus === 'connected' ? <Wifi size={12} /> : <WifiOff size={12} />}
          <span className="font-bold tracking-wide">
            {isSyncing ? "SYNCING..." : hiveStatus === 'connected' ? "HIVE ONLINE" : "OFFLINE"}
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          <Server size={12} />
          <span>{displayHost}</span>
        </div>
      </div>

      {/* Center: Active Activity */}
      <div className="flex items-center gap-2">
        {activeJobId && (
          <>
            <Activity size={12} className="animate-pulse text-purple-400" />
            <span className="text-purple-400 font-mono">JOB: {activeJobId.substring(0, 8)}</span>
          </>
        )}
      </div>

      {/* Right: Local Resources */}
      <div className="flex items-center gap-4">
        {localWorkerEnabled && (
          <div className="flex items-center gap-1.5 text-blue-400">
            <Database size={12} />
            <span>LOCAL WORKER</span>
          </div>
        )}
        <div>v0.7.0</div>
      </div>
    </div>
  );
}