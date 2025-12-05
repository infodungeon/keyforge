// ===== keyforge/ui/src/views/ArenaView.tsx =====
import { ArenaCanvas } from "../components/ArenaCanvas";
import { Inspector } from "../components/Inspector";

export function ArenaView() {
    return (
        <>
            {/* Left/Center: The main interaction area */}
            <ArenaCanvas />

            {/* Right: The control and settings panel */}
            <Inspector mode="arena" />
        </>
    );
}