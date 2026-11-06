// script/official: one module per card, c<code>.rs for c<code>.lua.
use engine::InitialEffect;

mod c10266279;
mod c12375297;
mod c12580477;
mod c13597785;
mod c14152862;
mod c14558127;
mod c15561463;
mod c1641882;
mod c22850702;
mod c24092792;
mod c24094653;
mod c25661743;
mod c26050548;
mod c26202165;
mod c29301450;
mod c30208479;
mod c30271097;
mod c31924889;
mod c33158448;
mod c33854624;
mod c34334692;
mod c36849933;
mod c39317553;
mod c40139997;
mod c4206964;
mod c42141493;
mod c42302563;
mod c44095762;
mod c45883110;
mod c46014517;
mod c46052429;
mod c46130346;
mod c50954680;
mod c53129443;
mod c5318639;
mod c53971455;
mod c54652250;
mod c55144522;
mod c5559570;
mod c56651978;
mod c5914858;
mod c59581480;
mod c60303688;
mod c64491754;
mod c66322203;
mod c66532962;
mod c68468459;
mod c69035382;
mod c6983839;
mod c70088809;
mod c70488851;
mod c73642296;
mod c74405783;
mod c76792184;
mod c78397661;
mod c83764718;
mod c84192580;
mod c84257639;
mod c85976588;
mod c87746184;
mod c91949988;
mod c93125329;
mod c94641726;
mod c96334243;
mod c97556336;

// Sorted by code.
pub const CARD_SCRIPTS: [(u32, InitialEffect); 65] = [
    (1641882, c1641882::initial_effect),
    (4206964, c4206964::initial_effect),
    (5318639, c5318639::initial_effect),
    (5559570, c5559570::initial_effect),
    (5914858, c5914858::initial_effect),
    (6983839, c6983839::initial_effect),
    (10266279, c10266279::initial_effect),
    (12375297, c12375297::initial_effect),
    (12580477, c12580477::initial_effect),
    (13597785, c13597785::initial_effect),
    (14152862, c14152862::initial_effect),
    (14558127, c14558127::initial_effect),
    (15561463, c15561463::initial_effect),
    (22850702, c22850702::initial_effect),
    (24092792, c24092792::initial_effect),
    (24094653, c24094653::initial_effect),
    (25661743, c25661743::initial_effect),
    (26050548, c26050548::initial_effect),
    (26202165, c26202165::initial_effect),
    (29301450, c29301450::initial_effect),
    (30208479, c30208479::initial_effect),
    (30271097, c30271097::initial_effect),
    (31924889, c31924889::initial_effect),
    (33158448, c33158448::initial_effect),
    (33854624, c33854624::initial_effect),
    (34334692, c34334692::initial_effect),
    (36849933, c36849933::initial_effect),
    (39317553, c39317553::initial_effect),
    (40139997, c40139997::initial_effect),
    (42141493, c42141493::initial_effect),
    (42302563, c42302563::initial_effect),
    (44095762, c44095762::initial_effect),
    (45883110, c45883110::initial_effect),
    (46014517, c46014517::initial_effect),
    (46052429, c46052429::initial_effect),
    (46130346, c46130346::initial_effect),
    (50954680, c50954680::initial_effect),
    (53129443, c53129443::initial_effect),
    (53971455, c53971455::initial_effect),
    (54652250, c54652250::initial_effect),
    (55144522, c55144522::initial_effect),
    (56651978, c56651978::initial_effect),
    (59581480, c59581480::initial_effect),
    (60303688, c60303688::initial_effect),
    (64491754, c64491754::initial_effect),
    (66322203, c66322203::initial_effect),
    (66532962, c66532962::initial_effect),
    (68468459, c68468459::initial_effect),
    (69035382, c69035382::initial_effect),
    (70088809, c70088809::initial_effect),
    (70488851, c70488851::initial_effect),
    (73642296, c73642296::initial_effect),
    (74405783, c74405783::initial_effect),
    (76792184, c76792184::initial_effect),
    (78397661, c78397661::initial_effect),
    (83764718, c83764718::initial_effect),
    (84192580, c84192580::initial_effect),
    (84257639, c84257639::initial_effect),
    (85976588, c85976588::initial_effect),
    (87746184, c87746184::initial_effect),
    (91949988, c91949988::initial_effect),
    (93125329, c93125329::initial_effect),
    (94641726, c94641726::initial_effect),
    (96334243, c96334243::initial_effect),
    (97556336, c97556336::initial_effect),
];
// The bootstrap cards of phase 2a.
pub const CLASSIC_CARDS: [u32; 12] = [
    4206964, 5318639, 12580477, 14558127, 26202165, 44095762, 46130346, 53129443, 54652250, 55144522, 83764718, 84257639,
];

// s.listed_names of the scripts that set it, sorted by code.
const LISTED_NAMES: [(u32, &[u32]); 13] = [
    (10266279, c10266279::LISTED_NAMES),
    (12375297, c12375297::LISTED_NAMES),
    (13597785, c13597785::LISTED_NAMES),
    (30208479, c30208479::LISTED_NAMES),
    (30271097, c30271097::LISTED_NAMES),
    (33854624, c33854624::LISTED_NAMES),
    (40139997, c40139997::LISTED_NAMES),
    (45883110, c45883110::LISTED_NAMES),
    (56651978, c56651978::LISTED_NAMES),
    (60303688, c60303688::LISTED_NAMES),
    (74405783, c74405783::LISTED_NAMES),
    (78397661, c78397661::LISTED_NAMES),
    (87746184, c87746184::LISTED_NAMES),
];

pub fn card_script(code: u32) -> Option<InitialEffect> {
    let index = CARD_SCRIPTS.binary_search_by_key(&code, |&(code, _)| code).ok()?;
    CARD_SCRIPTS.get(index).map(|&(_, initial_effect)| initial_effect)
}

// mt.fit_monster of the scripts Ritual.AddProcCode sets it for, sorted by code; Card.ListsCode reads it in place of
// s.listed_names.
const FIT_MONSTER: [(u32, &[u32]); 1] = [(76792184, c76792184::FIT_MONSTER)];

pub fn listed_names(code: u32) -> &'static [u32] {
    let names = |table: &'static [(u32, &'static [u32])]| {
        let index = table.binary_search_by_key(&code, |&(code, _)| code).ok()?;
        table.get(index).map(|&(_, names)| names)
    };
    names(&LISTED_NAMES).or_else(|| names(&FIT_MONSTER)).unwrap_or(&[])
}
