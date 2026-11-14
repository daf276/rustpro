// Gem-Knight Phantom Quartz
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 24484270;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT, SET_GEM];

const ZIRCONIA: u32 = 8692301;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GEM], Some(lc), st, tp)
    }
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_GEM_KNIGHT])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    // The Lua's NecroValleyFilter takes only Card.IsFaceup: the second filter it is given, Card.IsAbleToDeck, is ignored.
    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, aux::necro_valley_filter(CardId::is_faceup));
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None)),
            None,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(tc)) = (chk, tc) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3207);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_TODECK, None, 0, a.tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
        }
        Ok(true)
    }
}

// Link Summoned from Gem-Knight Garnet and Lapis, with Gem-Knight Nepyrim in the Deck to search and Garnet and Lapis (Rock) in the
// Graveyard to shuffle for Gem-Knight Zirconia in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ZIRCONIA, EXTRA),
    (91731841, MZONE),
    (91731841, MZONE),
    (99645428, MZONE),
    (99645428, MZONE),
    (91731841, GRAVE),
    (99645428, GRAVE),
    (51831560, DECK),
    (51831560, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(ZIRCONIA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
