use rand::Rng;

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

fn count_coins(coins: &Vec<Coin>) -> u32 {
    let mut total = 0;
    for ele in coins {
        total += *ele as u32;
    }
    total
}

fn main() {
    let mut coins: Vec<Coin> = Vec::new();
    for _ in 0..1000000 {
        let chosen_coin = rand::thread_rng().gen_range(0..(ALL_COINS.len()));
        let chosen_coin = ALL_COINS[chosen_coin];
        coins.push(chosen_coin);
    }
    
    let value = count_coins(&coins);

    let euros = (value as f32 / 100.0) as u32;
    let cents = value - euros * 100;

    println!("Counted {} coins with a total value of {} euros and {} cents", coins.len(), euros, cents);
}
