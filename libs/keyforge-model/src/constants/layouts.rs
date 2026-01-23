// libs/keyforge-model/src/constants/layouts.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Standard layout definitions for analysis and fingerprinting.

/// Standard QWERTY layout string (30 keys + punctuation).
pub const QWERTY: &str = "qwertyuiopasdfghjkl;zxcvbnm,./";

/// Standard Colemak layout string.
pub const COLEMAK: &str = "qwfpgjluy;arstdhneiozxcvbkm,./";

/// Standard Dvorak layout string.
pub const DVORAK: &str = "',.pyfgcrlaoeuidhtns;qjkxbmwvz";
