// Performapal Miss Director
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 92932860;

pub const LISTED_SERIES: &[u16] = &[SET_ODD_EYES];
// On the field with Key Mace (a Level 1 non-Tuner) in the Graveyard and Odd-Eyes Meteorburst Dragon (Level 7) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (1929294, GRAVE),
    (1929294, GRAVE),
    (80696379, EXTRA),
    (80696379, EXTRA),
    (80696379, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, econ);
    e2.set_target_card(duel, efilter);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_condition(duel, econ);
    e3.set_target_card(duel, efilter);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, sctg);
    e4.set_operation(duel, scop);
    c.register_effect(duel, e4);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(aux::TRUE, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn econ(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_defense_pos(duel)
}

fn efilter(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_ODD_EYES])
}

fn scfilter(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_synchro_summonable(duel, None, Some(mg))
}

fn mfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let mg = CardSet::from_iter([c, mc]);
    c.get_level(duel) == 1
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_type(duel, TYPE_TUNER)
        && duel.is_existing_matching_card(|sc, duel| scfilter(sc, duel, mg), tp, LOCATION_EXTRA, 0, 1, None)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chkc.is_some() {
        // The Lua calls the undefined s.scfilter1 here, which errors.
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|tc, duel| mfilter(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |tc, duel| mfilter(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    tc.register_effect(duel, e2);
    duel.special_summon_complete()?;
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = CardSet::from_iter([c, tc]);
    let g = duel.get_matching_group(|sc, duel| scfilter(sc, duel, mg), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, Some(mg))?;
        }
    }
    Ok(())
}
