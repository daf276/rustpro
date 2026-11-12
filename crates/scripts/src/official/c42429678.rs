// Bururu @Ignister
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 42429678;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and on the field beside Wizard @Ignister (Level 4) with Wind Pegasus @Ignister in the Extra Deck to Synchro Summon, and Ignister monsters in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (3723262, MZONE),
    (3723262, DECK),
    (30118811, DECK),
    (98506199, EXTRA),
    (98506199, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_IGNISTER]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO && c.get_reason_card(duel).is_some_and(|rc| rc.is_race(duel, RACE_CYBERSE))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_controler(duel, tp)
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let rc = c.get_reason_card(duel);
    unported!(duel, rc.is_none());
    let Some(rc) = rc else { return Ok(false) };
    let mg = rc.get_material(duel);
    if let Some(chkc) = chkc {
        return Ok(mg.contains(chkc) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && mg.is_exists(duel, |sc, duel| spfilter(sc, duel, e, tp), 1, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = mg.filter_select(duel, tp, |sc, duel| spfilter(sc, duel, e, tp), 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}
