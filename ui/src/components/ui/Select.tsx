import { SelectHTMLAttributes, forwardRef } from "react";
import { ChevronDown } from "lucide-react";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
    options: { label: string; value: string }[];
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
    ({ className = "", options, ...props }, ref) => {
        return (
            <div className="relative">
                <select
                    ref={ref}
                    className={`
                        w-full appearance-none bg-slate-950/50 border border-slate-800 
                        rounded-lg px-3 py-2 pr-8 text-xs text-slate-200 outline-none 
                        transition-all cursor-pointer
                        hover:border-slate-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20
                        ${className}
                    `}
                    {...props}
                >
                    {options.map((opt) => (
                        <option 
                            key={opt.value} 
                            value={opt.value}
                            className="bg-slate-900 text-slate-200 py-1"
                        >
                            {opt.label}
                        </option>
                    ))}
                </select>
                <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-slate-500">
                    <ChevronDown size={14} />
                </div>
            </div>
        );
    }
);

Select.displayName = "Select";