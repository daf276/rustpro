// I:P Masquerena
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 65741786;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// On the field with an Effect Monster, Mulcharmy Fuwalos, for the Link Summon of S:P Little Knight.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (42141493, MZONE), (29301450, EXTRA)];
// The hint of its other effect lands on the Link Monster summoned with it, which no check reads.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, |_, a| a.r == REASON_LINK);
    e2.set_operation(duel, indop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |sc, duel| sc.is_link_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_controler(duel, 1 - tp) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |sc, duel| sc.is_link_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.link_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3060);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp == 1 - e.get_owner_player(duel)));
    e1.set_owner_player(duel, a.ep);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    Ok(())
}
