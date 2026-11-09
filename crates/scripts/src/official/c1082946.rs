// Pyro Clock of Destiny
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};
use engine::Operation;

const ID: u32 = 1082946;

// Swords of Revealing Light, which grants the effect this card advances, and the opponent's monsters it flips.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (72302403, HAND), (72302403, HAND), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn has_effect(c: CardId, duel: &Duel) -> bool {
    c.is_has_effect(duel, ID).is_some()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(has_effect, a.tp, LOCATION_ALL, LOCATION_ALL, 1, None));
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let g = duel.select_matching_card(tp, has_effect, tp, LOCATION_ALL, LOCATION_ALL, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let sel = tc.get_card_effect(duel, ID);
    let turne = match sel.len() {
        0 => return Ok(()),
        1 => sel.first().copied(),
        _ => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_EFFECT);
            let descriptions: Vec<u64> = sel.iter().map(|te| te.get_description(duel)).collect();
            let op = duel.select_option(tp, &descriptions)?;
            usize::try_from(op).ok().and_then(|op| sel.get(op)).copied()
        }
    };
    let Some(turne) = turne else { return Ok(()) };
    let op = turne.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        let args = Args {
            e: turne,
            tp: turne.get_owner_player(duel),
            ep: 0,
            ev: ID,
            r: 0,
            rp: 0,
            ..Args::default()
        };
        op(duel, &args)?;
    }
    Ok(())
}
