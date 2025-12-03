import { HTMLAttributes } from "react";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
    noPadding?: boolean;
}

export function Card({ className = "", noPadding = false, children, ...props }: CardProps) {
    return (
        <div 
            className={`bg-slate-900 border border-slate-800 rounded-xl overflow-hidden ${noPadding ? '' : 'p-4'} ${className}`}
            {...props}
        >
            {children}
        </div>
    );
}