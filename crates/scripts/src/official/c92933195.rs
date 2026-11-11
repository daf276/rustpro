// Mist Valley Windmaster
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 92933195;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, handcon);
    e1.set_operation(duel, handop);
    c.register_effect(duel, e1);
}

fn handcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_HAND, 0) >= 5 && duel.get_field_group_count(a.tp, 0, LOCATION_HAND) >= 5
}

fn handop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = CardSet::default();
    for player in [tp, 1 - tp] {
        let ht = duel.get_field_group_count(player, LOCATION_HAND, 0) as u8;
        if ht >= 5 {
            duel.hint(HINT_SELECTMSG, player, HINTMSG_TOGRAVE);
            g.merge(duel.select_matching_card(player, aux::TRUE, player, LOCATION_HAND, 0, ht - 4, ht - 4, None)?);
        }
    }
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}
