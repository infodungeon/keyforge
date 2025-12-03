import { ButtonHTMLAttributes, ReactNode } from "react";
import { Loader2 } from "lucide-react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'optimize';
    size?: 'sm' | 'md' | 'lg' | 'icon';
    isLoading?: boolean;
    icon?: ReactNode;
    children?: ReactNode;
}

export function Button({ 
    variant = 'primary', 
    size = 'md', 
    isLoading = false, 
    icon, 
    children, 
    className = "", 
    disabled,
    ...props 
}: ButtonProps) {
    
    const baseStyles = "inline-flex items-center justify-center font-bold rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:opacity-50 disabled:cursor-not-allowed";
    
    const variants = {
        primary: "bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/20 border border-blue-500",
        secondary: "bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 hover:border-slate-600",
        danger: "bg-red-950/30 hover:bg-red-900/50 text-red-400 border border-red-900/50 hover:border-red-800",
        ghost: "bg-transparent hover:bg-slate-800 text-slate-500 hover:text-slate-200",
        optimize: "bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-500 hover:to-blue-500 text-white shadow-lg shadow-purple-900/20 border-0"
    };

    const sizes = {
        sm: "text-[10px] px-2 py-1 gap-1.5 h-7",
        md: "text-xs px-4 py-2 gap-2 h-9",
        lg: "text-sm px-6 py-3 gap-2.5 h-12",
        icon: "p-2 h-9 w-9"
    };

    return (
        <button 
            className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
            disabled={disabled || isLoading}
            {...props}
        >
            {isLoading && <Loader2 className="animate-spin" size={size === 'sm' ? 12 : 16} />}
            {!isLoading && icon && <span className="shrink-0">{icon}</span>}
            {children}
        </button>
    );
}