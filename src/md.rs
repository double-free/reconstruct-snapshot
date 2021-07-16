use std::error::Error;

enum Side {
    Buy,
    Sell,
    Unknown,
}

impl Side {
    pub fn from_string(s: &str) -> Side {
        match s {
            "1" => Side::Buy,
            "2" => Side::Sell,
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
    clockAtArrival: i64,
    sequenceNo: i64,
    exchId: i8,
    securityType: i8,
    __isRepeated: i8,
    TransactTime: i64,
    ChannelNo: i32,
    ApplSeqNum: i64,
    SecurityID: i32,
    secid: i32,
    mdSource: i8,
    Side: Side,
    OrderType: OrderType,
    __origTickSeq: i8,
    Price: i64,
    OrderQty: i64,
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

enum ExecuteType {
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
    clockAtArrival: i64,
    sequenceNo: i64,
    exchId: i8,
    securityType: i8,
    __isRepeated: i8,
    TransactTime: i64,
    ChannelNo: i32,
    ApplSeqNum: i64,
    SecurityID: i32,
    secid: i32,
    mdSource: i8,
    ExecType: ExecuteType,
    TradeBSFlag: char,
    __origTickSeq: i8,
    TradePrice: i64,
    TradeQty: i64,
    TradeMoney: i64,
    BidApplSeqNum: i64,
    OfferApplSeqNum: i64,
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

pub fn read_csv<T: Convertable>(filename: &str) -> Result<Vec<T>, Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_path(filename)?;
    let mut result = Vec::new();

    let mut records = rdr.records().into_iter();
    if let Some(Ok(header)) = records.next() {
        for maybe_row in records {
            let row = maybe_row?;
            result.push(T::from_string_record(&row));
        }
        return Ok(result);
    }

    return Err(Box::<dyn Error>::from(format!(
        "can not read from {}",
        filename
    )));
}
