//! Words for start-up, Auto Activation, services as sources, importing, licence info and accents,
//! in the same language order as `i18n::LANGS`. The words themselves are split over three files.

pub const N6: usize = 60;

use crate::i18n_extra5a::*;
use crate::i18n_extra5b::*;
use crate::i18n_extra5c::*;

pub const EXTRA5: [&[&str; N6]; 25] = [
    &EN, &ES, &FR, &DE, &IT, &PT, &NL, &SV, &DA, &NB, &FI, &PL, &CS, &TR, &RO, &HU, &EL, &RU, &UK, &ID, &VI, &ZH_HANS,
    &ZH_HANT, &JA, &KO,
];
