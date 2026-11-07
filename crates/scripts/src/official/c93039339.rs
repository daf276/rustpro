// Super Starslayer TY-PHON - Sky Crisis
use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 93039339;

pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummoned];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(xyzfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.has_flag_effect(1 - tp, ID + 1)
    }

    fn op(duel: &mut Duel, e: EffectId, tp: u8, _: CardId) -> Script<bool> {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        duel.register_effect(e2, tp);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 1);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_attack_above(duel, 3000)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, _: CardId) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None);
    g.get_max_group(duel, |c, duel| c.get_attack(duel).into())
        .is_some_and(|(max, _)| max.is_contains(c))
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_summon_location(duel, LOCATION_EXTRA) {
            let sp = tc.get_summon_player(duel);
            duel.register_flag_effect(sp, ID, RESET_PHASE | PHASE_END, 0, 1);
            if duel.has_flag_effect_full(sp, ID, 2) {
                duel.register_flag_effect(sp, ID + 1, RESET_PHASE | PHASE_END, 0, 2);
            }
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
