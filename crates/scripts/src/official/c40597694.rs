// Scatter Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 40597694;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];

const RUBY: u32 = 76614340;

// Gem-Knight Ruby (Pyro) in the Extra Deck with Gem-Knight Garnets in the Deck, and an opponent's monster for the condition.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (RUBY, EXTRA),
    (RUBY, EXTRA),
    (91731841, DECK),
    (91731841, DECK),
    (91731841, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(RUBY)];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_type(duel, TYPE_FUSION) && c.is_set_card(duel, &[SET_GEM_KNIGHT]) && !c.is_race(duel, RACE_ROCK)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)), None)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(tc)) = (chk, tc) {
            let c = e.get_handler(duel);
            c.set_card_target(duel, tc);
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 1));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_target_range(duel, 1, 0);
            e1.set_target_card(duel, |duel, _, c| {
                !c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_location(duel, LOCATION_EXTRA)
            });
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
            aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_set_card(duel, &[SET_GEM_KNIGHT]));
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, LOCATION_DECK as i32);
        }
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_card_target_count(duel) > 0 {
        let targets = c.get_card_target(duel);
        let dg = duel.get_matching_group(|c, _| targets.is_contains(c), a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
