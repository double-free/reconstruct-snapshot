mod md;
mod snapshot_builder;
use std::env;

fn main() {
    let matches = clap::App::new(env::args().next().unwrap())
        .arg(
            clap::Arg::with_name("order")
                .short("o")
                .long("order")
                .help("csv file of orders")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("trade")
                .short("t")
                .long("trade")
                .help("csv file of trades")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let orders = md::read_csv::<md::Order>(matches.value_of("order").unwrap()).unwrap();
    let trades = md::read_csv::<md::Trade>(matches.value_of("trade").unwrap()).unwrap();

    let mut builder = snapshot_builder::SnapshotBuilder::new(orders, trades);

    let timestamps = vec![1587605991164248];
    let snapshots = builder.build_snapshot(&timestamps);
    for snapshot in snapshots {
        println!("{:?}", snapshot);
    }
}
