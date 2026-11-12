// Magicolloidal Sol
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 67694706;

// Copies in the hand and Graveyard to Special Summon with Call of the Haunted (a Trap) and a Level 2 monster (Doron) to revive, and on the
// field as the Level 2 of Desert Locusts (Level 6) with Angel Trumpeter (a Level 4 Tuner) or of Number 29: Mannequin Cat with Doron.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (ID, MZONE),
    (97077563, HAND),
    (97077563, HAND),
    (756652, GRAVE),
    (756652, MZONE),
    (87979586, MZONE),
    (63410069, EXTRA),
    (63410069, EXTRA),
    (54191698, EXTRA),
    (54191698, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e2.set_condition(duel, |duel, a| duel.is_battle_phase() && duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, sxtg);
    e2.set_operation(duel, sxop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.re.is_some_and(|re| {
        re.is_trap_effect(duel)
            && ((re.is_activated(duel) && duel.get_chain_info::<Option<u8>>(0, CHAININFO_TRIGGERING_PLAYER) == Some(tp))
                || re.get_handler_player(duel) == tp)
    })
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && c.is_summon_location(duel, LOCATION_GRAVE)
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn sxtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let b1 = duel.is_existing_matching_card(
        |tc, duel| tc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        None,
    );
    let b2 = duel.is_existing_matching_card(
        |tc, duel| tc.is_xyz_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        None,
    );
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn sxop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let op = e.get_label(duel);
    if op == 1 {
        let g = duel.get_matching_group(
            |tc, duel| tc.is_synchro_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            None,
        );
        if !g.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
                duel.synchro_summon(tp, sc, Some(c.into()), None)?;
            }
        }
    } else if op == 2 {
        let g = duel.get_matching_group(
            |tc, duel| tc.is_xyz_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            None,
        );
        if !g.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
                duel.xyz_summon(tp, sc, Some(c.into()), None)?;
            }
        }
    }
    Ok(())
}
