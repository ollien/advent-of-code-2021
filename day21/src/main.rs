#![warn(clippy::all, clippy::pedantic)]
use std::{cmp, collections::HashMap, env, fs, iter};

use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{eof, map_res},
    multi::many0,
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone)]
struct DeterministicDie {
    next_value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    Player1,
    Player2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PlayerState {
    score: u32,
    position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GameState {
    player1_state: PlayerState,
    player2_state: PlayerState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MemoState {
    game_state: GameState,
    current_player: Player,
    pending_rolls: Vec<u32>,
}

impl Player {
    fn get_other(self) -> Player {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

impl PlayerState {
    fn new(starting_pos: u32) -> Self {
        Self {
            score: 0,
            position: starting_pos,
        }
    }

    fn move_forward(&mut self, n: u32) {
        self.position = (self.position + n - 1) % 10 + 1;
        self.score += self.position;
    }
}

impl GameState {
    fn get_player_state(&self, player: Player) -> &PlayerState {
        match player {
            Player::Player1 => &self.player1_state,
            Player::Player2 => &self.player2_state,
        }
    }

    fn get_player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        match player {
            Player::Player1 => &mut self.player1_state,
            Player::Player2 => &mut self.player2_state,
        }
    }

    fn get_winning_player(&self) -> Player {
        if self.player1_state.score > self.player2_state.score {
            Player::Player1
        } else {
            Player::Player2
        }
    }

    fn get_losing_player(&self) -> Player {
        self.get_winning_player().get_other()
    }
}

impl Default for DeterministicDie {
    fn default() -> Self {
        Self { next_value: 1 }
    }
}

impl DeterministicDie {
    fn new() -> Self {
        Self::default()
    }

    fn roll(&mut self) -> u32 {
        let val = self.next_value;
        self.next_value = self.next_value % 100 + 1;

        val
    }
}

fn parse_player_starting_position(chunk: &str) -> IResult<&str, u32> {
    preceded(
        tuple((tag("Player "), digit1, tag(" starting position: "))),
        map_res(digit1, str::parse),
    )(chunk)
}

fn parse_input(input: &str) -> IResult<&str, (u32, u32)> {
    terminated(
        separated_pair(
            parse_player_starting_position,
            char('\n'),
            parse_player_starting_position,
        ),
        pair(many0(char('\n')), eof),
    )(input)
}

fn part1(player1_start_pos: u32, player2_start_pos: u32) -> u32 {
    let mut die = DeterministicDie::new();
    let mut num_rolls = 0;
    let mut current_player = Player::Player1;

    let mut game_state = GameState {
        player1_state: PlayerState::new(player1_start_pos),
        player2_state: PlayerState::new(player2_start_pos),
    };

    while game_state
        .get_player_state(game_state.get_winning_player())
        .score
        < 1000
    {
        let roll_value = iter::once(0).cycle().take(3).map(|_| die.roll()).sum();
        num_rolls += 3;

        let current_player_state = game_state.get_player_state_mut(current_player);
        current_player_state.move_forward(roll_value);

        current_player = current_player.get_other();
    }

    let losing_player = game_state.get_losing_player();
    game_state.get_player_state(losing_player).score * num_rolls
}

// Find the number of games that each player wins
fn play_part2_game(
    initial_game_state: GameState,
    mut current_player: Player,
    initial_pending_rolls: Vec<u32>,
    winner_memo: &mut HashMap<MemoState, (u64, u64)>,
) -> (u64, u64) {
    let memo_state = MemoState {
        game_state: initial_game_state,
        current_player,
        pending_rolls: initial_pending_rolls,
    };

    if let Some(&outcome) = winner_memo.get(&memo_state) {
        // Given the same argument set, we can skip the computation if we've already done it
        return outcome;
    }

    // Deconstruct the memo state to move out the originals, so as to satisfy the borrow checker.
    let mut game_state = memo_state.game_state;
    let mut pending_rolls = memo_state.pending_rolls;

    // If there are 3 rolls, move the current player (which also updates the score) and end their turn
    if pending_rolls.len() == 3 {
        let total_pending = pending_rolls.iter().sum::<u32>();
        game_state
            .get_player_state_mut(current_player)
            .move_forward(total_pending);
        current_player = current_player.get_other();
        pending_rolls.clear();
    }

    let currently_winning_player = game_state.get_winning_player();
    // If the player who is currently "winning" has a score >= 21, they have accured a win
    if game_state.get_player_state(currently_winning_player).score >= 21 {
        return match currently_winning_player {
            Player::Player1 => (1, 0),
            Player::Player2 => (0, 1),
        };
    }

    let mut player1_wins = 0;
    let mut player2_wins = 0;
    // Produce the 3 rolls we need for part 2
    for roll in 1..=3 {
        let next_pending_rolls = iter::once(roll)
            .chain(pending_rolls.iter().copied())
            .collect::<Vec<_>>();

        let memo_state = MemoState {
            game_state: game_state.clone(),
            current_player,
            pending_rolls: next_pending_rolls.clone(),
        };

        // Start a new universe with these rolls
        let (resulting_player1_wins, resulting_player2_wins) = play_part2_game(
            game_state.clone(),
            current_player,
            next_pending_rolls,
            winner_memo,
        );

        winner_memo
            .entry(memo_state)
            .or_insert((resulting_player1_wins, resulting_player2_wins));

        player1_wins += resulting_player1_wins;
        player2_wins += resulting_player2_wins;
    }

    (player1_wins, player2_wins)
}

fn part2(player1_start_pos: u32, player2_start_pos: u32) -> u64 {
    let (player1_wins, player2_wins) = play_part2_game(
        GameState {
            player1_state: PlayerState::new(player1_start_pos),
            player2_state: PlayerState::new(player2_start_pos),
        },
        Player::Player1,
        Vec::new(),
        &mut HashMap::new(),
    );

    cmp::max(player1_wins, player2_wins)
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let (_, (player1_start_pos, player2_start_pos)) =
        parse_input(&input).expect("Failed to parse input");

    println!("Part 1: {}", part1(player1_start_pos, player2_start_pos));
    println!("Part 2: {}", part2(player1_start_pos, player2_start_pos));
}
