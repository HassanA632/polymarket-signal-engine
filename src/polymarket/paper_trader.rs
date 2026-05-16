#[derive(Debug, Clone, PartialEq)]
pub enum PaperPositionSide {
    Long,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaperExitReason {
    TakeProfit,
    StopLoss,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaperPosition {
    pub token_id: String,
    pub side: PaperPositionSide,
    pub entry_price: f64,
    pub stake: f64,
    pub entry_reason: String,
    pub is_open: bool,
}

impl PaperPosition {
    pub fn new_long(
        token_id: impl Into<String>,
        entry_price: f64,
        stake: f64,
        entry_reason: impl Into<String>,
    ) -> Self {
        Self {
            token_id: token_id.into(),
            side: PaperPositionSide::Long,
            entry_price,
            stake,
            entry_reason: entry_reason.into(),
            is_open: true,
        }
    }

    pub fn unrealised_pnl(&self, current_price: f64) -> f64 {
        match self.side {
            PaperPositionSide::Long => {
                let shares = self.stake / self.entry_price;
                (current_price - self.entry_price) * shares
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaperTrader {
    pub stake: f64,
    pub open_position: Option<PaperPosition>,
    pub closed_trades: Vec<ClosedPaperTrade>,
    pub realised_pnl: f64,
}

impl PaperTrader {
    pub fn new(stake: f64) -> Self {
        Self {
            stake,
            open_position: None,
            closed_trades: Vec::new(),
            realised_pnl: 0.0,
        }
    }

    pub fn has_open_position(&self) -> bool {
        self.open_position
            .as_ref()
            .is_some_and(|position| position.is_open)
    }

    pub fn open_long(
        &mut self,
        token_id: impl Into<String>,
        entry_price: f64,
        entry_reason: impl Into<String>,
    ) -> bool {
        if self.has_open_position() {
            return false;
        }

        self.open_position = Some(PaperPosition::new_long(
            token_id,
            entry_price,
            self.stake,
            entry_reason,
        ));

        true
    }

    pub fn maybe_close_position(
        &mut self,
        current_price: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Option<ClosedPaperTrade> {
        let position = self.open_position.as_ref()?;

        if !position.is_open {
            return None;
        }

        let pnl_pct = (current_price - position.entry_price) / position.entry_price;

        let exit_reason = if pnl_pct >= take_profit {
            PaperExitReason::TakeProfit
        } else if pnl_pct <= -stop_loss {
            PaperExitReason::StopLoss
        } else {
            return None;
        };

        let pnl = position.unrealised_pnl(current_price);

        let closed_trade = ClosedPaperTrade {
            token_id: position.token_id.clone(),
            side: position.side.clone(),
            entry_price: position.entry_price,
            exit_price: current_price,
            stake: position.stake,
            pnl,
            exit_reason,
        };

        self.realised_pnl += pnl;
        self.closed_trades.push(closed_trade.clone());
        self.open_position = None;

        Some(closed_trade)
    }

    pub fn closed_trade_count(&self) -> usize {
        self.closed_trades.len()
    }

    pub fn winning_trade_count(&self) -> usize {
        self.closed_trades
            .iter()
            .filter(|trade| trade.pnl > 0.0)
            .count()
    }

    pub fn losing_trade_count(&self) -> usize {
        self.closed_trades
            .iter()
            .filter(|trade| trade.pnl < 0.0)
            .count()
    }

    pub fn display_summary(&self) {
        println!(
            "PAPER_SUMMARY trades={} wins={} losses={} realised_pnl={:.4} open_position={}",
            self.closed_trade_count(),
            self.winning_trade_count(),
            self.losing_trade_count(),
            self.realised_pnl,
            self.has_open_position()
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosedPaperTrade {
    pub token_id: String,
    pub side: PaperPositionSide,
    pub entry_price: f64,
    pub exit_price: f64,
    pub stake: f64,
    pub pnl: f64,
    pub exit_reason: PaperExitReason,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_long_position() {
        let position = PaperPosition::new_long("token-1", 0.50, 10.0, "PriceMoveUp");

        assert_eq!(position.token_id, "token-1");
        assert_eq!(position.side, PaperPositionSide::Long);
        assert_eq!(position.entry_price, 0.50);
        assert_eq!(position.stake, 10.0);
        assert_eq!(position.entry_reason, "PriceMoveUp");
        assert!(position.is_open);
    }

    #[test]
    fn calculates_unrealised_profit_for_long_position() {
        let position = PaperPosition::new_long("token-1", 0.50, 10.0, "PriceMoveUp");

        let pnl = position.unrealised_pnl(0.60);

        const TOLERANCE: f64 = 1e-9;
        assert!((pnl - 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn calculates_unrealised_loss_for_long_position() {
        let position = PaperPosition::new_long("token-1", 0.50, 10.0, "PriceMoveUp");

        let pnl = position.unrealised_pnl(0.40);

        const TOLERANCE: f64 = 1e-9;
        assert!((pnl + 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn opens_long_position_when_no_position_is_open() {
        let mut trader = PaperTrader::new(10.0);

        let opened = trader.open_long("token-1", 0.50, "PriceMoveUp");

        assert!(opened);
        assert!(trader.has_open_position());

        let position = trader.open_position.unwrap();

        assert_eq!(position.token_id, "token-1");
        assert_eq!(position.entry_price, 0.50);
        assert_eq!(position.stake, 10.0);
        assert_eq!(position.entry_reason, "PriceMoveUp");
    }

    #[test]
    fn does_not_open_second_position_when_one_is_already_open() {
        let mut trader = PaperTrader::new(10.0);

        let first_opened = trader.open_long("token-1", 0.50, "PriceMoveUp");
        let second_opened = trader.open_long("token-1", 0.55, "PriceMoveUp");

        assert!(first_opened);
        assert!(!second_opened);

        let position = trader.open_position.unwrap();

        assert_eq!(position.entry_price, 0.50);
    }

    #[test]
    fn closes_position_when_take_profit_is_hit() {
        let mut trader = PaperTrader::new(10.0);

        trader.open_long("token-1", 0.50, "PriceMoveUp");

        let closed_trade = trader
            .maybe_close_position(0.55, 0.10, 0.05)
            .expect("expected trade to close");

        assert_eq!(closed_trade.exit_reason, PaperExitReason::TakeProfit);
        assert_eq!(closed_trade.exit_price, 0.55);
        assert!(closed_trade.pnl > 0.0);
        assert!(!trader.has_open_position());
        assert_eq!(trader.closed_trades.len(), 1);
    }

    #[test]
    fn closes_position_when_stop_loss_is_hit() {
        let mut trader = PaperTrader::new(10.0);

        trader.open_long("token-1", 0.50, "PriceMoveUp");

        let closed_trade = trader
            .maybe_close_position(0.475, 0.10, 0.05)
            .expect("expected trade to close");

        assert_eq!(closed_trade.exit_reason, PaperExitReason::StopLoss);
        assert_eq!(closed_trade.exit_price, 0.475);
        assert!(closed_trade.pnl < 0.0);
        assert!(!trader.has_open_position());
        assert_eq!(trader.closed_trades.len(), 1);
    }

    #[test]
    fn does_not_close_position_when_thresholds_are_not_hit() {
        let mut trader = PaperTrader::new(10.0);

        trader.open_long("token-1", 0.50, "PriceMoveUp");

        let closed_trade = trader.maybe_close_position(0.51, 0.10, 0.05);

        assert!(closed_trade.is_none());
        assert!(trader.has_open_position());
        assert_eq!(trader.closed_trades.len(), 0);
    }

    #[test]
    fn tracks_realised_pnl_after_close() {
        let mut trader = PaperTrader::new(10.0);

        trader.open_long("token-1", 0.50, "PriceMoveUp");
        let closed_trade = trader
            .maybe_close_position(0.55, 0.10, 0.05)
            .expect("expected trade to close");

        const TOLERANCE: f64 = 1e-9;
        assert!((trader.realised_pnl - closed_trade.pnl).abs() < TOLERANCE);
    }

    #[test]
    fn tracks_winning_and_losing_trade_counts() {
        let mut trader = PaperTrader::new(10.0);

        trader.open_long("token-1", 0.50, "PriceMoveUp");
        trader
            .maybe_close_position(0.55, 0.05, 0.05)
            .expect("expected winning trade to close");

        trader.open_long("token-1", 0.50, "PriceMoveUp");
        trader
            .maybe_close_position(0.45, 0.05, 0.05)
            .expect("expected losing trade to close");

        assert_eq!(trader.closed_trade_count(), 2);
        assert_eq!(trader.winning_trade_count(), 1);
        assert_eq!(trader.losing_trade_count(), 1);
    }
}
