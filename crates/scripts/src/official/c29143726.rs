// Samsara, Dragon of Rebirth
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 29143726;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Kojikocy (Level 4), with the opponent's Dark Hole
// in hand to destroy it and monsters to summon from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon1);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, spcon2);
    c.register_effect(duel, e2);
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp != a.tp && a.r & REASON_EFFECT != 0 && c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
