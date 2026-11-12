// Trickstar Delfiendium
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, REMOVED};

const ID: u32 = 3792766;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Trickstar Nightshades (which return from the Graveyard into its linked zones), with banished
// Nightshades to add, and opponent's Link Monsters (Sky Striker Ace - Hayate) to count.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (86825114, MZONE),
    (86825114, MZONE),
    (86825114, MZONE),
    (86825114, MZONE),
    (86825114, REMOVED),
    (86825114, REMOVED),
    (86825114, REMOVED),
    (8491308, MZONE | OPPONENT),
    (8491308, MZONE | OPPONENT),
];
// The effect needs the attack of a Link Summoned Delfiendium with a Trickstar returned into a linked zone, which random play rarely reaches.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && c.is_faceup(duel) && c.is_monster(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, thconfilter, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    let ct = duel.get_matching_group_count(CardId::is_link_monster, tp, 0, LOCATION_MZONE, None) as u8;
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel
        .select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, ct, None)?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(
        0,
        CATEGORY_ATKCHANGE,
        Some(a.e.get_handler(duel).into()),
        1,
        tp,
        g.len() as i32 * 1000,
    );
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
        if ct == 0 {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct as i32 * 1000);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
