// ===== keyforge/ui/src/components/modals/ExportModal.tsx =====
import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { useBackend } from "../../context/BackendContext";
import { useToast } from "../../context/ToastContext";
import { Button } from "../ui/Button";
import { Label } from "../ui/Label";
import { Select } from "../ui/Select";
import { X, Download, FileCode } from "lucide-react";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  layoutName: string;
  layoutString: string;
}

export function ExportModal({
  isOpen,
  onClose,
  layoutName,
  layoutString,
}: Props) {
  const [format, setFormat] = useState("qmk");
  const [isExporting, setIsExporting] = useState(false);
  const { addToast } = useToast();
  const backend = useBackend();

  if (!isOpen) return null;

  const handleExport = async () => {
    setIsExporting(true);
    try {
      // 1. Generate Code
      const code = await backend.exportFirmware(
        layoutName,
        layoutString,
        format,
      );

      // 2. Save to File
      const ext = format === "qmk" ? "c" : "keymap";
      const defaultName = `${layoutName.toLowerCase().replace(/\s+/g, "_")}.${ext}`;

      const filePath = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: format.toUpperCase() + " Source",
            extensions: [ext],
          },
        ],
      });

      if (filePath) {
        await backend.saveFile(filePath, code);
        addToast("success", `Exported to ${filePath}`);
        onClose();
      }
    } catch (e) {
      addToast("error", `Export Failed: ${e}`);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-in fade-in duration-200">
      <div className="bg-slate-900 border border-slate-800 rounded-xl shadow-2xl w-96 overflow-hidden">
        <div className="p-4 border-b border-slate-800 flex justify-between items-center bg-slate-950/50">
          <h3 className="text-sm font-bold text-white flex items-center gap-2">
            <FileCode size={16} className="text-blue-400" /> Export Firmware
          </h3>
          <button
            onClick={onClose}
            className="text-slate-500 hover:text-white transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        <div className="p-6 space-y-6">
          <div>
            <Label>Format</Label>
            <Select
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              options={[
                { label: "QMK (keymap.c)", value: "qmk" },
                { label: "ZMK (.keymap)", value: "zmk" },
              ]}
            />
            <p className="text-[10px] text-slate-500 mt-2">
              Generates a C array or DeviceTree overlay for your layout.
            </p>
          </div>

          <div className="bg-slate-950 rounded p-3 border border-slate-800">
            <div className="text-[10px] font-bold text-slate-500 uppercase mb-1">
              Preview
            </div>
            <div className="font-mono text-[10px] text-slate-400 truncate">
              {layoutString.substring(0, 40)}...
            </div>
          </div>

          <div className="flex gap-2">
            <Button variant="secondary" className="flex-1" onClick={onClose}>
              Cancel
            </Button>
            <Button
              variant="primary"
              className="flex-1"
              onClick={handleExport}
              isLoading={isExporting}
              icon={<Download size={14} />}
            >
              Export
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
