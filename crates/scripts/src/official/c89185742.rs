// Lightning, Dragon Ruler of Drafts
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 89185742;
const TEMPEST: u32 = 89399912;

pub const LISTED_NAMES: &[u32] = &[TEMPEST];

// A second copy and Red-Eyes Black Dragon (a Dragon) to discard, with Tempest in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74677422, HAND), (TEMPEST, DECK), (TEMPEST, DECK), (TEMPEST, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, ATTRIBUTE_WIND, TEMPEST>(duel, c);
}

// The effect of the four Level 3 Dragon Rulers that Special Summon their Level 7 Dragon Ruler from the Deck.
pub fn add_summon_effect<const ID: u32, const ATTRIBUTE: u32, const CODE: u32>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, DragonRuler::self_discard_cost::<ATTRIBUTE>);
    e1.set_target(duel, sptg::<CODE>);
    e1.set_operation(duel, spop::<CODE>);
    c.register_effect(duel, e1);
}

fn spfilter<const CODE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CODE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter::<CODE>(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter::<CODE>(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        cannot_attack_this_turn(duel, e.get_handler(duel), tc);
    }
    duel.special_summon_complete()?;
    Ok(())
}

pub fn cannot_attack_this_turn(duel: &mut Duel, c: CardId, tc: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
}
