// Unchained Soul of Anguish
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

use super::c67680512::{operation, target, thcon, thop, thtg};

const ID: u32 = 93084621;

pub const LISTED_SERIES: &[u16] = &[SET_UNCHAINED_SOUL];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_UNCHAINED_SOUL], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field, with an opponent's monster, and Unchained Abomination and Soul of Rage in the Extra Deck to Link Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (29479265, EXTRA), (67680512, EXTRA), (26236560, MZONE | OPPONENT)];
// Both effects have the same description, so the Link Summon of Rage stands for the first.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(67680512)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target::<ID, false>);
    e1.set_operation(duel, operation::<ID, false>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg::<ID>);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}
