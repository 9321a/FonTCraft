//! Words for app compatibility, the sidebar's Libraries/Folders, the Import window, the glyph menu,
//! character info, similar shapes and printing, in the same language order as `i18n::LANGS`.
//! The words are split over two files (`i18n_extra7a`, `i18n_extra7b`).

pub const N8: usize = 56;

use crate::i18n_extra7a::*;
use crate::i18n_extra7b::*;

pub const EXTRA7: [&[&str; N8]; 25] = [
    &EN, &ES, &FR, &DE, &IT, &PT, &NL, &SV, &DA, &NB, &FI, &PL, &CS, &TR, &RO, &HU, &EL, &RU, &UK, &ID, &VI, &ZH_HANS,
    &ZH_HANT, &JA, &KO,
];
