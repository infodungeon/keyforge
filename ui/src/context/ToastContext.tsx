import { createContext, useContext, useState, useCallback, ReactNode } from "react";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
    id: string;
    type: ToastType;
    message: string;
    duration?: number;
}

interface ToastContextType {
    addToast: (type: ToastType, message: string, duration?: number) => void;
    removeToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({ children }: { children: ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([]);

    const removeToast = useCallback((id: string) => {
        setToasts(prev => prev.filter(t => t.id !== id));
    }, []);

    const addToast = useCallback((type: ToastType, message: string, duration = 5000) => {
        const id = Math.random().toString(36).substring(2, 9);
        setToasts(prev => [...prev, { id, type, message, duration }]);

        if (duration > 0) {
            setTimeout(() => removeToast(id), duration);
        }
    }, [removeToast]);

    return (
        <ToastContext.Provider value={{ addToast, removeToast }}>
            {children}
            <div className="fixed bottom-10 right-6 z-50 flex flex-col gap-2 pointer-events-none">
                {toasts.map(toast => (
                    <ToastItem key={toast.id} toast={toast} onDismiss={removeToast} />
                ))}
            </div>
        </ToastContext.Provider>
    );
}

function ToastItem({ toast, onDismiss }: { toast: Toast, onDismiss: (id: string) => void }) {
    const icons = {
        success: <CheckCircle size={16} className="text-green-400" />,
        error: <AlertCircle size={16} className="text-red-400" />,
        warning: <AlertTriangle size={16} className="text-yellow-400" />,
        info: <Info size={16} className="text-blue-400" />
    };

    const borders = {
        success: "border-green-500/20 bg-slate-900/90",
        error: "border-red-500/20 bg-slate-900/90",
        warning: "border-yellow-500/20 bg-slate-900/90",
        info: "border-blue-500/20 bg-slate-900/90"
    };

    return (
        <div className={`
            pointer-events-auto flex items-start gap-3 p-3 rounded-lg border shadow-xl backdrop-blur-md w-80 animate-in slide-in-from-right-10 fade-in duration-300
            ${borders[toast.type]}
        `}>
            <div className="mt-0.5 shrink-0">{icons[toast.type]}</div>
            <div className="flex-1 text-xs text-slate-200 leading-relaxed font-medium break-words">
                {toast.message}
            </div>
            <button 
                onClick={() => onDismiss(toast.id)}
                className="text-slate-500 hover:text-white transition-colors"
            >
                <X size={14} />
            </button>
        </div>
    );
}

export const useToast = () => {
    const ctx = useContext(ToastContext);
    if (!ctx) throw new Error("useToast must be used within ToastProvider");
    return ctx;
};