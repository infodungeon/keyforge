import { LabelHTMLAttributes } from "react";

export function Label({ className = "", children, ...props }: LabelHTMLAttributes<HTMLLabelElement>) {
    return (
        <label 
            className={`block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1.5 ${className}`}
            {...props}
        >
            {children}
        </label>
    );
}