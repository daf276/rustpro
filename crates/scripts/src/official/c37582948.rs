// Live☆Twin Sunny's Snitch
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 37582948;

pub const LISTED_SERIES: &[u16] = &[SET_EVIL_TWIN, SET_LIVE_TWIN];

// Live☆Twin Ki-sikil and Lil-la to search, Evil★Twin Ki-sikil on the field, and the opponent's Elfnote Lucinas to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (36326160, DECK),
    (73810864, DECK),
    (9205573, MZONE),
    (9205573, MZONE),
    (13597785, HAND | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_RECOVER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, damcon);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIVE_TWIN]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let cg = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if !cg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = cg.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None))
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_EVIL_TWIN])),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    if duel.recover(a.tp, 200, REASON_EFFECT)? > 0 {
        duel.damage(1 - a.tp, 200, REASON_EFFECT)?;
    }
    Ok(())
}
