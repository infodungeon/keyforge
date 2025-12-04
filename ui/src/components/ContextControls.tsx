import { useKeyboard } from "../context/KeyboardContext";
import { Select } from "./ui/Select";
import { Label } from "./ui/Label";

interface Props {
    disabled?: boolean;
}

export function ContextControls({ disabled }: Props) {
    const {
        keyboards, selectedKeyboard, selectKeyboard,
        availableLayouts, layoutName, loadLayoutPreset,
        activeJobId
    } = useKeyboard();

    // Disable if passed prop is true OR if a job is running
    const isLocked = disabled || !!activeJobId;

    return (
        <div className="p-4 border-b border-slate-800 space-y-4 bg-slate-900/50">
            <div>
                <Label>Keyboard</Label>
                <Select
                    value={selectedKeyboard}
                    onChange={e => selectKeyboard(e.target.value)}
                    options={keyboards.map(k => ({ label: k, value: k }))}
                    disabled={isLocked}
                />
            </div>
            <div>
                <Label>Layout</Label>
                <Select
                    value={availableLayouts[layoutName] ? layoutName : "Custom"}
                    onChange={e => loadLayoutPreset(e.target.value)}
                    options={[
                        { label: "Custom / Edited", value: "Custom" },
                        ...Object.keys(availableLayouts).sort().map(k => ({ label: k, value: k }))
                    ]}
                    disabled={isLocked}
                />
            </div>
        </div>
    );
}