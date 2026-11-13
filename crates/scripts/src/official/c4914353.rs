// Space Insulator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 4914353;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// In the Graveyard, with Link Spider (a Cyberse Link Monster) to Link Summon from Normal Monsters (Digitron, Bitron).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (98978921, EXTRA),
    (98978921, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_linked_group(duel).is_contains(c));
    e1.set_value(duel, -800);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_race(duel, RACE_CYBERSE) && c.is_link_monster(duel) && c.is_link_summoned(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None) && aux::exccon(duel, a)
}

fn linked_zone(duel: &Duel, a: &Args) -> u32 {
    let tp = a.tp;
    let lg = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, tp), None);
    lg.iter().fold(0, |zone, tc| zone | tc.get_linked_zone(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = linked_zone(duel, a);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = linked_zone(duel, a);
    if c.is_relate_to_effect(duel, e) && zone != 0 && duel.special_summon_step_full(c, 0, tp, tp, false, false, POS_FACEUP, zone)? {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3312);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3300);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e2.set_value(duel, LOCATION_REMOVED as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e2, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
