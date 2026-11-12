use std::error::Error;
use std::ops::Range;
use std::sync::LazyLock;

use carddata::{Card, CardDatabase, Deck, TYPE_MONSTER, TYPE_NORMAL, TYPE_SPELL, TYPE_TRAP};
use engine::constants::{
    EFFECT_CHANGE_BATTLE_STAT, EFFECT_FLAG_SINGLE_RANGE, EFFECT_TYPE_CONTINUOUS, EFFECT_TYPE_FIELD, EFFECT_TYPE_SINGLE, EVENT_DRAW,
    LOCATION_MZONE, TYPE_EFFECT, TYPE_QUICKPLAY,
};
use engine::{
    Action, ActionError, CardId, DUEL_MODE_MR5, Duel, DuelError, DuelOptions, IdleCommand, InitialEffect, Library, PHASE_DAMAGE,
    PHASE_DAMAGE_CAL, PlayerOptions, Prompt, Status,
};

const CODES: std::ops::RangeInclusive<u32> = 1..=40;
const TYPE_FLIP: u32 = 0x20_0000;

// The cards phase 2a ports, with their type, level, attack and defense.
const EFFECT_CARDS: [(u32, u32, u32, i32, i32); 12] = [
    (55144522, TYPE_SPELL, 0, 0, 0),
    (84257639, TYPE_SPELL, 0, 0, 0),
    (46130346, TYPE_SPELL, 0, 0, 0),
    (12580477, TYPE_SPELL, 0, 0, 0),
    (53129443, TYPE_SPELL, 0, 0, 0),
    (83764718, TYPE_SPELL, 0, 0, 0),
    (5318639, TYPE_SPELL | TYPE_QUICKPLAY, 0, 0, 0),
    (4206964, TYPE_TRAP, 0, 0, 0),
    (44095762, TYPE_TRAP, 0, 0, 0),
    (54652250, TYPE_MONSTER | TYPE_EFFECT | TYPE_FLIP, 2, 450, 600),
    (26202165, TYPE_MONSTER | TYPE_EFFECT, 3, 1000, 600),
    (14558127, TYPE_MONSTER | TYPE_EFFECT, 3, 0, 1800),
];

static CARDS: LazyLock<CardDatabase> = LazyLock::new(|| {
    let monster = |code: u32| {
        let (attack, defense) = ((code * 100) as i32, (code % 7 * 300) as i32);
        Card {
            code,
            card_type: TYPE_MONSTER | TYPE_NORMAL,
            level: code % 8 + 1,
            attack,
            defense,
            ..Card::default()
        }
    };
    let effect = |(code, card_type, level, attack, defense)| Card {
        code,
        card_type,
        level,
        attack,
        defense,
        ..Card::default()
    };
    CODES.map(monster).chain(EFFECT_CARDS.map(effect)).collect()
});

fn new_duel() -> Result<Duel, DuelError> {
    let player = PlayerOptions::default();
    Duel::new(DuelOptions {
        seed: [1, 2, 3, 4],
        flags: DUEL_MODE_MR5,
        team1: player,
        team2: player,
    })
}

fn started_duel(main: &[u32], library: &'static Library) -> Result<Duel, Box<dyn Error>> {
    let mut duel = new_duel()?;
    duel.load_library(library);
    let deck = Deck {
        main: main.to_vec(),
        extra: Vec::new(),
    };
    for team in 0..2 {
        duel.add_deck(team, &deck, &CARDS)?;
    }
    duel.set_trace(true);
    duel.start();
    Ok(duel)
}

// The first decision taken in each situation a clone has to carry over.
#[derive(Debug, Default)]
struct Situations {
    mid_tribute: Option<usize>,
    damage_step: Option<usize>,
    mid_chain: Option<usize>,
    // A prompt of a cost, target or operation function, which reruns from its choice log once answered.
    script_prompt: Option<usize>,
}

