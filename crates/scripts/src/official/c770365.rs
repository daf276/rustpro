// Empress of Prophecy
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE};

const ID: u32 = 770365;

// In the Extra Deck, Xyz Summoned with Level 5 Spellcaster Normal Monsters (Leo Wizard, LaMoon, Ill Witch), with Spellbooks in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4392470, MZONE),
    (75850803, MZONE),
    (81686058, MZONE),
    (23314220, DECK),
    (23314220, DECK),
    (89739383, DECK),
    (89739383, DECK),
    (14824019, DECK),
    (14824019, DECK),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, c| (duel.get_overlay_count(c.get_controler(duel), 1, 0) * 300) as i32),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) >= 5)
    });
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.shuffle_deck(tp)?;
    duel.break_effect()?;
    duel.confirm_decktop(tp, 5)?;
    let g = duel.get_decktop_group(tp, 5);
    let ct = g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_SPELLBOOK]), None);
    let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if ct > 0 && !sg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = sg.select(duel, tp, 1, ct as u8, None)?;
        duel.hint_selection(dg);
        duel.destroy(dg, REASON_EFFECT)?;
        duel.break_effect()?;
    }
    duel.sort_decktop(tp, tp, 5)
}
