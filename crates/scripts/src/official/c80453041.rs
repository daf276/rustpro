// Phantom of Yubel
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 80453041;

// Copies in the hand and Deck as the "Yubel" monster for the effect, and the opponent's Effect Monster to respond to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (ID, DECK), (85976588, MZONE | OPPONENT)];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_YUBEL], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| fiendfilter(c, duel)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::Int(0));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(
            CardId::is_able_to_deck_or_extra_as_cost,
            tp,
            LOCATION_MZONE | LOCATION_HAND | LOCATION_GRAVE,
            0,
            None,
        )
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        let (fu, fd) = g.split(duel, CardId::is_faceup, None);
        if !fu.is_empty() {
            duel.hint_selection_full(fu, true);
        }
        if !fd.is_empty() {
            duel.confirm_cards(1 - tp, fd)?;
        }
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)
            .map(|_| ())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e4.set_cost(duel, Cost::self_tribute);
    e4.set_target(duel, chngtg);
    e4.set_operation(duel, chngop);
    c.register_effect(duel, e4);
}

fn fiendfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_attack(duel, &[0]) && c.is_defense(duel, &[0])
}

fn yubelfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUBEL]) && c.is_monster(duel) && (c.is_faceup(duel) || !c.is_on_field(duel))
}

fn chngtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            yubelfilter,
            a.tp,
            LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK,
            0,
            1,
            Some(c.into()),
        ));
    }
    Ok(false)
}

fn chngop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p = 1 - a.tp;
    duel.hint(HINT_SELECTMSG, p, HINTMSG_DESTROY);
    let g = duel.select_matching_card(p, yubelfilter, p, LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.destroy_full(g, REASON_EFFECT, LOCATION_GRAVE, p)?;
    }
    Ok(())
}
