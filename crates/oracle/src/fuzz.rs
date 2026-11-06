use std::time::{Duration, Instant};

use carddata::{CardDatabase, Deck};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::duel::{Duel, DuelError, Status};
use crate::message::{Message, MessageError, Prompt, parse};
use crate::random::respond;
use crate::scripts::Scripts;

#[derive(Debug, thiserror::Error)]
pub enum FuzzError {
    #[error(transparent)]
    Duel(#[from] DuelError),
    #[error(transparent)]
    Message(#[from] MessageError),
    #[error("the core awaits a response without sending a prompt")]
    MissingPrompt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Win(u8),
    // The core stopped without MSG_WIN.
    Ended,
    DecisionCap,
    Retry { prompt: Prompt, response: Vec<u8> },
    Unanswerable(Prompt),
}

#[derive(Debug)]
pub struct Report {
    pub outcome: Outcome,
    pub decisions: u32,
    pub turns: u32,
    pub setup: Duration,
    pub play: Duration,
    pub errors: Vec<String>,
    pub responses: Vec<Vec<u8>>,
}

// One seed fixes both decks (`deck` draws each from the seed's RNG), the duel seed and every response.
pub fn play_random_duel(
    cards: &CardDatabase,
    scripts: &Scripts,
    deck: impl Fn(&mut StdRng) -> Deck,
    seed: u64,
    max_decisions: u32,
    mut observe: impl FnMut(&[u8]),
) -> Result<Report, FuzzError> {
    let started = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut duel = Duel::new(cards, scripts, rng.random())?;
    for team in 0..2 {
        duel.add_deck(team, &deck(&mut rng));
    }
    duel.start();
    let setup = started.elapsed();
    let (mut decisions, mut turns, mut responses) = (0, 0, Vec::new());
    let mut last: Option<(Prompt, Vec<u8>)> = None;
    let outcome = loop {
        let status = duel.process()?;
        let (mut prompt, mut end) = (None, None);
        for message in duel.messages() {
            observe(message);
            match parse(message)? {
                Message::Win { player, .. } => end = Some(Outcome::Win(player)),
                Message::Retry => end = last.take().map(|(prompt, response)| Outcome::Retry { prompt, response }),
                Message::NewTurn { .. } => turns += 1,
                Message::Prompt(next) => prompt = Some(next),
                Message::Other(_) => {}
            }
        }
        if let Some(end) = end {
            break end;
        }
        match status {
            Status::End => break Outcome::Ended,
            Status::Continue => continue,
            Status::Awaiting if decisions == max_decisions => break Outcome::DecisionCap,
            Status::Awaiting => {}
        }
        let prompt = prompt.ok_or(FuzzError::MissingPrompt)?;
        let Some(response) = respond(&prompt, cards, &mut rng) else {
            break Outcome::Unanswerable(prompt);
        };
        duel.set_response(&response);
        decisions += 1;
        responses.push(response.clone());
        last = Some((prompt, response));
    };
    Ok(Report {
        outcome,
        decisions,
        turns,
        setup,
        play: started.elapsed() - setup,
        errors: duel.take_errors(),
        responses,
    })
}

// The card an error comes from, preferring the message over its traceback, else the first other script named.
pub fn culprit(error: &str) -> Option<&str> {
    let tokens = error.split(|char: char| !(char.is_ascii_alphanumeric() || char == '_' || char == '.'));
    tokens
        .clone()
        .find_map(card_script)
        .or_else(|| tokens.clone().find(|token| token.ends_with(".lua")))
}

// `c<id>` as in `[string "c123.lua"]:4:` or `(c123.initial_effect)`.
fn card_script(token: &str) -> Option<&str> {
    let name = token.split('.').next()?;
    name.strip_prefix('c')
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(name)
}
