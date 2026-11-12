// Melffy Mommy
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 76833149;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, _, _, _| c.is_race(duel, RACE_BEAST));
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// In the Extra Deck, Xyz Summoned with Melffy Rabby, a Level 2 Beast Normal Monster, which she also takes from the hand as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (20129614, HAND),
    (20129614, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    // Needs 5 materials and an attack into an Attack Position monster with ATK, which random play rarely reaches.
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) >= 3);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) >= 4);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, damcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_race(duel, RACE_BEAST)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let (c, tp) = (a.e.get_handler(duel), a.tp);
        return Ok(duel.is_existing_matching_card(
            |g, duel| attachfilter(g, duel, c, tp),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            None,
        ));
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(
        tp,
        |g, duel| attachfilter(g, duel, c, tp),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
    } else {
        duel.hint_selection(sc);
    }
    duel.overlay_full(c, sc, true)?;
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    c.get_overlay_count(duel) >= 5 && bc1 == Some(c) && bc2.is_some_and(|bc2| bc2.is_attack_pos(duel) && bc2.has_non_zero_attack(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let (_, bc) = duel.get_battle_monster(tp);
    let Some(bc) = bc else { return Ok(false) };
    a.e.set_label_object_group(duel, Some(bc.into()));
    bc.create_effect_relation(duel, a.e);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, bc.get_attack(duel));
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(bc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if bc.is_relate_to_effect(duel, a.e) && bc.is_faceup(duel) && bc.is_attack_pos(duel) {
        duel.damage(1 - a.tp, bc.get_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
