use crate::md;
use std::cmp;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
struct Level {
    pub price: i64,
    pub quantity: i64,
}

struct Book {
    inst_id: i32,
    pub timestamp: i64,
    // this does not include best orders
    bid_levels: VecDeque<Level>,
    bid_best_order_quantity: i64,
    ask_levels: VecDeque<Level>,
    ask_best_order_quantity: i64,

    // key: ApplSeqNum of order
    // value: order
    orders_: HashMap<i64, Rc<md::Order>>,

    // some accumulated statics
    pub cum_volume: i64,
    pub cum_amount: i64,
    pub num_trades: i64,
    pub close: i64,      // latest trade price
    pub open_price: i64, // first trade price
}

impl Book {
    pub const PRICE_DIVISOR: f64 = 10000.0;

    pub fn new(inst_id: i32) -> Book {
        Book {
            inst_id: inst_id,
            timestamp: 0,
            bid_levels: VecDeque::new(),
            bid_best_order_quantity: 0,
            ask_levels: VecDeque::new(),
            ask_best_order_quantity: 0,
            orders_: HashMap::new(),
            cum_volume: 0,
            cum_amount: 0,
            num_trades: 0,
            close: 0,
            open_price: 0,
        }
    }

    pub fn apply_change(&mut self, side: md::Side, price: i64, quantity: i64) {
        let (levels, aggressive_ordering) = match side {
            md::Side::Bid => (&mut self.bid_levels, cmp::Ordering::Greater),
            md::Side::Ask => (&mut self.ask_levels, cmp::Ordering::Less),
            md::Side::Unknown => {
                println!("Unknown side is impossible, skip");
                return;
            }
        };

        // find the first level that is less aggressive than given price
        let mut idx = 0;
        for level in levels.iter() {
            if level.price.cmp(&price) != aggressive_ordering {
                break;
            }
            idx += 1;
        }

        if levels.len() == idx || levels[idx].price != price {
            // it's a new level
            let level = Level {
                price: price,
                quantity: quantity,
            };
            println!(
                "At timestamp {}, insert {:?} at {} for {:?} side of instrument {}",
                self.timestamp, &level, idx, side, self.inst_id
            );
            levels.insert(idx, level);
            return;
        }

        let prev_level = levels[idx].clone();
        // level exists, update it
        levels[idx].quantity += quantity;
        println!(
            "At timestamp {}, update level {} from {:?} to {:?} for {:?} side of instrument {}",
            self.timestamp, idx, prev_level, &levels[idx], side, self.inst_id
        );
        if levels[idx].quantity <= 0 {
            levels.remove(idx);
        }
    }

    pub fn handle_order(&mut self, order: &Rc<md::Order>) {
        if self.timestamp > order.clockAtArrival {
            // it's possible that multiple message comes in 1 packet, do not use >=
            return;
        }

        self.timestamp = order.clockAtArrival;
        self.orders_.insert(order.ApplSeqNum, Rc::clone(order));

        match order.OrderType {
            md::OrderType::LimitOrder | md::OrderType::MarketOrder => {
                self.apply_change(order.Side, order.Price, order.OrderQty);
            }
            md::OrderType::BestOrder => {
                match order.Side {
                    md::Side::Bid => self.bid_best_order_quantity += order.OrderQty,
                    md::Side::Ask => self.ask_best_order_quantity += order.OrderQty,
                    md::Side::Unknown => {}
                }
                println!(
                    "snapshot when receive best order at {}: {:?}",
                    order.clockAtArrival,
                    self.to_snapshot()
                );
            }
            md::OrderType::Unknown => {}
        }

        // last order timestamp before 09:25
        if self.timestamp >= 1587605144280888 && self.crossed() {
            // book crossed, trigger trade
            self.handle_cross();
        }
    }

    fn crossed(&self) -> bool {
        if self.ask_levels.is_empty() || self.bid_levels.is_empty() {
            return false;
        }

        return self.ask_levels[0].price <= self.bid_levels[0].price;
    }

    fn handle_cross(&mut self) {
        while self.crossed() {
            println!(
                "handle simulated trade for {}, top bid = {:?}, top ask = {:?}",
                &self.inst_id, &self.bid_levels[0], &self.ask_levels[0]
            );

            let cross_quantity = cmp::min(self.bid_levels[0].quantity, self.ask_levels[0].quantity);
            // note that the price is not trade price
            // it only means to remove from level 0
            self.apply_change(md::Side::Bid, self.bid_levels[0].price, -cross_quantity);
            self.apply_change(md::Side::Ask, self.ask_levels[0].price, -cross_quantity);
        }
    }

