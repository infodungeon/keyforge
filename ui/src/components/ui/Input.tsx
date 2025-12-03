import { InputHTMLAttributes, forwardRef } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
    mono?: boolean;
    error?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
    ({ className = "", mono = false, error = false, ...props }, ref) => {
        return (
            <input
                ref={ref}
                className={`
                    w-full bg-slate-950/50 border rounded-lg px-3 py-2 text-xs text-slate-200 
                    placeholder:text-slate-600 outline-none transition-all
                    focus:bg-slate-900 focus:ring-1
                    ${mono ? "font-mono tracking-wide" : "font-sans"}
                    ${error 
                        ? "border-red-900/50 focus:border-red-500 focus:ring-red-500/20" 
                        : "border-slate-800 focus:border-blue-500 focus:ring-blue-500/20"}
                    ${className}
                `}
                {...props}
            />
        );
    }
);

Input.displayName = "Input";