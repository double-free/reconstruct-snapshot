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

    let snapshot_2290 = md::Snapshot {
        ms: "09:25:45.090771",
        clock: 1587605145091829,
        threadId: 23994,
        clockAtArrival: 1587605145091648,
        sequenceNo: 1543447,
        source: 24,
        StockID: 2290,
        exchange: "SZ",
        time: "09:25:00.000",
        cum_volume: 0,
        cum_amount: 0.0,
        close: 0.0,
        __origTickSeq: 0,
        bid1p: 5.12,
        bid2p: 5.11,
        bid3p: 5.10,
        bid4p: 5.08,
        bid5p: 5.07,
        bid1q: 3000,
        bid2q: 1500,
        bid3q: 9800,
        bid4q: 15800,
        bid5q: 1000,
        ask1p: 5.22,
        ask2p: 5.23,
        ask3p: 5.30,
        ask4p: 5.35,
        ask5p: 5.38,
        ask1q: 1000,
        ask2q: 600,
        ask3q: 1001,
        ask4q: 1000,
        ask5q: 1200,
        openPrice: 0.0,
        numTrades: 0,
    };

    let snapshot_2385 = md::Snapshot {
        ms: "09:25:45.124771",
        clock: 1587605145125220,
        threadId: 23994,
        clockAtArrival: 1587605145124998,
        sequenceNo: 1548408,
        source: 24,
        StockID: 2385,
        exchange: "SZ",
        time: "09:25:00.000",
        cum_volume: 11641250,
        cum_amount: 111756000.0,
        close: 9.60,
        __origTickSeq: 0,
        bid1p: 9.60,
        bid2p: 9.59,
        bid3p: 9.58,
        bid4p: 9.57,
        bid5p: 9.56,
        bid1q: 552050,
        bid2q: 15500,
        bid3q: 96900,
        bid4q: 1700,
        bid5q: 8900,
        ask1p: 9.61,
        ask2p: 9.62,
        ask3p: 9.63,
        ask4p: 9.64,
        ask5p: 9.65,
        ask1q: 30300,
        ask2q: 110700,
        ask3q: 30733,
        ask4q: 41700,
        ask5q: 285300,
        openPrice: 9.6,
        numTrades: 3493,
    };

    // builder.init(&vec![snapshot_2290, snapshot_2385]);

    // 1587605145124998: 09:25:00
    // 1587605445122501: 09:30:00
    // 1587605991164248: 09:39:06
    let timestamps = vec![1587605991164248];
    let snapshots = builder.build_snapshot(&timestamps);
    for snapshot in snapshots {
        println!("{:?}", snapshot);
    }
}
