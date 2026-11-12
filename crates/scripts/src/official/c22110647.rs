// Mecha Phantom Beast Dracossack
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 22110647;

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECHA_PHANTOM_BEAST];

// On the field with Xyz Import to attach Sangan, Level 7 Normal Monsters (Gaia The Fierce Knight) to Xyz Summon it, and Mecha Phantom
// Beast Blue Impala to release.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (6368038, MZONE),
    (6368038, MZONE),
    (67489919, MZONE),
    (67489919, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, descost);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_MECHA_PHANTOM_BEAST, TYPES_TOKEN, 0, 0, 3, RACE_MACHINE, ATTRIBUTE_WIND)
}

fn tknfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TOKEN) || c.is_original_type(duel, TYPE_TOKEN)
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(tknfilter, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_player_can_special_summon_monster(tp, TOKEN_MECHA_PHANTOM_BEAST, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 1
        && duel.is_player_can_special_summon_monster(tp, TOKEN_MECHA_PHANTOM_BEAST, token_stats())
    {
        for _ in 0..2 {
            let token = duel.create_token(tp, TOKEN_MECHA_PHANTOM_BEAST_DRACOSSACK);
            unported!(duel, token.is_none());
            if let Some(token) = token {
                duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn descfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST])
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let dg = duel.get_matching_group(
        |tc, duel| tc.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    let check = aux::release_check_target(dg);
    if chk == 0 {
        return Ok(
            c.get_attack_announced_count(duel) == 0 && duel.check_release_group_cost(tp, descfilter, (1, 1), false, Some(&check), None)
        );
    }
    let g = duel.select_release_group_cost(tp, descfilter, (1, 1), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD) > 1
            && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    // Lua errors on a nil target
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
