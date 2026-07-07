use gmo_coin_fx_core::models::ws_events::ExecutionEvent;
use gmo_coin_fx_domain_risk::types::RiskMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub pnl: f64,
    pub timestamp: String,
    pub effective_leverage: f64,
    pub required_margin: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeJournal {
    pub records: Vec<TradeRecord>,
}

impl TradeJournal {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn append(
        &mut self,
        event: &ExecutionEvent,
        metrics: &RiskMetrics,
    ) -> Result<(), gmo_coin_fx_core::error::GmoFxError> {
        let price = event.execution_price.parse::<f64>().map_err(|e| {
            gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                "failed to parse executionPrice: {}",
                e
            ))
        })?;
        let size = event.execution_size.parse::<f64>().map_err(|e| {
            gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                "failed to parse executionSize: {}",
                e
            ))
        })?;
        let pnl = event.loss_gain.parse::<f64>().map_err(|e| {
            gmo_coin_fx_core::error::GmoFxError::InvalidRequest(format!(
                "failed to parse lossGain: {}",
                e
            ))
        })?;

        self.records.push(TradeRecord {
            trade_id: event.execution_id.to_string(),
            order_id: event.order_id.to_string(),
            symbol: event.symbol.clone(),
            side: event.side.clone(),
            price,
            size,
            pnl,
            timestamp: event.execution_timestamp.clone(),
            effective_leverage: metrics.effective_leverage,
            required_margin: metrics.required_margin,
        });

        Ok(())
    }

    pub fn to_csv(&self) -> Result<String, String> {
        let mut csv = String::new();
        csv.push_str("trade_id,order_id,symbol,side,price,size,pnl,timestamp,effective_leverage,required_margin\n");
        for r in &self.records {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                r.trade_id,
                r.order_id,
                r.symbol,
                r.side,
                r.price,
                r.size,
                r.pnl,
                r.timestamp,
                r.effective_leverage,
                r.required_margin
            ));
        }
        Ok(csv)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gmo_coin_fx_core::models::ws_events::MsgType;
    use gmo_coin_fx_core::models::SettleType;

    fn dummy_event() -> ExecutionEvent {
        ExecutionEvent {
            amount: "10000".to_string(),
            root_order_id: 123,
            order_id: 456,
            client_order_id: Some("client-123".to_string()),
            execution_id: 789,
            symbol: "USD_JPY".to_string(),
            settle_type: SettleType::Open,
            order_type: "LIMIT".to_string(),
            execution_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            execution_price: "150.25".to_string(),
            execution_size: "10000.0".to_string(),
            position_id: 999,
            loss_gain: "1500.0".to_string(),
            settled_swap: None,
            fee: "0.0".to_string(),
            order_price: "150.25".to_string(),
            order_executed_size: "10000.0".to_string(),
            order_size: "10000.0".to_string(),
            msg_type: MsgType::ExecutionReport,
            order_timestamp: "2026-07-07T12:00:00Z".to_string(),
            execution_timestamp: "2026-07-07T12:00:05Z".to_string(),
        }
    }

    #[test]
    fn test_journal_append_success() {
        let mut journal = TradeJournal::new();
        let event = dummy_event();
        let metrics = RiskMetrics {
            notional_value: 1502500.0,
            required_margin: 60100.0,
            effective_leverage: 25.0,
            margin_rate: 400.0,
            loss_per_1yen: 10000.0,
        };

        let result = journal.append(&event, &metrics);
        assert!(result.is_ok());
        assert_eq!(journal.records.len(), 1);

        let record = &journal.records[0];
        assert_eq!(record.trade_id, "789");
        assert_eq!(record.order_id, "456");
        assert_eq!(record.symbol, "USD_JPY");
        assert_eq!(record.side, "BUY");
        assert_eq!(record.price, 150.25);
        assert_eq!(record.size, 10000.0);
        assert_eq!(record.pnl, 1500.0);
        assert_eq!(record.timestamp, "2026-07-07T12:00:05Z");
        assert_eq!(record.effective_leverage, 25.0);
        assert_eq!(record.required_margin, 60100.0);
    }

    #[test]
    fn test_journal_append_invalid_price() {
        let mut journal = TradeJournal::new();
        let mut event = dummy_event();
        event.execution_price = "invalid".to_string();
        let metrics = RiskMetrics {
            notional_value: 1502500.0,
            required_margin: 60100.0,
            effective_leverage: 25.0,
            margin_rate: 400.0,
            loss_per_1yen: 10000.0,
        };

        let result = journal.append(&event, &metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_journal_export_json_and_csv() {
        let mut journal = TradeJournal::new();
        let event1 = dummy_event();
        let mut event2 = dummy_event();
        event2.execution_id = 790;
        event2.execution_price = "150.30".to_string();
        event2.loss_gain = "-500.0".to_string();

        let metrics = RiskMetrics {
            notional_value: 1502500.0,
            required_margin: 60100.0,
            effective_leverage: 25.0,
            margin_rate: 400.0,
            loss_per_1yen: 10000.0,
        };

        journal.append(&event1, &metrics).unwrap();
        journal.append(&event2, &metrics).unwrap();

        // Test JSON Export
        let json_str = journal.to_json().unwrap();
        let decoded: Vec<TradeRecord> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].price, 150.25);
        assert_eq!(decoded[1].price, 150.30);
        assert_eq!(decoded[1].pnl, -500.0);

        // Test CSV Export
        let csv_str = journal.to_csv().unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();
        assert_eq!(lines.len(), 3); // Header + 2 rows
        assert_eq!(lines[0], "trade_id,order_id,symbol,side,price,size,pnl,timestamp,effective_leverage,required_margin");
        assert!(lines[1].starts_with("789,456,USD_JPY,BUY,"));
        assert!(lines[2].starts_with("790,456,USD_JPY,BUY,"));
    }
}
