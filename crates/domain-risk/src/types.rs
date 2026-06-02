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

fn format_jpy(val: f64) -> String {
    let rounded = val.round() as i64;
    let sign = if rounded < 0 { "-" } else { "" };
    let abs_val = rounded.abs();
    let s = abs_val.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    format!("{}¥{}", sign, result)
}

impl std::fmt::Display for RiskMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_notional = format_jpy(self.notional_value);
        let formatted_margin = format_jpy(self.required_margin);
        let formatted_loss = format_jpy(self.loss_per_1yen);

        write!(
            f,
            "Position Value: {} | Required Margin: {}\n\
             Effective Leverage: {:.1}x | Margin Rate: {:.1}%\n\
             Loss per 1¥: {}",
            formatted_notional,
            formatted_margin,
            self.effective_leverage,
            self.margin_rate,
            formatted_loss
        )
    }
}

impl std::fmt::Display for RiskCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.allowed {
            write!(f, "Status: Allowed\n{}", self.metrics)
        } else {
            writeln!(f, "Status: Rejected")?;
            writeln!(f, "Reasons:")?;
            for reason in &self.reasons {
                writeln!(f, " - {}", reason)?;
            }
            write!(f, "{}", self.metrics)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_metrics_display() {
        let metrics = RiskMetrics {
            notional_value: 3151200.0,
            required_margin: 126048.0,
            effective_leverage: 10.504,
            margin_rate: 237.99,
            loss_per_1yen: 20000.0,
        };

        let display_str = metrics.to_string();
        let expected = "\
Position Value: ¥3,151,200 | Required Margin: ¥126,048\n\
Effective Leverage: 10.5x | Margin Rate: 238.0%\n\
Loss per 1¥: ¥20,000";
        assert_eq!(display_str, expected);
    }

    #[test]
    fn test_risk_check_result_display_allowed() {
        let metrics = RiskMetrics {
            notional_value: 787800.0,
            required_margin: 31512.0,
            effective_leverage: 2.626,
            margin_rate: 952.01,
            loss_per_1yen: 5000.0,
        };
        let result = RiskCheckResult {
            allowed: true,
            reasons: vec![],
            metrics,
        };

        let display_str = result.to_string();
        let expected = "\
Status: Allowed\n\
Position Value: ¥787,800 | Required Margin: ¥31,512\n\
Effective Leverage: 2.6x | Margin Rate: 952.0%\n\
Loss per 1¥: ¥5,000";
        assert_eq!(display_str, expected);
    }

    #[test]
    fn test_risk_check_result_display_rejected() {
        let metrics = RiskMetrics {
            notional_value: 3151200.0,
            required_margin: 126048.0,
            effective_leverage: 10.504,
            margin_rate: 237.99,
            loss_per_1yen: 20000.0,
        };
        let result = RiskCheckResult {
            allowed: false,
            reasons: vec![
                "Effective leverage exceeds limit: 10.5x > 5.0x".to_string(),
                "Margin maintenance rate is below threshold: 238% < 500%".to_string(),
            ],
            metrics,
        };

        let display_str = result.to_string();
        let expected = format!(
            "Status: Rejected\nReasons:\n - {}\n - {}\n{}",
            "Effective leverage exceeds limit: 10.5x > 5.0x",
            "Margin maintenance rate is below threshold: 238% < 500%",
            metrics
        );
        assert_eq!(display_str, expected);
    }
}
