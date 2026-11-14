// Magical Musket Mastermind Caspar
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 3287359;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];
// In the Extra Deck, Link Summoned from Magical Musketeer Starfire (LIGHT Fiend) and another monster, with more Starfires in the hand and
// Deck to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (31629407, MZONE),
    (31629407, MZONE),
    (15025844, MZONE),
    (3134241, MZONE),
    (31629407, HAND),
    (31629407, DECK),
    (31629407, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, spsettg);
    e1.set_operation(duel, spsetop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_target_range(duel, LOCATION_HAND, 0);
    e2a.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e2b);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(lc), sumtype, tp) && c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp)
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, opp: u8) -> bool {
    (duel.get_mzone_count(opp, None) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, opp, 0xff))
        || c.is_ssetable_full(duel, false, Some(opp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && sg.is_exists(duel, |sc, duel| setfilter(sc, duel, e, tp, 1 - tp), 1, Some(c.into()))
}

fn rescon(duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> (bool, bool) {
    (
        sg.is_exists(duel, CardId::is_monster, 1, None) && sg.is_exists(duel, |c, duel| spfilter(c, duel, e, tp, sg), 1, None),
        false,
    )
}

fn musket_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        |c, duel| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        None,
    )
}

fn spsettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.get_mzone_count(tp, None) <= 0 {
            return Ok(false);
        }
        let g = musket_group(duel, tp);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            rescon(duel, e, tp, sg)
        }));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spsetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_mzone_count(tp, None) <= 0 {
        return Ok(());
    }
    let g = musket_group(duel, tp);
    if g.len() < 2 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, e, tp, sg), tp, aux::stringid(ID, 1))?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let spg = sg.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp, sg), 1, 1, None)?;
    if !spg.is_empty() && duel.special_summon(spg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let opp = 1 - tp;
        let Some(setc) = sg.difference(spg).get_first() else {
            return Ok(());
        };
        let b1 = duel.get_mzone_count(opp, None) > 0
            && setc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, opp, 0xff);
        let b2 = setc.is_ssetable_full(duel, false, Some(opp));
        let op = if b1 && b2 {
            duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
        } else {
            Some(if b1 { 1 } else { 2 })
        };
        match op {
            Some(1) => {
                duel.special_summon(setc, 0, tp, opp, false, false, POS_FACEDOWN_DEFENSE)?;
                duel.confirm_cards(tp, setc)?;
            }
            Some(2) => {
                duel.sset_full(tp, setc, opp, true)?;
            }
            _ => {}
        }
    }
    Ok(())
}