// Picks a legal action by a hash of the decision number, so the choice depends on nothing else.
fn play(duel: &mut Duel, decisions: Range<usize>) -> Result<Situations, Box<dyn Error>> {
    let mut status = duel.process();
    let mut situations = Situations::default();
    for decision in decisions {
        if status == Status::End {
            break;
        }
        let legal = duel.legal_actions();
        let action = *legal
            .get((decision.wrapping_mul(0x9e37_79b9) >> 8) % legal.len().max(1))
            .ok_or("no legal action")?;
        if matches!(action, Action::Card(_)) && matches!(duel.prompt(), Some(Prompt::Tribute { max: 2.., .. })) {
            situations.mid_tribute.get_or_insert(decision + 1);
        }
        if [PHASE_DAMAGE, PHASE_DAMAGE_CAL].contains(&duel.phase()) {
            situations.damage_step.get_or_insert(decision);
        }
        if duel.get_current_chain() > 0 {
            situations.mid_chain.get_or_insert(decision);
            // Outside a chain only attack targets are chosen with this prompt.
            if matches!(duel.prompt(), Some(Prompt::Card { .. })) {
                situations.script_prompt.get_or_insert(decision);
            }
        }
        status = duel.apply(action)?;
    }
    Ok(situations)
}

const DECISIONS: usize = 20_000;

// The situations a deck must reach and be cloned in.
type ClonePoints = fn(Situations) -> [Option<usize>; 2];

#[test]
fn should_finish_identically_when_cloned_at_a_prompt() {
    let effects = EFFECT_CARDS.iter().flat_map(|card| [card.0; 2]).chain(1..=16).collect();
    let decks: [(Vec<u32>, ClonePoints); 2] = [
        (CODES.collect(), |s| [s.mid_tribute, s.damage_step]),
        (effects, |s| [s.mid_chain, s.script_prompt]),
    ];
    for (main, required) in decks {
        let mut full = started_duel(&main, &scripts::LIBRARY).unwrap();
        let situations = play(&mut full, 0..DECISIONS).unwrap();
        assert!(full.result().is_some());
        // Cloned into after the first prompt: a duel played to the end, with its spare snapshot.
        let mut pooled = full.clone();
        for prompt in [0, 1, 17, 100, 333].into_iter().chain(required(situations).map(Option::unwrap)) {
            let mut duel = started_duel(&main, &scripts::LIBRARY).unwrap();
            play(&mut duel, 0..prompt).unwrap();
            let mut clone = duel.clone();
            pooled.clone_from(&duel);
            for duel in [&mut duel, &mut clone, &mut pooled] {
                play(duel, prompt..DECISIONS).unwrap();
                assert_eq!(
                    (duel.take_trace(), duel.result()),
                    (full.clone().take_trace(), full.result()),
                    "cloned at prompt {prompt}"
                );
            }
        }
    }
}

#[test]
fn should_hash_a_state_alike_whatever_order_reached_it() {
    let spells: Vec<u32> = EFFECT_CARDS
        .iter()
        .filter(|card| card.1 & TYPE_SPELL != 0)
        .map(|card| card.0)
        .collect();
    let mut start = started_duel(&spells, &scripts::LIBRARY).unwrap();
    start.process();
    // Sets the cards at these indices of the hand's settable spells into these Spell & Trap Zones, in order, chaining nothing.
    let set = |sets: [(u16, u8); 2]| {
        let mut duel = start.clone();
        for (index, zone) in sets {
            let action = Action::IdleCommand(IdleCommand::SpellSet(index));
            let (card, _) = duel.action_card(action);
            duel.apply(action).unwrap();
            assert!(card.is_some() && duel.prompt_card() == card);
            duel.apply(Action::Place(zone)).unwrap();
            pass_chains(&mut duel).unwrap();
        }
        duel.state_hash()
    };
    let first = set([(0, 8), (0, 9)]);
    assert!(first.is_some());
    assert_eq!(set([(1, 9), (0, 8)]), first);
    assert_ne!(set([(1, 8), (0, 9)]), first);
    assert_ne!(start.state_hash(), first);
}

fn pass_chains(duel: &mut Duel) -> Result<(), ActionError> {
    while matches!(duel.prompt(), Some(Prompt::Chain { .. })) {
        duel.apply(Action::Chain(None))?;
    }
    Ok(())
}

