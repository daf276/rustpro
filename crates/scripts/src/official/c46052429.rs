// Advanced Ritual Art
use crate::prelude::*;
use crate::testing::{DECK, HAND, SZONE};

struct Params;

impl RitualParams for Params {
    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_field_group(tp, LOCATION_DECK, 0))
    }

    fn extraop(duel: &mut Duel, mat: CardSet, _: EffectId, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(mat, REASON_EFFECT + REASON_MATERIAL + REASON_RITUAL)?;
        Ok(())
    }

    // s.forcedgroup.
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_DECK) && c.is_type(duel, TYPE_NORMAL) && c.is_able_to_grave(duel)
    }
}

// In the hand and set, Ritual Summoning Magician of Black Chaos with Normal Monsters (Levels 4, 4 and 3) from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (46052429, HAND),
    (46052429, SZONE),
    (30208479, HAND),
    (15025844, DECK),
    (69247929, DECK),
    (38142739, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Ritual::create_proc::<Params, RITPROC_EQUAL>(duel, c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DECKDES);
    c.register_effect(duel, e1);
}
