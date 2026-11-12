// Encode Talker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 6622715;

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters, which then battle stronger Elfnote Lucinas of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_CONFIRM);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

// The attacker and its battle target, the first being the controller's.
fn battlers(duel: &Duel, tp: u8) -> Option<(CardId, CardId)> {
    let a = duel.get_attacker()?;
    let b = a.get_battle_target(duel)?;
    Some(if a.is_controler(duel, 1 - tp) { (b, a) } else { (a, b) })
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    unported!(duel, duel.get_attacker().is_none());
    let Some((a, b)) = battlers(duel, a.tp) else { return false };
    a.get_controler(duel) != b.get_controler(duel)
        && lg.contains(a)
        && a.is_faceup(duel)
        && b.is_faceup(duel)
        && b.get_attack(duel) > a.get_attack(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    unported!(duel, duel.get_attacker().is_none());
    let Some((a, b)) = battlers(duel, tp) else { return Ok(()) };
    if a.is_relate_to_battle(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        a.register_effect(duel, e1);
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_label_object_group(duel, Some(b.into()));
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_operation(duel, atkop);
    e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
    c.register_effect(duel, e3);
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(b) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if b.is_facedown(duel) || !b.is_relate_to_battle(duel) {
        return Ok(());
    }
    let mut lg = c.get_linked_group(duel);
    lg.insert(c);
    let tc = if lg.len() == 1 {
        lg.get_first()
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        lg.select(duel, tp, 1, 1, None)?.get_first()
    };
    let Some(tc) = tc else { return Ok(()) };
    duel.hint_selection(tc);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, b.get_attack(duel));
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    Ok(())
}
