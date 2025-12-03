interface NetworkProps {
  hiveUrl: string;
  setHiveUrl: (url: string) => void;
  status: 'disconnected' | 'connected' | 'error';
}

export function NetworkBar({ hiveUrl, setHiveUrl, status }: NetworkProps) {
  return (
    <div className="bg-slate-950 border-b border-slate-800 p-2 flex items-center justify-between text-xs px-4">
      <div className="flex items-center gap-4">
        <span className="font-bold text-slate-500">HIVE SERVER</span>
        <input 
          className="bg-slate-800 border border-slate-700 rounded px-2 py-1 text-slate-300 w-64 font-mono outline-none focus:border-blue-500"
          value={hiveUrl}
          onChange={(e) => setHiveUrl(e.target.value)}
          placeholder="http://localhost:3000"
        />
      </div>
      
      <div className="flex items-center gap-2">
        <span className="uppercase font-bold text-slate-500 text-[10px] tracking-wider">Status:</span>
        <div className={`w-2 h-2 rounded-full ${status === 'connected' ? 'bg-green-500' : 'bg-red-500'}`} />
        <span className={`uppercase font-bold ${status === 'connected' ? 'text-green-500' : 'text-slate-400'}`}>
          {status}
        </span>
      </div>
    </div>
  );
}