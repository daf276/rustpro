// Giant Mimighoul
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 16955631;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters (Key Mace, White Duston), with Mimighoul Master in the Deck to search and
// the opponent's Sangan face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (1929294, MZONE),
    (1929294, MZONE),
    (3557275, MZONE),
    (3557275, MZONE),
    (55537983, DECK),
    (55537983, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_MIMIGHOUL])
    });
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_facedown, a.e.get_handler_player(duel), 0, LOCATION_MZONE, 1, None)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel));
    }
    let ct = duel.get_matching_group_count(CardId::is_facedown, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct as u8, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, count as i32 * 1000);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() && duel.destroy(tg, REASON_EFFECT)? > 0 {
        let ct = duel.get_operated_group().len();
        duel.damage(1 - tp, ct as u32 * 1000, REASON_EFFECT)?;
    }
    Ok(())
}
