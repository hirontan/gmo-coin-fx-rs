/// リスク計算や注文前チェックで利用する設定値
#[derive(Debug, Clone, Copy)]
pub struct RiskConfig {
    /// 許容する最大実効レバレッジ
    pub max_effective_leverage: f64,
    /// 許容する最小証拠金維持率（%）
    pub min_margin_rate: f64,
    /// 1トレードあたりの許容リスク（有効証拠金に対する割合）
    pub risk_per_trade_pct: f64,
    /// 取引単位（例: 1000通貨）
    pub quantity_unit: f64,
    /// 許容する最大ポジション数
    pub max_open_positions: Option<usize>,
}

/// リスク計算の結果をまとめた指標
#[derive(Debug, Clone, Copy)]
pub struct RiskMetrics {
    /// ポジション総額
    pub notional_value: f64,
    /// 必要証拠金
    pub required_margin: f64,
    /// 実効レバレッジ
    pub effective_leverage: f64,
    /// 証拠金維持率（%）
    pub margin_rate: f64,
    /// 1円変動あたりの損益額
    /// （USD/JPYのようなクロス円通貨ペアで有用）
    pub loss_per_1yen: f64,
}

/// 注文前リスクチェックの結果
#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    /// 注文が許可されるかどうか
    pub allowed: bool,
    /// 却下された理由（許可された場合は空）
    pub reasons: Vec<String>,
    /// 計算されたリスク指標
    pub metrics: RiskMetrics,
}
