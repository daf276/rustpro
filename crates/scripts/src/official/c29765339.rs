// Watthydra
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 29765339;

pub const LISTED_SERIES: &[u16] = &[SET_WATT];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_WATT], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_THUNDER, Some(sc), sumtype, tp)
    }
}

// Copies on the field to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_REMOVE, LOCATION_DECK>);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && duel.get_attack_target().is_none()
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tg) = g.get_first() else { return Ok(()) };
    duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_REMOVED);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_reset_full(
        duel,
        RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
        2,
    );
    e1.set_condition(duel, thcon);
    e1.set_operation(duel, thop);
    e1.set_label(duel, 0);
    tg.register_effect(duel, e1);
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.get_label(duel) == 1 {
        let c = e.get_handler(duel);
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    } else {
        e.set_label(duel, 1);
    }
    Ok(())
}
