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
