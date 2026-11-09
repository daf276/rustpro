// Codebreaker Virus Berserker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 48736598;

// Codebreaker Virus Swordsmen on the field, whose vertical arrows make a mutual link with it when it is Link Summoned
// or revived by Monster Reborn, Zero Days to Special Summon from the hand and Graveyard, and the opponent's face-down
// Spells for its second effect to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, GRAVE),
    (8662794, MZONE),
    (8662794, MZONE),
    (8662794, HAND),
    (8662794, GRAVE),
    (84121193, MZONE),
    (84121193, MZONE),
    (CARD_MONSTER_REBORN, HAND),
    (CARD_MONSTER_REBORN, HAND),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_CODEBREAKER], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, p: u8, zones: [u32; 2]) -> bool {
    let zone = zones.get(usize::from(p)).copied().unwrap_or(0);
    c.is_set_card(duel, &[SET_CODEBREAKER]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, p, zone)
}

fn pointed_zones(duel: &Duel) -> [u32; 2] {
    [aux::get_mm_zones_pointed_to(duel, 0), aux::get_mm_zones_pointed_to(duel, 1)]
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zones = pointed_zones(duel);
    if chk == 0 {
        return Ok([tp, 1 - tp].into_iter().any(|p| {
            duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, p, zones),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                0,
                1,
                None,
            )
        }));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zones = pointed_zones(duel);
    let group = |duel: &Duel, p: u8| {
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, p, zones));
        duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, None)
    };
    let g = group(duel, tp).union(group(duel, 1 - tp));
    if g.is_empty() {
        return Ok(());
    }
    let ft = duel.get_location_count_full(0, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zones[0])
        + duel.get_location_count_full(1, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zones[1]);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let max = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        ft.min(2)
    };
    let sg = g.select(duel, tp, 1, max as u8, None)?;
    for tc in sg {
        let own = spfilter(tc, duel, e, tp, tp, zones);
        let p = if own && spfilter(tc, duel, e, tp, 1 - tp, zones) {
            if duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                1 - tp
            } else {
                tp
            }
        } else if own {
            tp
        } else {
            1 - tp
        };
        duel.special_summon_step_full(
            tc,
            0,
            tp,
            p,
            false,
            false,
            POS_FACEUP,
            zones.get(usize::from(p)).copied().unwrap_or(0),
        )?;
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_CODEBREAKER]) && c.is_linked(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None)
                && duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let count = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None) as u8;
    let g = duel.select_matching_card(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, count, None)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
