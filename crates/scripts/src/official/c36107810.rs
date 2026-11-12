// Night's End Sorcerer
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 36107810;

// Revived by Monster Reborn from the Graveyard, with Sangan on the opponent's field and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (83764718, HAND),
    (83764718, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 2, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
