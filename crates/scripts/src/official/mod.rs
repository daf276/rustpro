// script/official: one module per card, c<code>.rs for c<code>.lua, registered by build.rs.
use engine::InitialEffect;

use crate::testing::Check;

include!(concat!(env!("OUT_DIR"), "/official.rs"));

// A ported card: its script and what its scenarios need.
pub struct CardScript {
    pub code: u32,
    pub initial_effect: InitialEffect,
    pub support: &'static [(u32, u16)],
    pub checks: &'static [Check],
}

// The bootstrap cards of phase 2a.
pub const CLASSIC_CARDS: [u32; 12] = [
    4206964, 5318639, 12580477, 14558127, 26202165, 44095762, 46130346, 53129443, 54652250, 55144522, 83764718, 84257639,
];

pub fn card(code: u32) -> Option<&'static CardScript> {
    let index = CARD_SCRIPTS.binary_search_by_key(&code, |card| card.code).ok()?;
    CARD_SCRIPTS.get(index)
}

pub fn card_script(code: u32) -> Option<InitialEffect> {
    card(code).map(|card| card.initial_effect)
}

// s.listed_names, or else mt.fit_monster, which Ritual.AddProcCode sets and Card.ListsCode reads in its place.
pub fn listed_names(code: u32) -> &'static [u32] {
    let index = LISTED_NAMES.binary_search_by_key(&code, |&(code, _)| code).ok();
    index.and_then(|index| LISTED_NAMES.get(index)).map_or(&[], |&(_, names)| names)
}

// s.material_setcode, which Card.ListsArchetypeAsMaterial reads.
pub fn material_setcode(code: u32) -> &'static [u16] {
    let index = MATERIAL_SETCODE.binary_search_by_key(&code, |&(code, _)| code).ok();
    index
        .and_then(|index| MATERIAL_SETCODE.get(index))
        .map_or(&[], |&(_, setcodes)| setcodes)
}

// s.dark_calling = true, which the Evil HERO cards read on the cards they check.
pub fn dark_calling(code: u32) -> bool {
    let index = DARK_CALLING.binary_search_by_key(&code, |&(code, _)| code).ok();
    index.and_then(|index| DARK_CALLING.get(index)).is_some_and(|&(_, flag)| flag)
}

// s.xyz_number, which the Number monsters declare and Card.xyz_number reads.
pub fn xyz_number(code: u32) -> Option<u32> {
    let index = XYZ_NUMBER.binary_search_by_key(&code, |&(code, _)| code).ok();
    index.and_then(|index| XYZ_NUMBER.get(index)).map(|&(_, number)| number)
}
