// Vulcan the Divine
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 98012938;

// In the Extra Deck, Synchro Summoned with Tune Warrior (Level 3 Tuner) and Mountain Warrior (Level 3), with the opponent's Protector of the Throne to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (74093656, MZONE),
    (74093656, MZONE),
    (4931562, MZONE),
    (4931562, MZONE),
    (10071456, MZONE | OPPONENT),
    (10071456, MZONE | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SYNCHRO
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    if duel.is_existing_target(aux::faceup_filter(CardId::is_able_to_hand), tp, LOCATION_ONFIELD, 0, 1, None)
        && duel.is_existing_target(aux::faceup_filter(CardId::is_able_to_hand), tp, 0, LOCATION_ONFIELD, 1, None)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let mut g1 = duel
            .select_target(tp, aux::faceup_filter(CardId::is_able_to_hand), tp, LOCATION_ONFIELD, 0, 1, 1, None)?
            .unwrap_or_default();
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let g2 = duel
            .select_target(tp, aux::faceup_filter(CardId::is_able_to_hand), tp, 0, LOCATION_ONFIELD, 1, 1, None)?
            .unwrap_or_default();
        g1.merge(g2);
        duel.set_operation_info(0, CATEGORY_TOHAND, Some(g1), 2, 0, 0);
    }
    Ok(false)
}

fn hfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_faceup(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    let g = g.filter(duel, |c, duel| hfilter(c, duel, e), None);
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    for tc in g.iter() {
        if tc.is_location(duel, LOCATION_HAND) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
            e1.set_target_range(duel, 1, 0);
            e1.set_value(duel, Value::EffectPlayer(aclimit));
            e1.set_label(duel, i64::from(tc.get_code(duel)));
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
    }
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_code(duel, e.get_label(duel) as u32)
}
