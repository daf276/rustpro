// Demiurge Ema
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 26364381;
const TOKEN_HOMUNCULUS: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_HOMUNCULUS];

// In the hand with four monsters of 2400 or more ATK and 1000 DEF (Mobius, Caius, Thestalos and Astrazma Urania) to send, and
// Spells and Traps on both sides to destroy for the Tokens.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (4929256, DECK),
    (4929256, HAND),
    (9748752, DECK),
    (26205777, DECK),
    (18738846, MZONE),
    (CARD_TOON_WORLD, SZONE),
    (CARD_TOON_WORLD, SZONE | OPPONENT),
    (CARD_TOON_WORLD, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(TOKEN_HOMUNCULUS),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e1.set_count_limit(duel, 1, (ID, 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(
        duel,
        CATEGORY_DESTROY + CATEGORY_TOKEN + CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE,
    );
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, tkntg);
    e2.set_operation(duel, tknop);
    e2.set_count_limit(duel, 1, (ID, 1));
    c.register_effect(duel, e2);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_above(duel, 2400)
        && c.is_defense(duel, &[1000])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
        && c.is_able_to_grave_as_cost(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(
        selfspcostfilter,
        tp,
        LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK,
        0,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(g.len() >= 4 && duel.get_mzone_count(tp, Some(g)) > 0);
    }
    let sg = aux::select_unselect_group(duel, g, 4, 4, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE)?;
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn cansummontokentothisplayer(duel: &mut Duel, sum_player: u8, target_player: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 800, 800, 2, RACE_FAIRY, ATTRIBUTE_LIGHT);
    duel.is_player_can_special_summon_monster_full(sum_player, TOKEN_HOMUNCULUS, stats, POS_FACEUP_DEFENSE, target_player, 0)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let classes = sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel)));
    (
        classes == 2 && duel.get_mzone_count(tp, Some(sg)) > 0 && duel.get_mzone_count(1 - tp, Some(sg)) > 0,
        false,
    )
}

fn tkntg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_target_group(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    if chk == 0 {
        return Ok(g.len() >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && cansummontokentothisplayer(duel, tp, tp)
            && cansummontokentothisplayer(duel, tp, 1 - tp)
            && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_DESTROY)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), tg.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, PLAYER_ALL, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, PLAYER_ALL, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(e.get_handler(duel).into()), 1, tp, 1600);
    Ok(false)
}

fn tknop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty()
        && duel.destroy(tg, REASON_EFFECT)? > 0
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && cansummontokentothisplayer(duel, tp, tp)
        && cansummontokentothisplayer(duel, tp, 1 - tp)
    {
        duel.break_effect()?;
        if let Some(token1) = duel.create_token(tp, TOKEN_HOMUNCULUS) {
            duel.special_summon_step(token1, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
        if let Some(token2) = duel.create_token(tp, TOKEN_HOMUNCULUS) {
            duel.special_summon_step(token2, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)?;
        }
        if duel.special_summon_complete()? == 0 {
            return Ok(());
        }
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            c.update_attack(duel, 1600, None, None, None);
        }
    }
    Ok(())
}
