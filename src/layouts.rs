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
    HandsDownRef, // Renamed to "Ref" to imply it's the analyzer baseline, not necessarily "Neu"
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

            // FIXED: Removed duplicate 'c', added missing 'd'
            // Z X C D V K H , . /
            Self::ColemakDH => "qwfpbjluy;arstgmneiozxcdvkh,./",

            Self::Workman => "qdrwbjfup;ashtgyneoizxmcvkl,./",

            // Canary (User Variant: B on Top-Left-Stretch, K on Bottom-Right-Index)
            // w l y p b  z f o u '
            // c r s t g  m n e i a
            // q j v d k  x h / , .
            Self::Canary => "wlypbzfou'crstgmneiaqjvdkxh/,.",

            // Sturdy (Standard)
            // v m l c p  x f o u j
            // s t r d y  n a e i h
            // z k q g w  b ' ; .
            Self::Sturdy => "vmlcpxfoujstrydnaeihzkqgwb';.,",

            // Hands Down (Reference/Gold-ish variation often used in analyzers)
            Self::HandsDownRef => "xrybpjlcu;snhtgmoeaizwvdkqf,./",

            // Focal (Standard)
            // w l y p k  z o u ; ,
            // r s t n g  m h a e i
            // q x c d v  j f b ' .
            Self::Focal => "wlypkzou;,rstngmhaieqxcdvjfb'.",

            // Graphite
            // b l d w z  ' f o u j
            // n r t s g  y h a e i
            // q x m c v  k p . , /
            Self::Graphite => "bldwz'foujnrtsgyhaieqxmcvkp.,/",

            // Gallium v2
            // b l d c v  j y o u ,
            // n r t s g  p h a e i
            // x q m w z  k f ' ; .
            Self::Gallium => "bldcvjyou,nrtsgphaiexqmwzkf';.",

            // Engram (Standard)
            // b y o u '  l d w v z
            // c i e a ,  h t s n q
            // g x j k -  r m f p ? (Approximated to standard punctuation)
            Self::Engram => "byou'ldwvzciea,htsnqgxjkmfp;./",
        }
    }

    pub fn to_bytes(&self) -> [u8; 30] {
        let s = self.get_str();
        let mut bytes = [0u8; 30];
        // Copy bytes, padding with spaces if string is short (safety check)
        for (i, b) in s.bytes().take(30).enumerate() {
            bytes[i] = b;
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
