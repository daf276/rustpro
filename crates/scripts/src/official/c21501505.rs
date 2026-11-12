// Cairngorgon, Antiluminescent Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 21501505;

// On the field with Xyz Import to attach Sangan, with other monsters for the opponent's Mage Power and Mind Control to target, and in the
// Extra Deck, Xyz Summoned with Level 4 Normal Monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (2311603, MZONE),
    (7359741, MZONE),
    (14602126, HAND),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition_mut(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &mut Duel, a: &Args) -> bool {
    let e = a.e;
    let Some(re) = a.re else { return false };
    if e == re || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS) else {
        return false;
    };
    let Some(tc) = g.get_first().filter(|_| g.len() == 1) else {
        return false;
    };
    e.set_label_object_group(duel, Some(tc.into()));
    tc.is_on_field(duel)
}

fn filter(c: CardId, duel: &Duel, ct: u32) -> bool {
    duel.check_mut(|duel| duel.check_chain_target(ct as u8, c))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp, ev) = (a.e, a.tp, a.ev);
    let label = duel.get_flag_effect_label(0, ID);
    let ct = label
        .filter(|label| i64::from(ev) == label >> 16)
        .map_or(ev, |label| (label & 0xffff) as u32);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel, ct));
    }
    let exception = e.get_label_object_group(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, ct), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c, duel| filter(c, duel, ct),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        exception,
    )?;
    let val = ct + ((ev + 1) << 16);
    if label.is_some() {
        duel.set_flag_effect_label(0, ID, val as i32);
    } else {
        duel.register_flag_effect_full(0, ID, RESET_CHAIN, 0, 1, val);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_target_card(a.ev as u8, tc.into());
    }
    Ok(())
}
