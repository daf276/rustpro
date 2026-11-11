// Kagemusha of the Six Samurai
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 1498130;

// On the field beside Six Samurai monsters (Legendary Six Samurai - Kageki), with the opponent's Mage Power and Mind Control to target them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (2511717, MZONE),
    (2511717, MZONE),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, tgcon);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(ev, CHAININFO_TARGET_CARDS) else {
        return false;
    };
    let Some(tc) = g.get_first().filter(|_| g.len() == 1) else {
        return false;
    };
    let c = a.e.get_handler(duel);
    if tc == c
        || tc.get_controler(duel) != tp
        || tc.is_facedown(duel)
        || !tc.is_location(duel, LOCATION_MZONE)
        || !tc.is_set_card(duel, &[SET_SIX_SAMURAI])
    {
        return false;
    }
    duel.check_mut(|duel| duel.check_chain_target(ev, c))
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.change_target_card(a.ev as u8, c.into());
    }
    Ok(())
}
