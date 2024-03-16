use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rayon::prelude::*;

// coin sorting
#[derive(Copy, Clone)]
enum Coin {
    OneCent = 1,
    TwoCent = 2,
    FiveCent = 5,
    TenCent = 10,
    TwentyCent = 20,
    FiftyCent = 50,
    OneEuro = 100,
    TwoEuro = 200
}

const ALL_COINS: [Coin; 8] = [ Coin::OneCent, Coin::TwoCent, Coin::FiveCent, Coin::TenCent, Coin::TwentyCent, Coin::FiftyCent, Coin::OneEuro, Coin::TwoEuro ];

fn count_coins(coins: &[Coin]) -> u32 {
    coins.iter().map(|&c| c as u32).sum()
}

fn main() {
    let coins: Vec<Coin> = (0..1_000_000)
        .into_par_iter()
        .map(|_| {
             let chosen_coin = SmallRng::from_entropy().gen_range(0..ALL_COINS.len());
             ALL_COINS[chosen_coin]
             })
        .collect();
    
    let value = count_coins(&coins);

    let euros = (value as f32 / 100.0) as u32;
    let cents = value - euros * 100;

    println!("Counted {} coins with a total value of {} euros and {} cents", coins.len(), euros, cents);
}
