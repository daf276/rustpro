// Number 3: Cicada King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4997565;
pub const XYZ_NUMBER: u32 = 3;

// In the Extra Deck, Xyz Summoned with two Level 3 Insects, with more Insects in the hand and Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (41403766, MZONE),
    (56283725, MZONE),
    (70924884, MZONE),
    (77568553, HAND),
    (88643173, GRAVE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_CHANGE_POS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_INSECT) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    loc & LOCATION_ONFIELD != 0
        && re.is_monster_effect(duel)
        && rc.is_negatable_monster(duel)
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && rc.is_relate_to_effect(duel, re)
        && rc.is_can_be_effect_target(duel, a.e)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(re) = a.re {
        duel.set_target_card(re.get_handler(duel));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_INSECT) && (!c.is_type(duel, TYPE_LINK) || c.is_can_change_position(duel))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER)) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    tc.register_effect(duel, e2);
    if tc.is_immune_to_effect(duel, e1) || tc.is_immune_to_effect(duel, e2) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    duel.break_effect()?;
    if sc.is_can_change_position(duel) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.change_position_full(sc, [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, 0], false)?;
    } else {
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
        e3.set_value(duel, 500);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e3);
    }
    Ok(())
}
