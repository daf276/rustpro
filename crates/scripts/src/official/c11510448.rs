// Zoodiac Tigermortar
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE};

const ID: u32 = 11510448;

pub const LISTED_SERIES: &[u16] = &[SET_ZOODIAC];
// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby), with Zoodiac Xyz Monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (74393852, GRAVE),
    (14970113, GRAVE),
];

// The Xyz procedure of the Zoodiac Xyz Monsters that Xyz Summon over a "Zoodiac" monster of another name once per turn.
pub struct Procedure<const CODE: u32, const CT: usize, const MAXCT: usize>;

impl<const CODE: u32, const CT: usize, const MAXCT: usize> XyzProcedure for Procedure<CODE, CT, MAXCT> {
    const LV: Option<u32> = Some(4);
    const CT: usize = CT;
    const MAXCT: usize = MAXCT;
    const ALTERF: Option<AlterFilter> = Some(ovfilter::<CODE>);
    const DESC: u64 = aux::stringid(CODE, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, CODE) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, CODE, RESET_PHASE | PHASE_END, 0, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<ID, 3, 3>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_value(duel, Value::Card(defval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

pub fn ovfilter<const CODE: u32>(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_set_card_full(duel, &[SET_ZOODIAC], Some(lc), SUMMON_TYPE_XYZ as u64, tp)
        && !c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[CODE])
}

pub fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e.get_handler(duel).get_overlay_group(duel);
    g.filter(duel, |c, duel| c.is_set_card(duel, &[SET_ZOODIAC]) && c.get_attack(duel) >= 0, None)
        .get_sum(duel, |c, duel| c.get_attack(duel).into()) as i32
}

pub fn defval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e.get_handler(duel).get_overlay_group(duel);
    g.filter(
        duel,
        |c, duel| c.is_set_card(duel, &[SET_ZOODIAC]) && c.get_defense(duel) >= 0,
        None,
    )
    .get_sum(duel, |c, duel| c.get_defense(duel).into()) as i32
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ZOODIAC]) && c.is_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let xyz = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ));
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(&xyz, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(filter2, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    duel.select_target(tp, &xyz, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_target(tp, filter2, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    let tc1 = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None).get_first();
    let g2 = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if let Some(tc1) = tc1
        && tc1.is_faceup(duel)
        && !tc1.is_immune_to_effect(duel, a.e)
        && !g2.is_empty()
    {
        duel.overlay(tc1, g2)?;
    }
    Ok(())
}
