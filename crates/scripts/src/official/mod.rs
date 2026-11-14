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
    // Whether its own source (or a card source it uses) reads or picks zones, or checks or changes battle positions, as
    // build.rs judges by identifiers; Link arrows and Pendulum Zones are not counted, so callers check those types themselves.
    pub uses_zones: bool,
    pub uses_positions: bool,
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

// s.listed_names, which Card.ListsCodeWithArchetype reads, and Card.ListsCode unless it is nil.
pub fn listed_names(code: u32) -> Option<&'static [u32]> {
    lookup(&LISTED_NAMES, code)
}

// mt.fit_monster, which Ritual.AddProcCode sets and Card.ListsCode reads when s.listed_names is nil.
pub fn fit_monster(code: u32) -> Option<&'static [u32]> {
    lookup(&FIT_MONSTER, code)
}

// s.listed_series, which Card.ListsArchetype reads.
pub fn listed_series(code: u32) -> &'static [u16] {
    lookup(&LISTED_SERIES, code).unwrap_or(&[])
}

// s.counter_place_list, which Card.PlacesCounter reads.
pub fn counter_place_list(code: u32) -> &'static [u16] {
    lookup(&COUNTER_PLACE_LIST, code).unwrap_or(&[])
}

// s.counter_list, which Card.ListsCounter reads in preference to counter_place_list.
pub fn counter_list(code: u32) -> Option<&'static [u16]> {
    lookup(&COUNTER_LIST, code)
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

// s.toss_coin, which the cards that search for a coin tossing effect read on the cards they check.
pub fn toss_coin(code: u32) -> bool {
    lookup(&TOSS_COIN, code).unwrap_or(false)
}

// s.roll_dice, which the cards that search for a die rolling effect read on the cards they check.
pub fn roll_dice(code: u32) -> bool {
    lookup(&ROLL_DICE, code).unwrap_or(false)
}

// s.xyz_number, which Card.xyz_number reads.
pub fn xyz_number(code: u32) -> Option<u32> {
    lookup(&XYZ_NUMBER, code)
}

// s.assault_mode, the code of the Synchro Monster an Assault Mode monster is the Special Summon for.
pub fn assault_mode(code: u32) -> Option<u32> {
    lookup(&ASSAULT_MODE, code)
}

// s.pendulum_level, which Pendulum.Filter reads in place of the card's Level.
pub fn pendulum_level(code: u32) -> Option<u32> {
    lookup(&PENDULUM_LEVEL, code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_flag_scripts_that_read_zones_or_positions() {
        // Junora needs a material in the center Main Monster Zone, Artmage Diactorus changes a battle position.
        let flags = |code| card(code).map(|card| (card.uses_zones, card.uses_positions));
        let expected = [Some((true, false)), Some((false, true)), Some((false, false))];
        assert_eq!([5914858, 27184601, 55144522].map(flags), expected);
    }
}