#[test]
fn should_hash_a_chain_window_alike_whatever_order_reached_it_and_apart_by_its_chain() {
    let typhoon = 5318639;
    let (pot, keto) = (55144522, 84257639);
    let mut start = started_duel(&[typhoon, pot, keto, 12580477, 53129443], &scripts::LIBRARY).unwrap();
    start.process();
    let kind_on = |duel: &Duel, code: u32, kind: fn(Action) -> bool| {
        let picks = |action: Action| duel.action_card(action).0.is_some_and(|card| card.get_code(duel) == code);
        duel.legal_actions()
            .into_iter()
            .find(|&action| kind(action) && picks(action))
            .unwrap()
    };
    // Sets two spells into these Spell & Trap Zones in order, then activates Mystical Space Typhoon on the one in `target`.
    let chain_window = |sets: [(u32, u8); 2], target: u32| {
        let mut duel = start.clone();
        for (code, zone) in sets {
            duel.apply(kind_on(&duel, code, |action| {
                matches!(action, Action::IdleCommand(IdleCommand::SpellSet(_)))
            }))
            .unwrap();
            duel.apply(Action::Place(zone)).unwrap();
            pass_chains(&mut duel).unwrap();
        }
        duel.apply(kind_on(&duel, typhoon, |action| {
            matches!(action, Action::IdleCommand(IdleCommand::Activate(_)))
        }))
        .unwrap();
        loop {
            let legal = duel.legal_actions();
            let action = match duel.prompt() {
                Some(Prompt::Chain { .. }) => return duel.state_hash(),
                Some(Prompt::Place { .. }) => legal.first().copied(),
                // The target function waits at this prompt.
                _ => {
                    assert_eq!(duel.state_hash(), None);
                    let at_target = |action: &Action| {
                        duel.action_card(*action)
                            .0
                            .is_some_and(|card| card.get_sequence(&duel) == target - 8)
                    };
                    legal.into_iter().find(at_target)
                }
            };
            duel.apply(action.unwrap()).unwrap();
        }
    };
    let first = chain_window([(pot, 8), (keto, 9)], 8);
    assert!(first.is_some());
    assert_eq!(chain_window([(keto, 9), (pot, 8)], 8), first);
    assert_ne!(chain_window([(pot, 8), (keto, 9)], 9), first);
}

#[test]
fn should_leave_room_for_105_tokens_beside_the_largest_decks() {
    let mut duel = new_duel().unwrap();
    let deck = |main, extra| Deck {
        main: vec![1; main],
        extra: vec![1; extra],
    };
    for team in 0..2 {
        duel.add_deck(team, &deck(60, 15), &CARDS).unwrap();
    }
    let room = (0..).take_while(|_| duel.add_deck(0, &deck(1, 0), &CARDS).is_ok()).count();
    assert_eq!((room, duel.add_deck(0, &deck(1, 0), &CARDS)), (105, Err(DuelError::TooManyCards)));
}

// Every card gets EFFECT_CHANGE_BATTLE_STAT, which damage calculation does not apply yet.
fn unported_battle_stat(_: u32) -> Option<InitialEffect> {
    Some(|duel: &mut Duel, card: CardId| {
        let effect = duel.create_effect(card);
        effect.set_type(duel, EFFECT_TYPE_SINGLE.into());
        effect.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        effect.set_range(duel, LOCATION_MZONE.into());
        effect.set_code(duel, EFFECT_CHANGE_BATTLE_STAT);
        effect.set_value(duel, 500);
        card.register_effect(duel, effect);
    })
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "unported call site"))]
fn should_mark_the_duel_when_an_effect_reaches_an_unported_call_site() {
    static LIBRARY: Library = Library {
        load: |_| {},
        card_script: unported_battle_stat,
    };
    let mut duel = started_duel(&CODES.collect::<Vec<_>>(), &LIBRARY).unwrap();
    assert_eq!(duel.unported_site(), None);
    play(&mut duel, 0..DECISIONS).unwrap();
    assert!(duel.unported_site().is_some_and(|site| site.contains("processor.rs")));
}

// A global effect declared promptless whose operation asks a question on every draw.
fn load_prompting_promptless(duel: &mut Duel) {
    let effect = duel.global_effect();
    effect.set_type(duel, (EFFECT_TYPE_FIELD | EFFECT_TYPE_CONTINUOUS).into());
    effect.set_code(duel, EVENT_DRAW);
    effect.set_promptless_operation(duel, |duel, a| duel.select_yes_no(a.tp, 0).map(drop));
    duel.register_effect_core(effect, 0);
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "a promptless function prompted"))]
fn should_mark_the_duel_when_a_promptless_function_prompts() {
    static LIBRARY: Library = Library {
        load: load_prompting_promptless,
        card_script: |_| None,
    };
    let mut duel = started_duel(&CODES.collect::<Vec<_>>(), &LIBRARY).unwrap();
    play(&mut duel, 0..DECISIONS).unwrap();
    assert!(duel.unported_site().is_some_and(|site| site.contains("script.rs")));
}
