// Link Devotee
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 48068378;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        matfilter(c, duel, lc, sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[TOKEN_LINK];

// On the field under Traffic Ghosts (Bottom arrow, so mutually linked), with Cannon Soldiers to Tribute it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (41248270, MZONE),
    (41248270, MZONE),
    (11384280, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(TOKEN_LINK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, ls);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_label(duel) > 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_LEAVE_FIELD_P);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_label_object(duel, Some(e2));
    e3.set_operation(duel, chk);
    c.register_effect(duel, e3);
}

fn chk(duel: &mut Duel, a: &Args) -> Script<()> {
    let count = a.e.get_handler(duel).get_mutual_linked_group_count(duel) as i64;
    if let Some(e2) = a.e.get_label_object(duel) {
        e2.set_label(duel, count);
    }
    Ok(())
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
}

fn ls(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_spsummon(duel, splimit);
    duel.register_effect(e1, tp);
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, sumtp: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    c.is_link_monster(duel) && c.get_link(duel) >= 3 && sumtp & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_CYBERSE, ATTRIBUTE_LIGHT)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_player_can_special_summon_monster(tp, TOKEN_LINK, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) < 2
        || !duel.is_player_can_special_summon_monster(tp, TOKEN_LINK, token_stats())
    {
        return Ok(());
    }
    for _ in 0..2 {
        if let Some(token) = duel.create_token(tp, TOKEN_LINK) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
