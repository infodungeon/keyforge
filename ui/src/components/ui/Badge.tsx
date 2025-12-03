import { HTMLAttributes } from "react";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
    variant?: 'default' | 'success' | 'warning' | 'neutral';
}

export function Badge({ variant = 'default', className = "", children, ...props }: BadgeProps) {
    const variants = {
        default: "bg-blue-900/30 text-blue-400 border-blue-900/50",
        success: "bg-green-900/30 text-green-400 border-green-900/50",
        warning: "bg-yellow-900/30 text-yellow-400 border-yellow-900/50",
        neutral: "bg-slate-800 text-slate-400 border-slate-700",
    };

    return (
        <span 
            className={`
                inline-flex items-center px-1.5 py-0.5 rounded text-[9px] font-bold uppercase tracking-wider border
                ${variants[variant]} ${className}
            `}
            {...props}
        >
            {children}
        </span>
    );
}