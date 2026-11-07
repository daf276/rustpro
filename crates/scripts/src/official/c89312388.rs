// Elemental HERO Prisma
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE};

const ID: u32 = 89312388;

// On the field, with Fusion Monsters in the Extra Deck (Fusionist, Invoked Magellanica, Shining Neos Wingman) and their
// named materials (Petit Angel, Mystical Sheep #2, Aleister the Invoker, Neos) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (1641882, EXTRA),
    (48791583, EXTRA),
    (38142739, DECK),
    (83464209, DECK),
    (86120751, DECK),
    (56733747, EXTRA),
    (CARD_NEOS, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, coscost);
    e1.set_operation(duel, cosoperation);
    c.register_effect(duel, e1);
}

fn filter2(c: CardId, duel: &Duel, fc: CardId) -> bool {
    c.is_able_to_grave_as_cost(duel)
        && Fusion::material(duel, fc.get_original_code(duel))
            .iter()
            .any(|&code| c.is_code(duel, code))
}

fn filter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    !Fusion::material(duel, c.get_original_code(duel)).is_empty()
        && c.is_type(duel, TYPE_FUSION)
        && duel.is_existing_matching_card(|tc, duel| filter2(tc, duel, c), tp, LOCATION_DECK, 0, 1, None)
}

fn coscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter1(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, |c, duel| filter1(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let fc = g.get_first().unwrap_or_default();
    let cg = duel.select_matching_card(tp, |c, duel| filter2(c, duel, fc), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(cg, REASON_COST)?;
    e.set_label(duel, cg.get_first().map_or(0, |tc| i64::from(tc.get_code(duel))));
    Ok(false)
}

fn cosoperation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, e.get_label(duel) as i32);
    c.register_effect(duel, e1);
    Ok(())
}
