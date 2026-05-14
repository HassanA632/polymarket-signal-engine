#[derive(Debug, Clone, PartialEq)]
pub enum PaperPositionSide {
    Long,
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
}

impl PaperTrader {
    pub fn new(stake: f64) -> Self {
        Self {
            stake,
            open_position: None,
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
}
