// Phantom Fortress Enterblathnir
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 95113856;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 9 monsters, against the opponent's monster, Spell/Trap and Graveyard cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (12197223, MZONE),
    (78783557, MZONE),
    (26202165, MZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn removable(duel: &Duel, tp: u8, location: u32) -> bool {
    duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, location, 1, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(removable(
            duel,
            tp,
            LOCATION_ONFIELD | LOCATION_HAND | LOCATION_GRAVE | LOCATION_DECK,
        ));
    }
    let locations = [LOCATION_ONFIELD, LOCATION_HAND, LOCATION_GRAVE, LOCATION_DECK];
    let options: Vec<_> = (1..)
        .zip(locations)
        .map(|(n, location)| (removable(duel, tp, location), aux::stringid(ID, n)))
        .collect();
    let op = duel.select_effect(tp, &options)?;
    a.e.set_label(duel, op.map_or(0, |op| op as i64));
    let loc = op.and_then(|op| locations.get(op - 1)).map_or(0, |&location| location);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, loc as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        op @ (1 | 3) => {
            let location = if op == 1 { LOCATION_ONFIELD } else { LOCATION_GRAVE };
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, location, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        2 => {
            let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None);
            if !g.is_empty() {
                let sg = g.random_select(duel, tp, 1);
                duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        4 => {
            let g = duel.get_decktop_group(1 - tp, 1);
            if !g.is_empty() {
                duel.disable_shuffle_check(true);
                duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
