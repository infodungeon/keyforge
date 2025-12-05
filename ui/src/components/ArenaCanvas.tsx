// ===== keyforge/ui/src/components/ArenaCanvas.tsx =====
import { RotateCcw } from "lucide-react";
import { useArena, ZOOM_LEVELS } from "../context/ArenaContext";

export function ArenaCanvas() {
    const { 
        words, input, currentIndex, 
        isFinished, isLoading, 
        inputRef, 
        handleKeyDown, handleChange, nextSession,
        zoomIndex
    } = useArena();

    const renderWord = (word: string, idx: number) => {
        const isCurrent = idx === currentIndex;
        const sizeClass = ZOOM_LEVELS[zoomIndex];
        
        // Base styling for the word
        let className = `${sizeClass} font-mono tracking-wide px-1.5 py-1 my-1 mr-2 inline-block transition-colors `;
        
        if (idx < currentIndex) className += "text-slate-600"; // Passed
        else if (isCurrent) className += "text-white";         // Active
        else className += "text-slate-700";                    // Future

        if (isCurrent) {
            return (
                <span key={idx} className={className}>
                    {word.split('').map((char, charIdx) => {
                        let charClass = "";
                        // Character coloring logic
                        if (charIdx < input.length) {
                            charClass = input[charIdx] === char 
                                ? "text-slate-300" // Correct
                                : "text-red-400 border-b-2 border-red-500"; // Incorrect
                        }
                        const isCursor = charIdx === input.length;
                        
                        return (
                            <span key={charIdx} className={`relative ${charClass}`}>
                                {isCursor && (
                                    <span className="absolute -left-[1px] -top-1 bottom-0 w-[2px] bg-purple-500 animate-pulse" />
                                )}
                                {char}
                            </span>
                        );
                    })}
                    
                    {/* Trailing cursor (at end of word) */}
                    {input.length >= word.length && (
                        <span className="inline-block w-[2px] h-[1em] bg-purple-500 animate-pulse align-middle ml-[1px]" />
                    )}
                    
                    {/* Extra characters typed beyond word length */}
                    {input.length > word.length && (
                        <span className="text-red-800 opacity-50">{input.slice(word.length)}</span>
                    )}
                </span>
            );
        }
        return <span key={idx} className={className}>{word}</span>;
    };

    return (
        <div
            className="flex-1 flex flex-col items-center justify-start bg-[#0B0F19] relative overflow-hidden"
            onClick={() => inputRef.current?.focus()}
        >
            {/* Invisible Input to capture keystrokes */}
            <input
                ref={inputRef}
                className="absolute opacity-0 pointer-events-none"
                value={input}
                onChange={handleChange}
                onKeyDown={handleKeyDown}
                autoFocus
                onBlur={() => { 
                    if (!isFinished) setTimeout(() => inputRef.current?.focus(), 10); 
                }}
            />

            <div className="flex-1 w-full max-w-5xl p-12 overflow-y-auto custom-scrollbar">
                {isLoading ? (
                    <div className="text-slate-500 animate-pulse text-center mt-20">
                        Initializing Corpus...
                    </div>
                ) : isFinished ? (
                    <div className="flex flex-col items-center justify-center h-full gap-6 animate-in fade-in zoom-in duration-300 opacity-50">
                        <div className="text-4xl text-slate-700 font-bold mb-4">Session Complete</div>
                        <button 
                            onClick={nextSession} 
                            className="p-4 bg-slate-800 hover:bg-slate-700 rounded-full text-slate-400 transition-colors shadow-lg"
                        >
                            <RotateCcw size={32} />
                        </button>
                        <p className="text-sm text-slate-500">Check the sidebar for detailed stats.</p>
                    </div>
                ) : (
                    <div className="flex flex-wrap justify-start content-start text-left cursor-text select-text">
                        {words.map((w, i) => renderWord(w, i))}
                    </div>
                )}
            </div>
        </div>
    );
}