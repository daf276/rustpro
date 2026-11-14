// Ojama Emperor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 34031284;

pub const LISTED_SERIES: &[u16] = &[SET_OJAMA];

const OJAMA_COUNTRY: u32 = 90011152;

pub const LISTED_NAMES: &[u32] = &[OJAMA_COUNTRY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_BEAST, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_OJAMA], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Link Summoned from Ojamas (Beast Normal Monsters), with Ojama Country in the hand for its effects, Ojamas in the Graveyard to Special
// Summon, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (12482652, MZONE),
    (42941100, MZONE),
    (79335209, MZONE),
    (79335209, MZONE),
    (OJAMA_COUNTRY, HAND),
    (42941100, GRAVE),
    (12482652, GRAVE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, 3000);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, 1);
    e2.set_condition(duel, atkcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    e3.set_condition(duel, refcon);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn ffilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, OJAMA_COUNTRY)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(ffilter, a.e.get_handler_player(duel), LOCATION_FZONE, LOCATION_FZONE, 1, None)
}

fn refcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attack_target() == Some(a.e.get_handler(duel))
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_OJAMA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && !c.is_link_monster(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_FUSION) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_FUSION));
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(())
}