    pub fn handle_trade(&mut self, trade: &md::Trade) {
        if self.timestamp > trade.clockAtArrival {
            // it's possible that multiple message comes in 1 packet, do not use >=
            return;
        }
        self.timestamp = trade.clockAtArrival;

        match trade.ExecType {
            md::ExecuteType::Traded => {
                self.num_trades += 1;
                self.cum_volume += trade.TradeQty;
                self.cum_amount += trade.TradeQty * trade.TradePrice;
                self.close = trade.TradePrice;
                if self.open_price == 0 {
                    // only update once in a day
                    self.open_price = trade.TradePrice;
                }

                // the trade is simulated in cross event
                // we don't need to change the order book again
                // a trade shall change both side
                // self.apply_change(
                //     md::Side::Bid,
                //     trade.TradePrice,
                //     -trade.TradeQty,
                // );
                // self.apply_change(
                //     md::Side::Ask,
                //     trade.TradePrice,
                //     -trade.TradeQty,
                // );
            }
            md::ExecuteType::Cancelled => {
                // we can only query the price
                let order = if trade.BidApplSeqNum != 0 {
                    &self.orders_[&trade.BidApplSeqNum]
                } else {
                    &self.orders_[&trade.OfferApplSeqNum]
                };

                match order.OrderType {
                    md::OrderType::MarketOrder | md::OrderType::LimitOrder => {
                        self.apply_change(order.Side, order.Price, -trade.TradeQty);
                    }
                    md::OrderType::BestOrder => match order.Side {
                        md::Side::Bid => self.bid_best_order_quantity -= order.OrderQty,
                        md::Side::Ask => self.ask_best_order_quantity -= order.OrderQty,
                        md::Side::Unknown => {}
                    },
                    md::OrderType::Unknown => {}
                }
            }

            md::ExecuteType::Unknown => {}
        }

        // Debug
        if self.num_trades == 50866 {
            println!(
                "snapshot when 2385 has {} trade: {:?}",
                self.num_trades,
                self.to_snapshot()
            );
        }
    }

    pub fn to_snapshot(&self) -> md::Snapshot {
        md::Snapshot {
            ms: "08:24:47.847788",
            clock: self.timestamp,
            threadId: 23994,
            clockAtArrival: self.timestamp,
            sequenceNo: -1,
            source: 24,
            StockID: self.inst_id,
            exchange: "SZ",
            time: "08:24:03.000",
            cum_volume: self.cum_volume,
            cum_amount: self.cum_amount as f64 / Book::PRICE_DIVISOR,
            close: self.close as f64 / Book::PRICE_DIVISOR,
            __origTickSeq: -1,
            bid1p: self.bid_levels[0].price as f64 / Book::PRICE_DIVISOR,
            bid2p: self.bid_levels[1].price as f64 / Book::PRICE_DIVISOR,
            bid3p: self.bid_levels[2].price as f64 / Book::PRICE_DIVISOR,
            bid4p: self.bid_levels[3].price as f64 / Book::PRICE_DIVISOR,
            bid5p: self.bid_levels[4].price as f64 / Book::PRICE_DIVISOR,
            bid1q: self.bid_levels[0].quantity,
            bid2q: self.bid_levels[1].quantity,
            bid3q: self.bid_levels[2].quantity,
            bid4q: self.bid_levels[3].quantity,
            bid5q: self.bid_levels[4].quantity,
            ask1p: self.ask_levels[0].price as f64 / Book::PRICE_DIVISOR,
            ask2p: self.ask_levels[1].price as f64 / Book::PRICE_DIVISOR,
            ask3p: self.ask_levels[2].price as f64 / Book::PRICE_DIVISOR,
            ask4p: self.ask_levels[3].price as f64 / Book::PRICE_DIVISOR,
            ask5p: self.ask_levels[4].price as f64 / Book::PRICE_DIVISOR,
            ask1q: self.ask_levels[0].quantity,
            ask2q: self.ask_levels[1].quantity,
            ask3q: self.ask_levels[2].quantity,
            ask4q: self.ask_levels[3].quantity,
            ask5q: self.ask_levels[4].quantity,
            openPrice: self.open_price as f64 / Book::PRICE_DIVISOR,
            numTrades: self.num_trades,
        }
    }
}

pub struct SnapshotBuilder {
    orders_: Vec<Rc<md::Order>>,
    trades_: Vec<Rc<md::Trade>>,

    // key: stock id
    books_: HashMap<i32, Book>,

    // current status
    order_idx_: usize,
    trade_idx_: usize,
}

impl SnapshotBuilder {
    pub fn new(orders: Vec<Rc<md::Order>>, trades: Vec<Rc<md::Trade>>) -> SnapshotBuilder {
        SnapshotBuilder {
            orders_: orders,
            trades_: trades,
            books_: HashMap::new(),

            order_idx_: 0,
            trade_idx_: 0,
        }
    }

