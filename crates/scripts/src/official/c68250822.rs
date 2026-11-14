// Spright Double Cross
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 68250822;

// Gigantic Spright (Rank 2) to attach to, Spright Sprind (Link 2) to point at zones, and monsters in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (54498517, MZONE),
    (54498517, MZONE),
    (72329844, MZONE),
    (7892180, GRAVE),
    (7892180, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn atchfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && !c.is_type(duel, TYPE_TOKEN)
        && (c.is_controler(duel, tp) || c.is_location(duel, LOCATION_GRAVE) || c.is_able_to_change_controler(duel))
        && duel.is_existing_matching_card(xyzfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_rank(duel, &[2]) && c.is_faceup(duel)
}

fn ctrlfilter(c: CardId, duel: &Duel, zone: u32) -> bool {
    c.is_controler_can_be_changed_full(duel, false, zone)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn link2_zones(duel: &Duel, tp: u8) -> u32 {
    aux::get_mm_zones_pointed_to_full(duel, tp, |c, duel| c.is_link(duel, &[2]), LOCATION_MZONE, 0, tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = link2_zones(duel, tp);
    let both = LOCATION_MZONE | LOCATION_GRAVE;
    let b1 = duel.is_existing_target(|c, duel| atchfilter(c, duel, tp), tp, both, both, 1, None);
    let b2 = duel.is_existing_target(|c, duel| ctrlfilter(c, duel, zone), tp, 0, LOCATION_MZONE, 1, None);
    let b3 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(
            |c, duel| spfilter(c, duel, e, tp, zone),
            tp,
            LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            None,
        );
    if let Some(chkc) = chkc {
        match e.get_label(duel) {
            1 => return Ok(chkc.is_location(duel, both) && atchfilter(chkc, duel, tp)),
            2 => return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && ctrlfilter(chkc, duel, zone)),
            3 => return Ok(chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp, zone)),
            _ => {}
        }
    }
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    let op = op.unwrap_or(0);
    e.set_label(duel, op as i64);
    match op {
        1 => {
            e.set_category(duel, 0);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
            let g = duel.select_target(tp, |c, duel| atchfilter(c, duel, tp), tp, both, both, 1, 1, None)?;
            if let Some(g) = g
                && g.get_first().is_some_and(|c| c.is_location(duel, LOCATION_GRAVE))
            {
                duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g), 1, 0, 0);
            }
        }
        2 => {
            e.set_category(duel, CATEGORY_CONTROL);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
            // The Lua passes the undefined global `ctrlfilter`, so any monster can be targeted.
            let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
        }
        _ => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_target(
                tp,
                |c, duel| spfilter(c, duel, e, tp, zone),
                tp,
                LOCATION_GRAVE,
                LOCATION_GRAVE,
                1,
                1,
                None,
            )?;
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
        }
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let zone = link2_zones(duel, tp);
    match e.get_label(duel) {
        1 => {
            if !tc.is_immune_to_effect(duel, e) && duel.is_existing_matching_card(xyzfilter, tp, LOCATION_MZONE, 0, 1, Some(tc.into())) {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
                if let Some(xc) = duel
                    .select_matching_card(tp, xyzfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(tc.into()))?
                    .get_first()
                {
                    duel.overlay_full(xc, tc, true)?;
                }
            }
        }
        2 => {
            if tc.is_relate_to_effect(duel, e) && tc.is_controler(duel, 1 - tp) {
                let reason_player = duel.get_reason_player();
                duel.get_control_full(tc, tp, 0, 0, zone, reason_player)?;
            }
        }
        _ => {
            if duel.get_location_count(tp, LOCATION_MZONE) > 0 && zone > 0 && spfilter(tc, duel, e, tp, zone) {
                duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?;
            }
        }
    }
    Ok(())
}
