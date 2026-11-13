// Dark Infant @Ignister
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 74567889;
const AI_LAND: u32 = 59054773;

pub const LISTED_NAMES: &[u32] = &[AI_LAND];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_link_monster(duel) && c.is_set_card_full(duel, &[SET_IGNISTER], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from an Ignister monster, and on the field, with Ignister A.I.Land to search and the opponent's Cyberse monsters of 2300 ATK
// with effects to chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (3723262, MZONE),
    (3723262, MZONE),
    (3723262, MZONE),
    (42429678, MZONE),
    (AI_LAND, DECK),
    (AI_LAND, DECK),
    (ID, MZONE),
    (61245672, MZONE | OPPONENT),
    (46947713, MZONE | OPPONENT),
    (15844566, MZONE | OPPONENT),
    (98506199, MZONE | OPPONENT),
    (86066372, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, AI_LAND) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && re.is_monster_effect(duel)
        && rc.get_base_attack(duel) == 2300
        && rc.is_race(duel, RACE_CYBERSE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let zone = a.e.get_handler(duel).get_linked_zone(duel) & ZONES_MMZ;
    if chk == 0 {
        return Ok(duel.get_location_count_full(a.tp, LOCATION_MZONE, a.tp, LOCATION_REASON_TOFIELD, zone) > 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let zone = c.get_linked_zone(duel) & ZONES_MMZ;
    if c.is_relate_to_effect(duel, a.e) && duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0 {
        duel.move_sequence(c, zone.trailing_zeros() as u8)?;
        if c.is_faceup(duel) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            let att = c.announce_another_attribute(duel, tp)?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
            e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
            e1.set_value(duel, att as i32);
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
