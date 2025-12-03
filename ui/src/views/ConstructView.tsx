import { KeyboardDesigner } from "../components/KeyboardDesigner";
import { useKeyboard } from "../context/KeyboardContext";

export function ConstructView() {
    const { refreshData } = useKeyboard();
    // Designer needs full width, no sidebars
    return (
        <div className="flex-1 min-w-0 bg-[#0B0F19]">
            <KeyboardDesigner onSaveSuccess={refreshData} />
        </div>
    );
}