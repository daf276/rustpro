// K9-17 "Ripper"
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 27420823;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

// K9-00 Lupis in the Deck to add, and an opponent's Elfnote Lucina in the hand to activate from there. Setting a K9 Quick-Play
// Spell needs K9-X Forced Release, which is not ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (91025875, DECK),
    (91025875, DECK),
    (13597785, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_CHAIN,
        CounterFilter::Chain(|duel, re, _, _| !re.is_monster_effect(duel)),
    );
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_K9]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_K9]) && c.is_quick_play_spell(duel) && c.is_ssetable(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        if duel.get_custom_activity_count(ID, 1 - tp, ACTIVITY_CHAIN) > 0
            && duel.is_existing_matching_card(aux::necro_valley_filter(setfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let sg = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(setfilter),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                1,
                None,
            )?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.sset(tp, sg)?;
            }
        }
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.re.is_some_and(|re| {
        a.ep == 1 - a.tp
            && re.is_monster_effect(duel)
            && trig_loc & (LOCATION_HAND | LOCATION_GRAVE) > 0
            && duel.is_chain_disablable(a.ev as u8)
    })
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
    Ok(false)
}
