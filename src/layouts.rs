use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, EnumIter, EnumString, Display, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum KnownLayout {
    Qwerty,
    Colemak,
    ColemakDH,
    Canary,
    Dvorak,
    HandsDownNeu,
    Sturdy,
    Focal,
    Graphite,
    Gallium,
    Engram,
    Workman,
}

impl KnownLayout {
    // Maps standard 30-key row-stagger.
    pub fn get_str(&self) -> &'static str {
        match self {
            Self::Qwerty => "qwertyuiopasdfghjkl;zxcvbnm,./",
            Self::Dvorak => "',.pyfgcrlaoeuidhtns;qjkxbmwvz",
            Self::Colemak => "qwfpgjluy;arstdhneiozxcvbkm,./",
            Self::ColemakDH => "qwfpbjluy;arstgmneiozxccdkv,./",
            Self::Workman => "qdrwbjfup;ashtgyneoizxmcvkl,./",

            // Canary (Standard)
            Self::Canary => "wlypkzfou;crstgmneiaqjvdkhx,./",

            // Sturdy (Standard)
            Self::Sturdy => "vmlhkqjou;strygfaeibxcdwnzp,./",

            // Hands Down Neu (Standard)
            Self::HandsDownNeu => "xrybpjlcu;snhtgmoeaizwvdkqf,./",

            // Focal (Standard)
            Self::Focal => "wlypkvzou;rsntghjaeiqxcbdmf,./",

            // Graphite (Corrected)
            // Source: b l d w z ' f o u j  |  n r t s g y h a e i  |  q x m c v k p . , /
            // Note: ' is not in standard char set usually, mapped to ; for comparison or handled gracefully
            // We will map ' to ; for the standard 30-key block if strict
            Self::Graphite => "bldwz'foujnrtsgyhaieqxmcvkp.,/",

            // Gallium v2 (Corrected)
            // Source: b l d c v j y o u ,  |  n r t s g p h a e i  |  x q m w z k f ' ; .
            // Mapped ' to / and , to ; for standard fit if needed, but let's use literal chars
            // Assuming standard 30-key set: q w e r t y u i o p a s d f g h j k l ; z x c v b n m , . /
            // We will approximate the punctuation slots to match the corpus expectations
            Self::Gallium => "bldcvjyou,nrtsgphaiexqmwzkf';.",

            // Engram (Standard)
            Self::Engram => "byou'ldwvpciea,htsnqjxkrmgzf.;",
        }
    }

    pub fn to_bytes(&self) -> [u8; 30] {
        let s = self.get_str();
        let mut bytes = [0u8; 30];
        for (i, b) in s.bytes().enumerate() {
            if i < 30 {
                bytes[i] = b;
            }
        }
        bytes
    }
}

pub fn get_all_layouts() -> HashMap<KnownLayout, [u8; 30]> {
    let mut map = HashMap::new();
    for layout in KnownLayout::iter() {
        map.insert(layout, layout.to_bytes());
    }
    map
}
