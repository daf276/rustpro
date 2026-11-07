// Chaos Form
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_BLUEEYES_W_DRAGON];

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_CHAOS, SET_NUMBER_C]) && c.is_ritual_monster(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_matching_group(mfilter, tp, LOCATION_GRAVE, 0, None))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
        }
        Ok(true)
    }
}

// Magician of Black Chaos (Level 8) to Ritual Summon, with Dark Magician and Blue-Eyes White Dragon in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (21082832, HAND),
    (30208479, HAND),
    (CARD_DARK_MAGICIAN, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_equal::<Params>(duel, c);
}

fn mfilter(c: CardId, duel: &Duel) -> bool {
    duel.is_player_affected_by_effect(c.get_controler(duel), CARD_SPIRIT_ELIMINATION)
        .is_none()
        && c.has_level(duel)
        && (c.is_code(duel, CARD_DARK_MAGICIAN) || c.is_code(duel, CARD_BLUEEYES_W_DRAGON))
        && c.is_monster(duel)
        && c.is_able_to_remove(duel)
}
