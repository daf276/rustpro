// Fire Fighting Daruma Doll
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 64514892;

// On the field and in the Extra Deck, Link Summoned from Vorse Raiders (Beast-Warriors), with Mystical Space Typhoon on both sides to destroy, one in the
// Graveyard to add and the opponent's Decoys to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, GRAVE),
    (5318639, SZONE),
    (5318639, SZONE | OPPONENT),
    (20240828, MZONE | OPPONENT),
    (20240828, MZONE | OPPONENT),
    (20240828, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACES_BEAST_BWARRIOR_WINGB, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_condition(duel, aux::bdocon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
}

fn atkval(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(cfilter, 0, LOCATION_MZONE, LOCATION_MZONE, None) as i32 * 100
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g1 = duel
        .select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g2 = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    g1.merge(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g1), 2, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
