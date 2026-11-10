// Magic Drain
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 59344077;

// Set, against the opponent's Raigeki and Dark Hole in hand, which is also the Spell the opponent can discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, |duel, a| {
        a.rp != a.tp
            && a.re
                .is_some_and(|re| re.is_spell_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
            && duel.is_chain_negatable(a.ev as u8)
    });
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_chain_disablable(0) {
        let g = duel.get_matching_group(CardId::is_spell, tp, 0, LOCATION_HAND, None);
        duel.hint(HINT_SELECTMSG, 1 - tp, aux::stringid(ID, 0));
        let sel = if g.is_empty() {
            duel.select_option(1 - tp, &[1214])? + 1
        } else {
            duel.select_option(1 - tp, &[1213, 1214])?
        };
        if sel == 0 {
            duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DISCARD);
            let sg = g.select(duel, 1 - tp, 1, 1, None)?;
            duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
            duel.negate_effect(0);
            return Ok(());
        }
    }
    if let Some(re) = a.re
        && duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
