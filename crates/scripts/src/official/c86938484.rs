// Shaddoll Construct
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 86938484;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_FLIP, Some(lc), st, tp)
    }
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SHADDOLL])
    }
}

// Link Summoned from Shaddoll Squamata (Flip), with El Shaddoll Winda in the Extra Deck to Fusion Summon from Squamata and Mokey Mokey (LIGHT).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, GRAVE),
    (94977269, EXTRA),
    (30328508, MZONE),
    (30328508, MZONE),
    (30328508, MZONE),
    (30328508, MZONE),
    (30328508, HAND),
    (30328508, HAND),
    (27288416, MZONE),
    (27288416, HAND),
    (27288416, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel)) && c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_able_to_grave_as_cost(duel)
}

fn mzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let sg = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && !sg.is_empty()
            && (ft > 0 || sg.is_exists(duel, mzfilter, 1, None)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let sg = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = if ft <= 0 {
        sg.filter_select(duel, tp, mzfilter, 1, 1, None)?
    } else {
        sg.select(duel, tp, 1, 1, None)?
    };
    if duel.send_to_grave(g, REASON_EFFECT)? != 0 && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
