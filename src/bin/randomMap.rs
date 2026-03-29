use clap::Parser;
use std::collections::HashSet;

/// A utility to map a space of source values to a space of target values, both starting from 1.
#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 48)]
    source_space: i64,
    #[arg(short, long, default_value_t = 250)]
    target_space: i64,
    draws: Vec<i64>,
}

fn main() {
    let args = Args::parse_from(wild::args());

    let number_of_draws = args.draws.len() as i64;

    let total_source_space = cmd_utils::perm(args.source_space, number_of_draws);

    let rejection_bound = total_source_space / args.target_space * args.target_space;

    if total_source_space < args.target_space {
        panic!("Not enough target space to map all source values");
    }

    let order_in_source_space = get_perm_order(&args.source_space, &args.draws);

    if order_in_source_space >= rejection_bound {
        panic!("Drawn values are out of bounds for mapping");
    }

    let final_mapped_value = order_in_source_space % args.target_space + 1;

    println!(
        "Drawn values {:?} mapped to <{}>",
        args.draws, final_mapped_value
    );
    println!(
        "(Source space: {}, Target space: {}, Order in source space: {}, Rejection bound: {})",
        args.source_space, args.target_space, order_in_source_space, rejection_bound
    );
}

fn get_perm_order(source_space: &i64, draws: &Vec<i64>) -> i64 {
    let number_of_draws = draws.len() as i64;
    let mut picked = HashSet::<i64>::new();
    return draws
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let res = (val - 1 - picked.iter().filter(|&&x| x < val).count() as i64)
                * cmd_utils::perm(source_space - i as i64 - 1, number_of_draws - i as i64 - 1);
            picked.insert(val);
            res
        })
        .sum::<i64>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_perm_order() {
        assert_eq!(get_perm_order(&5, &vec![2, 1, 3]), 12);
        assert_eq!(get_perm_order(&5, &vec![1, 2, 3]), 0);
        assert_eq!(get_perm_order(&5, &vec![3, 1, 2]), 24);
        assert_eq!(get_perm_order(&48, &vec![2, 1]), 47);
        assert_eq!(get_perm_order(&48, &vec![3, 1]), 94);
        assert_eq!(get_perm_order(&48, &vec![3, 47]), 139);
    }
}
