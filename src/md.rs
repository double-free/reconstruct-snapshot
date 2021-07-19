use std::error::Error;
use std::rc::Rc;

#[derive(Debug)]
pub enum Side {
    Bid,
    Ask,
    Unknown,
}

impl Side {
    pub fn from_string(s: &str) -> Side {
        match s {
            "1" => Side::Bid,
            "2" => Side::Ask,
            _ => Side::Unknown,
        }
    }
}

enum OrderType {
    MarketOrder,
    LimitOrder,
    BestOrder,
    Unknown,
}

impl OrderType {
    pub fn from_string(s: &str) -> OrderType {
        match s {
            "1" => OrderType::MarketOrder,
            "2" => OrderType::LimitOrder,
            "U" => OrderType::BestOrder,
            _ => OrderType::Unknown,
        }
    }
}

pub trait Convertable {
    fn from_string_record(sr: &csv::StringRecord) -> Self;
}

pub struct Order {
    pub clockAtArrival: i64,
    sequenceNo: i64,
    exchId: i8,
    securityType: i8,
    __isRepeated: i8,
    TransactTime: i64,
    ChannelNo: i32,
    pub ApplSeqNum: i64,
    pub SecurityID: i32,
    secid: i32,
    mdSource: i8,
    pub Side: Side,
    OrderType: OrderType,
    __origTickSeq: i8,
    pub Price: i64,
    pub OrderQty: i64,
}

impl Convertable for Order {
    fn from_string_record(row: &csv::StringRecord) -> Order {
        Order {
            clockAtArrival: row[0].parse::<i64>().unwrap(),
            sequenceNo: row[1].parse::<i64>().unwrap(),
            exchId: row[2].parse::<i8>().unwrap(),
            securityType: row[3].parse::<i8>().unwrap(),
            __isRepeated: row[4].parse::<i8>().unwrap(),
            TransactTime: row[5].parse::<i64>().unwrap(),
            ChannelNo: row[6].parse::<i32>().unwrap(),
            ApplSeqNum: row[7].parse::<i64>().unwrap(),
            SecurityID: row[8].parse::<i32>().unwrap(),
            secid: row[9].parse::<i32>().unwrap(),
            mdSource: row[10].parse::<i8>().unwrap(),
            Side: Side::from_string(&row[11]),
            OrderType: OrderType::from_string(&row[12]),
            __origTickSeq: row[13].parse::<i8>().unwrap(),
            Price: row[14].parse::<i64>().unwrap(),
            OrderQty: row[15].parse::<i64>().unwrap(),
        }
    }
}

pub enum ExecuteType {
    Cancelled,
    Traded,
    Unknown,
}

impl ExecuteType {
    pub fn from_string(s: &str) -> ExecuteType {
        match s {
            "4" => ExecuteType::Cancelled,
            "F" => ExecuteType::Traded,
            _ => ExecuteType::Unknown,
        }
    }
}

pub struct Trade {
    pub clockAtArrival: i64,
    sequenceNo: i64,
    exchId: i8,
    securityType: i8,
    __isRepeated: i8,
    TransactTime: i64,
    ChannelNo: i32,
    ApplSeqNum: i64,
    pub SecurityID: i32,
    secid: i32,
    mdSource: i8,
    pub ExecType: ExecuteType,
    TradeBSFlag: char,
    __origTickSeq: i8,
    pub TradePrice: i64,
    pub TradeQty: i64,
    TradeMoney: i64,
    pub BidApplSeqNum: i64,
    pub OfferApplSeqNum: i64,
}

impl Convertable for Trade {
    fn from_string_record(row: &csv::StringRecord) -> Trade {
        Trade {
            clockAtArrival: row[0].parse::<i64>().unwrap(),
            sequenceNo: row[1].parse::<i64>().unwrap(),
            exchId: row[2].parse::<i8>().unwrap(),
            securityType: row[3].parse::<i8>().unwrap(),
            __isRepeated: row[4].parse::<i8>().unwrap(),
            TransactTime: row[5].parse::<i64>().unwrap(),
            ChannelNo: row[6].parse::<i32>().unwrap(),
            ApplSeqNum: row[7].parse::<i64>().unwrap(),
            SecurityID: row[8].parse::<i32>().unwrap(),
            secid: row[9].parse::<i32>().unwrap(),
            mdSource: row[10].parse::<i8>().unwrap(),
            ExecType: ExecuteType::from_string(&row[11]),
            TradeBSFlag: 'N',
            __origTickSeq: row[13].parse::<i8>().unwrap(),
            TradePrice: row[14].parse::<i64>().unwrap(),
            TradeQty: row[15].parse::<i64>().unwrap(),
            TradeMoney: row[16].parse::<i64>().unwrap(),
            BidApplSeqNum: row[17].parse::<i64>().unwrap(),
            OfferApplSeqNum: row[18].parse::<i64>().unwrap(),
        }
    }
}

pub fn read_csv<T: Convertable>(filename: &str) -> Result<Vec<Rc<T>>, Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_path(filename)?;
    let mut result = Vec::new();

    let mut records = rdr.records().into_iter();
    if let Some(Ok(_header)) = records.next() {
        for maybe_row in records {
            let row = maybe_row?;
            result.push(Rc::new(T::from_string_record(&row)));
        }
        return Ok(result);
    }

    return Err(Box::<dyn Error>::from(format!(
        "can not read from {}",
        filename
    )));
}

#[derive(Debug)]
pub struct Snapshot {
    pub ms: &'static str,
    pub clock: i64,
    pub threadId: i32,
    pub clockAtArrival: i64,
    pub sequenceNo: i64,
    pub source: i8,
    pub StockID: i32,
    pub exchange: &'static str,
    pub time: &'static str,
    pub cum_volume: i64,
    pub cum_amount: f64,
    pub close: f64,
    pub __origTickSeq: i8,
    pub bid1p: f64,
    pub bid2p: f64,
    pub bid3p: f64,
    pub bid4p: f64,
    pub bid5p: f64,
    pub bid1q: i64,
    pub bid2q: i64,
    pub bid3q: i64,
    pub bid4q: i64,
    pub bid5q: i64,
    pub ask1p: f64,
    pub ask2p: f64,
    pub ask3p: f64,
    pub ask4p: f64,
    pub ask5p: f64,
    pub ask1q: i64,
    pub ask2q: i64,
    pub ask3q: i64,
    pub ask4q: i64,
    pub ask5q: i64,
    pub openPrice: f64,
    pub numTrades: i64,
}
