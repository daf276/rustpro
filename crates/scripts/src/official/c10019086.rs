// Tri-Brigade Arms Bucephalus II
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 10019086;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(5);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACES_BEAST_BWARRIOR_WINGB, Some(lc), sumtype, tp)
    }

    fn spcon(duel: &Duel, e: EffectId, tp: u8) -> bool {
        splimit(duel, e, tp)
    }
}

// On the field to attack and to be destroyed by the opponent's Raigeki, with Beast Extra Deck monsters to
// send. Its Link Summon needs three Tri-Brigade Spells/Traps in the Graveyard, and none is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (4796100, EXTRA),
    (90303227, EXTRA),
    (11765832, EXTRA),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(|duel, e, _, sp, _, _, _| splimit(duel, e, sp)));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, sucop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRI_BRIGADE]) && c.is_spell_trap(duel)
}

fn splimit(duel: &Duel, e: EffectId, sp: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || duel.is_existing_matching_card(spcostfilter, sp, LOCATION_GRAVE, 0, 3, None)
}

fn sucop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.eg
        .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, tp), 1, None))
    {
        duel.set_chain_limit_till_chain_end(|_, _, rp, tp, _| rp == tp, 0);
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    if c.is_relate_to_effect(duel, e) && c.is_able_to_remove(duel) {
        duel.remove(c, POS_FACEUP, REASON_EFFECT)?;
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_EXTRA)
}
