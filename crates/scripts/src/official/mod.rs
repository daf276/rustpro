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

fn lookup<T: Copy>(table: &'static [(u32, T)], code: u32) -> Option<T> {
    let index = table.binary_search_by_key(&code, |&(code, _)| code).ok()?;
    table.get(index).map(|&(_, value)| value)
}

// s.listed_names, or else mt.fit_monster, which Ritual.AddProcCode sets and Card.ListsCode reads in its place.
pub fn listed_names(code: u32) -> &'static [u32] {
    lookup(&LISTED_NAMES, code).unwrap_or(&[])
}

// s.material_setcode, which Card.ListsArchetypeAsMaterial reads.
pub fn material_setcode(code: u32) -> &'static [u16] {
    lookup(&MATERIAL_SETCODE, code).unwrap_or(&[])
}

// s.material of a Synchro, Xyz or Link Monster; Fusion Monsters record theirs in the duel.
pub fn material(code: u32) -> &'static [u32] {
    lookup(&MATERIAL, code).unwrap_or(&[])
}

// s.dark_calling, which the Evil HERO cards read on the cards they check.
pub fn dark_calling(code: u32) -> bool {
    lookup(&DARK_CALLING, code).unwrap_or(false)
}

// s.xyz_number, which Card.xyz_number reads.
pub fn xyz_number(code: u32) -> Option<u32> {
    lookup(&XYZ_NUMBER, code)
}

// s.assault_mode, the code of the Synchro Monster an Assault Mode monster is the Special Summon for.
pub fn assault_mode(code: u32) -> Option<u32> {
    lookup(&ASSAULT_MODE, code)
}
