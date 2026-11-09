// Beyond the Pendulum
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 22125101;
const JUNOLDO: u32 = 10266279;
const NERVEDO: u32 = 17473466;
const JUPREDO: u32 = 43871165;
const FLASH_KNIGHT: u32 = 17390179;
const VIDOLIUM: u32 = 70488851;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned from Nervedo and Sangan with Flash Knight in the Deck to search, and on the field with
// Junoldo and Vidolium as scales and Flash Knight, Jupredo and Junoldo in hand to Pendulum Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (NERVEDO, MZONE),
    (26202165, MZONE),
    (JUNOLDO, LOCATION_PZONE as u16),
    (VIDOLIUM, LOCATION_PZONE as u16),
    (FLASH_KNIGHT, HAND),
    (JUPREDO, HAND),
    (FLASH_KNIGHT, DECK),
    (FLASH_KNIGHT, DECK),
];
// The destruction needs it Link Summoned into an Extra Monster Zone, then two Pendulum Summoned monsters in the zones it points to.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_link_summoned(duel) && c.is_in_extra_mzone(duel)
    });
    e1.set_cost(duel, Cost::pay_lp::<1200>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DISABLE);
    e2.set_target_range(duel, LOCATION_PZONE, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    // The Lua table {e1, e2} is e1 and e2 chained as label objects.
    e1.set_label_object(duel, Some(e2));
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_label_object(duel, Some(e1));
    e0.set_operation(duel, checkop);
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e0, tp);
    Ok(())
}

fn psfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_pendulum_summoned(duel)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(e1) = e.get_label_object(duel)
        && a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| psfilter(c, duel, tp), 1, None))
    {
        e1.reset(duel);
        if let Some(e2) = e1.get_label_object(duel) {
            e2.reset(duel);
        }
        e.reset(duel);
    }
    Ok(())
}

fn pendfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    psfilter(c, duel, tp) && c.has_level(duel)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let eg = a.eg.unwrap_or_default();
    if eg.contains(c) {
        return false;
    }
    let lg = aux::zptgroup(duel, eg, aux::TRUE, c).filter(duel, |c, duel| pendfilter(c, duel, tp), None);
    lg.get_class_count(duel, |c, duel| i64::from(c.get_original_level(duel))) > 1
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = duel.get_target_cards(a.e)
        && !g.is_empty()
    {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
