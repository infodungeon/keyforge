import { Select } from "./ui/Select";
import { Label } from "./ui/Label";

interface Props {
    keyboards: string[];
    selectedKeyboard: string;
    onSelectKeyboard: (k: string) => void;
    
    availableLayouts: Record<string, string>;
    layoutName: string;
    onSelectLayout: (n: string) => void;
    
    disabled?: boolean;
}

export function ContextControls({
    keyboards, selectedKeyboard, onSelectKeyboard,
    availableLayouts, layoutName, onSelectLayout,
    disabled
}: Props) {
    return (
        <div className="p-4 border-b border-slate-800 space-y-4 bg-slate-900/50">
            <div>
                <Label>Keyboard</Label>
                <Select 
                    value={selectedKeyboard} 
                    onChange={e => onSelectKeyboard(e.target.value)}
                    options={keyboards.map(k => ({ label: k, value: k }))}
                    disabled={disabled}
                />
            </div>
            <div>
                <Label>Layout</Label>
                <Select
                    value={availableLayouts[layoutName] ? layoutName : "Custom"}
                    onChange={e => onSelectLayout(e.target.value)}
                    options={[
                        { label: "Custom / Edited", value: "Custom" },
                        ...Object.keys(availableLayouts).sort().map(k => ({ label: k, value: k }))
                    ]}
                    disabled={disabled}
                />
            </div>
        </div>
    );
}