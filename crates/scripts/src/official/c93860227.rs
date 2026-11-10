// Necroquip Princess
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 93860227;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, _, _, _| eqmatfilter(c, duel)),
        FusionMaterial::Filter(fiendmatfilter),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::Int(0));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// Black Metal Dragon in the hand to equip itself to Red-Eyes Black Dragon, with Sangan (Fiend) as the other material; Dictator of D.
// and Blue-Eyes White Dragon in the hand to send a monster from the hand as a cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (93969023, HAND),
    (93969023, HAND),
    (74677422, MZONE),
    (74677422, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (66961194, HAND),
    (CARD_BLUEEYES_W_DRAGON, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, effcon);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn eqmatfilter(c: CardId, duel: &Duel) -> bool {
    c.get_equip_group(duel).is_exists(duel, CardId::is_monster_card, 1, None)
}

fn fiendmatfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_monster_card(duel) && c.is_race_full(duel, RACE_FIEND, Some(fc), sumtype, tp)
}

fn effconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_reason(duel, REASON_COST) && c.is_previous_location(duel, LOCATION_HAND)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, effconfilter, 1, None) && a.re.is_some_and(|re| re.is_activated(duel))
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    effconfilter(c, duel)
        && c.check_unique_on_field(duel, tp)
        && (c.is_controler(duel, tp) || c.is_able_to_change_controler(duel))
        && !c.is_forbidden(duel)
        && c.is_location(duel, LOCATION_GRAVE)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    let b1 = duel.get_location_count(tp, LOCATION_SZONE) > 0 && eg.is_exists(duel, |c, duel| eqfilter(c, duel, tp), 1, None);
    let b2 = duel.is_player_can_draw(tp, 1);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.set_target_card(eg);
        e.set_category(duel, CATEGORY_EQUIP);
        duel.set_operation_info(0, CATEGORY_EQUIP, Some(eg), 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        let c = e.get_handler(duel);
        if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.get_target_cards(e).unwrap_or_default();
        let ec = g.filter_select(duel, tp, |c, duel| eqfilter(c, duel, tp), 1, 1, None)?.get_first();
        if let Some(ec) = ec
            && duel.equip(tp, ec, c)?
        {
            let e1 = duel.create_effect(ec);
            e1.set_type(duel, EFFECT_TYPE_EQUIP);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 500);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            ec.register_effect(duel, e1);
            let e2 = duel.create_effect(ec);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_EQUIP_LIMIT);
            e2.set_label_object_group(duel, Some(c.into()));
            e2.set_value(duel, Value::Card(super::c38423248::equip_limit));
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            ec.register_effect(duel, e2);
        }
    } else if op == 2 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
