// Destiny HERO - Destro-Dogma
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, OPPONENT};

const ID: u32 = 25158975;

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK) || c.is_race(duel, RACE_WARRIOR)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(
            0,
            CATEGORY_TODECK,
            None,
            1,
            a.tp,
            (LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE) as i32,
        );
        Ok(false)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_deck);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the Graveyard, Special Summoned by banishing three Destiny HERO monsters (DARK Warriors) from the Graveyard, with Witchcrafter
// Vice-Madame in the Extra Deck to Fusion Summon with Witchcrafter Genni (DARK Fusion Monster) when the opponent's Pot of Greed is activated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (56570271, GRAVE),
    (56570271, GRAVE),
    (56570271, GRAVE),
    (9603252, EXTRA),
    (64756282, HAND),
    (64756282, HAND),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
    });
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_FUSION_SUMMON + CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_race(duel, RACE_WARRIOR) || c.is_attribute(duel, ATTRIBUTE_DARK))
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, Some(c.into()));
    g.len() >= 3 && duel.get_mzone_count(tp, Some(g)) > 0 && aux::select_unselect_group_check(duel, g, 3, 3, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, a.c.map(CardSet::from));
    let sg = aux::select_unselect_group_full(duel, g, 3, 3, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE, None, true)?;
    if sg.len() == 3 {
        e.set_label_object_group(duel, Some(sg));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel)
        && sg.len() == 3
    {
        duel.remove(sg, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 2000);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint_full(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 3)), RESET_PHASE | PHASE_STANDBY, 1);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, |duel, a| {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.damage(1 - a.tp, 2000, REASON_EFFECT)?;
        Ok(())
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY);
    duel.register_effect(e1, tp);
    Ok(())
}
