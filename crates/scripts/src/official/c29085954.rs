// Number 78: Number Archive
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 29085954;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
pub const XYZ_NUMBER: u32 = 78;

// In the Extra Deck (Xyz Summoned with two Level 1 Normal Monsters) among other Number Xyz Monsters for the opponent to pick.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (55935416, EXTRA),
    (48928529, EXTRA),
    (7194917, EXTRA),
    (1929294, MZONE),
    (3557275, MZONE),
    (12057781, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, pg: CardSet) -> bool {
    crate::official::xyz_number(c.get_code(duel)).is_some_and(|no| (1..=99).contains(&no))
        && c.is_set_card(duel, &[SET_NUMBER])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && (pg.is_empty() || pg.is_contains(mc))
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok(pg.len() <= 1
            && duel.get_location_count_from_ex(tp, tp, Some(c.into()), TYPE_XYZ) > 0
            && duel.get_field_group_count(tp, LOCATION_EXTRA, 0) > 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty()
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), TYPE_XYZ) > 0
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
        && c.is_controler(duel, tp)
        && !c.is_immune_to_effect(duel, e)
    {
        let tg = g.random_select(duel, 1 - tp, 1);
        duel.confirm_cards(1 - tp, tg)?;
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        if tg.is_exists(duel, |tc, duel| filter(tc, duel, e, tp, c, pg), 1, None)
            && let Some(tc) = tg.get_first()
        {
            tc.set_material(duel, Some(c.into()));
            duel.overlay(tc, c)?;
            duel.special_summon(tc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
            let fid = c.get_field_id(duel);
            tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, i64::from(fid), 0);
            tc.complete_procedure(duel);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
            e1.set_code(duel, EVENT_PHASE + PHASE_END);
            e1.set_count_limit(duel, 1, 0);
            e1.set_label(duel, i64::from(fid));
            e1.set_label_object_group(duel, Some(tc.into()));
            e1.set_condition_mut(duel, rmcon);
            e1.set_operation(duel, rmop);
            duel.register_effect(e1, tp);
        }
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect_label(duel, ID) != Some(a.e.get_label(duel))) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
