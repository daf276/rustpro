// Archnemeses Eschatos
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 33212663;

// In the hand with monsters of three Types to banish (Sangan, Alpha The Magnet Warrior, Mystical Elf) on the field and in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26202165, GRAVE),
    (99785935, GRAVE),
    (15025844, GRAVE),
    (99785935, MZONE),
    (15025844, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && (c.is_faceup(duel) || !c.is_on_field(duel))
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, true)
}

fn rescon(duel: &Duel, tp: u8, sg: CardSet) -> (bool, bool) {
    let races = || sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64);
    (
        aux::chkf_mmz(tp, 1)(duel, sg, CardSet::default(), None).0 && races() == sg.len(),
        races() != sg.len(),
    )
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && rg.len() > 2
        && aux::select_unselect_group_check(duel, rg, 3, 3, &|duel, sg, _, _| rescon(duel, tp, sg))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, tp, sg),
        tp,
        HINTMSG_REMOVE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let race = g.iter().fold(0, |race, tc| race | tc.get_race(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RACE);
    let arc = duel.announce_race(tp, 1, race)?;
    a.e.set_label(duel, arc as i64);
    let dg = g.filter(duel, |c, duel| c.is_race(duel, arc), None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let arc = e.get_label(duel) as u64;
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_race(duel, arc)),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 1);
    e1.set_label(duel, arc as i64);
    e1.set_target_spsummon(duel, |duel, e, c, _, _, _, _, _| c.is_race(duel, e.get_label(duel) as u64));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    Ok(())
}
