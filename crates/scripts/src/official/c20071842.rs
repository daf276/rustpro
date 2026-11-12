// Heavy Interlock
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 20071842;
const BORRELOAD_RIOT_DRAGON: u32 = 7987191;

pub const LISTED_NAMES: &[u32] = &[BORRELOAD_RIOT_DRAGON];

// Borreload Riot Dragon in the hand, with Rokket monsters on the field and in the hand as materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (BORRELOAD_RIOT_DRAGON, HAND),
    (25554552, MZONE),
    (67127799, MZONE),
    (68464358, HAND),
    (48355999, HAND),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(BORRELOAD_RIOT_DRAGON)];

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, BORRELOAD_RIOT_DRAGON) && c.is_ritual_monster(duel)
    }

    fn lv(_: CardId, _: &Duel) -> u32 {
        8
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_matching_group(|c, duel| mfilter(c, duel, e), tp, LOCATION_MZONE | LOCATION_HAND, 0, None))
    }

    fn extraop(duel: &mut Duel, mg: CardSet, e: EffectId, tp: u8, _: CardId) -> Script<()> {
        let mut mg = mg;
        let rg = mg.filter(duel, |c, duel| mfilter(c, duel, e), None);
        let mut mat2 = CardSet::default();
        if !rg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            mat2 = rg.filter_select(duel, tp, |c, duel| mfilter(c, duel, e), 1, rg.len() as u8, None)?;
            mg = mg.difference(mat2);
        }
        duel.release_ritual_material(mg)?;
        duel.destroy(mat2, REASON_EFFECT)?;
        Ok(())
    }

    fn stage2(duel: &mut Duel, _: CardSet, e: EffectId, _: u8, tc: CardId) -> Script<()> {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(efilter));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e2.set_value(duel, Value::Card(indval));
        tc.register_effect_full(duel, e2, true);
        Ok(())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
}

fn mfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.has_level(duel)
        && c.is_set_card(duel, &[SET_ROKKET])
        && c.is_monster(duel)
        && c.is_destructable_full(duel, e)
}

fn efilter(duel: &Duel, _: EffectId, te: EffectId) -> bool {
    let tc = te.get_owner(duel);
    tc.is_special_summoned(duel)
        && tc.is_summon_location(duel, LOCATION_EXTRA)
        && te.is_monster_effect(duel)
        && te.is_activated(duel)
        && te.get_activate_location(duel) == LOCATION_MZONE
}

fn indval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(c.is_summon_location(duel, LOCATION_EXTRA))
}
