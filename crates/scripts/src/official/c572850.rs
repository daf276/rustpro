// Tearlaments Scheiren
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 572850;

// The fusion parameters of the Tearlaments monsters that Fusion Summon when sent to the Graveyard.
pub struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (
            Some(duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_GRAVE, 0, None)),
            None,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        Some(Fusion::forced_handler(duel, e))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let c = a.e.get_handler(duel);
        duel.set_operation_info(
            0,
            CATEGORY_TODECK,
            Some(c.into()),
            1,
            a.tp,
            (LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE) as i32,
        );
        Ok(false)
    }
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
    let gg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE), None);
    if !gg.is_empty() {
        duel.hint_selection_full(gg, true);
    }
    let rg = sg.filter(duel, CardId::is_facedown, None);
    if !rg.is_empty() {
        duel.confirm_cards(1 - tp, rg)?;
    }
    duel.send_to_deck(*sg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
    let dg = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    let ct = dg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    if ct > 0 {
        duel.sort_deckbottom(tp, tp, ct as u16)?;
    }
    if dg.len() > ct {
        duel.sort_deckbottom(tp, 1 - tp, (dg.len() - ct) as u16)?;
    }
    sg.clear();
    Ok(())
}

// In the hand with a monster to send to the Graveyard, and copies in the Deck for the mill to send, to Fusion Summon Rulkallos
// with Kitkallos in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (26202165, HAND),
    (84330567, EXTRA),
    (92731385, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.get_current_phase() != PHASE_DAMAGE && a.e.get_handler(duel).is_reason(duel, REASON_EFFECT)
    });
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()))
            && duel.is_player_can_discard_deck(tp, 3));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 3);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        duel.break_effect()?;
        duel.discard_deck(tp, 3, REASON_EFFECT)?;
    }
    Ok(())
}
