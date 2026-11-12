// Vector Scare Archfiend
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 13452889;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// On the field with Cyberse Normal Monsters in its Link zones to release, and weak ones of the opponent's to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, aux::bdogcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, g: CardSet, zone: u32) -> bool {
    g.is_contains(c)
        && (duel.check_location(c.get_controler(duel), LOCATION_MZONE, c.get_sequence(duel))
            || duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let lg = c.get_linked_group(duel);
    let zone = c.get_linked_zone_full(duel, 1 - tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |rc, duel| cfilter(rc, duel, tp, lg, zone), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |rc, duel| cfilter(rc, duel, tp, lg, zone), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(false) };
    if chk == 0 {
        let (zone1, zone2) = (c.get_linked_zone_full(duel, tp), c.get_linked_zone_full(duel, 1 - tp));
        return Ok(c.get_linked_zone(duel) != 0
            && (bc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone1)
                || bc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, zone2)));
    }
    duel.set_target_card(bc);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.get_linked_zone(duel) == 0 || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let (zone1, zone2) = (c.get_linked_zone_full(duel, tp), c.get_linked_zone_full(duel, 1 - tp));
    if tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone1)
        && (!tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, zone2)
            || duel.select_yes_no(tp, aux::stringid(ID, 1))?)
    {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone1)?;
    } else if duel.special_summon_full(tc, 0, tp, 1 - tp, false, false, POS_FACEUP, zone2)? != 0 && c.is_relate_to_battle(duel) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
