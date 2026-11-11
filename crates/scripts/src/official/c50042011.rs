// Mementotlan Shleepy
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 50042011;
const MEMENTOMICTLAN_TECUHTLICA_CREATION_KING: u32 = 14529511;
const MEMENTOTLAN_ANGWITCH: u32 = 54550967;
const MEMENTOTLAN_TATSUNOOTOSHIGO: u32 = 81677154;
const DARK_HOLE: u32 = 53129443;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEMENTO])
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In hand with the opponent's Dark Hole to Special Summon it, and Mementotlan monsters on the field and in the hand to Fusion Summon
// Creation King in the Extra Deck; destroyed by Dark Hole to send a Memento card from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (MEMENTOMICTLAN_TECUHTLICA_CREATION_KING, EXTRA),
    (MEMENTOTLAN_ANGWITCH, MZONE),
    (MEMENTOTLAN_TATSUNOOTOSHIGO, MZONE),
    (MEMENTOTLAN_ANGWITCH, DECK),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| !duel.is_phase(PHASE_DAMAGE));
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_BATTLE | REASON_EFFECT));
    e4.set_target(duel, tgtg);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROY);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_EFFECT)
        && (c.is_previous_location(duel, LOCATION_MZONE) || (c.is_monster(duel) && !c.is_previous_location(duel, LOCATION_ONFIELD)))
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.eg.unwrap_or_default().filter(duel, checkfilter, None);
    if g.is_empty() {
        return Ok(());
    }
    for p in 0..2 {
        if g.is_exists(duel, |c, duel| c.is_previous_controler(duel, p), 1, None) && !duel.has_flag_effect(p, ID) {
            duel.register_flag_effect(p, ID, RESET_PHASE + PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