    fn process_order(&mut self) {
        let order = &self.orders_[self.order_idx_];

        if !self.books_.contains_key(&order.SecurityID) {
            self.books_
                .insert(order.SecurityID, Book::new(order.SecurityID));
        }

        let book = self.books_.get_mut(&order.SecurityID).unwrap();
        book.handle_order(order);

        self.order_idx_ += 1;
    }

    fn process_trade(&mut self) {
        let trade = &self.trades_[self.trade_idx_];
        // no trade can happen without a book
        // it's a safe assumption that process order shall already create a book
        let book = self.books_.get_mut(&trade.SecurityID).unwrap();
        book.handle_trade(trade);

        self.trade_idx_ += 1;
    }

    // process event until timestamp
    pub fn process_until(&mut self, timestamp: i64) {
        while (self.order_idx_ < self.orders_.len()
            && self.orders_[self.order_idx_].clockAtArrival < timestamp)
            && (self.trade_idx_ < self.trades_.len()
                && self.trades_[self.trade_idx_].clockAtArrival < timestamp)
        {
            let order = &self.orders_[self.order_idx_];
            let trade = &self.trades_[self.trade_idx_];

            if order.clockAtArrival <= trade.clockAtArrival {
                // can not be '<' because we need always process orders first
                self.process_order();
            } else {
                self.process_trade()
            }
        }
        // no more orders
        while self.trade_idx_ < self.trades_.len()
            && self.trades_[self.trade_idx_].clockAtArrival < timestamp
        {
            self.process_trade();
        }

        // no more trades
        while self.order_idx_ < self.orders_.len()
            && self.orders_[self.order_idx_].clockAtArrival < timestamp
        {
            self.process_order();
        }
    }

    // start from these snapshots
    pub fn init(&mut self, snapshots: &Vec<md::Snapshot>) {
        for snapshot in snapshots {
            self.books_
                .insert(snapshot.StockID, Book::new(snapshot.StockID));
            let book = &mut self.books_.get_mut(&snapshot.StockID).unwrap();
            book.timestamp = snapshot.clockAtArrival;
            book.cum_volume = snapshot.cum_volume;
            book.cum_amount = (snapshot.cum_amount * Book::PRICE_DIVISOR) as i64;
            book.num_trades = snapshot.numTrades;
            book.close = (snapshot.close * Book::PRICE_DIVISOR) as i64;
            book.open_price = (snapshot.openPrice * Book::PRICE_DIVISOR) as i64;
            // for bids
            book.apply_change(
                md::Side::Bid,
                (snapshot.bid1p * Book::PRICE_DIVISOR) as i64,
                snapshot.bid1q,
            );
            book.apply_change(
                md::Side::Bid,
                (snapshot.bid2p * Book::PRICE_DIVISOR) as i64,
                snapshot.bid2q,
            );
            book.apply_change(
                md::Side::Bid,
                (snapshot.bid3p * Book::PRICE_DIVISOR) as i64,
                snapshot.bid3q,
            );
            book.apply_change(
                md::Side::Bid,
                (snapshot.bid4p * Book::PRICE_DIVISOR) as i64,
                snapshot.bid4q,
            );
            book.apply_change(
                md::Side::Bid,
                (snapshot.bid5p * Book::PRICE_DIVISOR) as i64,
                snapshot.bid5q,
            );
            // for asks
            book.apply_change(
                md::Side::Ask,
                (snapshot.ask1p * Book::PRICE_DIVISOR) as i64,
                snapshot.ask1q,
            );
            book.apply_change(
                md::Side::Ask,
                (snapshot.ask2p * Book::PRICE_DIVISOR) as i64,
                snapshot.ask2q,
            );
            book.apply_change(
                md::Side::Ask,
                (snapshot.ask3p * Book::PRICE_DIVISOR) as i64,
                snapshot.ask3q,
            );
            book.apply_change(
                md::Side::Ask,
                (snapshot.ask4p * Book::PRICE_DIVISOR) as i64,
                snapshot.ask4q,
            );
            book.apply_change(
                md::Side::Ask,
                (snapshot.ask5p * Book::PRICE_DIVISOR) as i64,
                snapshot.ask5q,
            );
        }
    }

    pub fn build_snapshot(&mut self, timestamps: &Vec<i64>) -> Vec<md::Snapshot> {
        let mut snapshots = Vec::with_capacity(timestamps.len());
        for ts in timestamps {
            self.process_until(*ts);

            // turn book into snapshot

            for (_, book) in self.books_.iter() {
                snapshots.push(book.to_snapshot());
            }
        }
        return snapshots;
    }

    pub fn reset(&mut self) {
        self.order_idx_ = 0;
        self.trade_idx_ = 0;
    }
}
