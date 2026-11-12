// Chanbar, the Flashy Sportsknight
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 50643638;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Copies on the field and in the Graveyard, with Equip Spells (United We Stand, Megamorph) in the hand, Deck and Spell & Trap Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (56747793, HAND),
    (56747793, DECK),
    (56747793, DECK),
    (22046459, SZONE),
    (22046459, SZONE),
    (22046459, SZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tgdestg);
    e2.set_operation(duel, tgdesop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn atkdvalfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && (c.is_faceup(duel) || c.get_equip_target(duel).is_some())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    -400 * duel.get_matching_group_count(atkdvalfilter, e.get_handler_player(duel), LOCATION_ONFIELD, 0, None) as i32
}

fn equip_to_grave(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && c.is_able_to_grave(duel)
}

fn tgdestg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(equip_to_grave, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
                && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn tgdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, equip_to_grave, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
    {
        // The Lua errors on a nil target.
        unported!(duel, tc.is_none());
        if let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
        {
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| c.is_faceup(duel), tp, LOCATION_STZONE, 0, 2, None)
                && duel.get_mzone_count(tp, None) > 0
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| c.is_faceup(duel), tp, LOCATION_STZONE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty() && duel.destroy(tg, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
