use crate::db::entities::owner::Model as OwnerModel;
use crate::db::entities::possession::Model as PossessionModel;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type TradesMutex = Arc<Mutex<Vec<Trade>>>;

pub fn get_trades_mutex() -> TradesMutex {
    return Arc::new(Mutex::new(Vec::new()));
}

pub trait TradeLogic {
    fn add_to_trade(&mut self, owner: &OwnerModel, item: &PossessionModel) -> bool;
    fn remove_from_trade(&mut self, owner: &OwnerModel, item: &PossessionModel) -> bool;
    fn change_trade_status(&mut self, owner: &OwnerModel);
}

pub struct Trade {
    id: u64,

    trader_1: OwnerModel,
    trade_1_accept: bool,
    trade_1_items: Vec<i32>,

    trader_2: OwnerModel,
    trade_2_accept: bool,
    trade_2_items: Vec<i32>,
}

impl TradeLogic for Trade {
    fn add_to_trade(&mut self, owner: &OwnerModel, item: &PossessionModel) -> bool {
        if self.trader_1.id == owner.id {
            self.trade_1_items.push(item.id);
            self.trade_1_accept = false;
            self.trade_2_accept = false;
            return true;
        } else if self.trader_2.id == owner.id {
            self.trade_2_items.push(item.id);
            self.trade_1_accept = false;
            self.trade_2_accept = false;
            return true;
        }
        return false;
    }

    fn remove_from_trade(&mut self, owner: &OwnerModel, item: &PossessionModel) -> bool {
        if self.trader_1.id == owner.id {
            if let Some(i) = self
                .trade_1_items
                .iter()
                .position(|value| *value == item.id)
            {
                self.trade_1_items.swap_remove(i);
                self.trade_1_accept = false;
                self.trade_2_accept = false;
                return true;
            }
        } else if self.trader_2.id == owner.id {
            if let Some(i) = self
                .trade_2_items
                .iter()
                .position(|value| *value == item.id)
            {
                self.trade_2_items.swap_remove(i);
                self.trade_1_accept = false;
                self.trade_2_accept = false;
                return true;
            }
        }
        return false;
    }
    fn change_trade_status(&mut self, owner: &OwnerModel) {
        if self.trader_1.id == owner.id {
            self.trade_1_accept = !self.trade_1_accept;
        } else if self.trader_2.id == owner.id {
            self.trade_2_accept = !self.trade_2_accept;
        }
    }
}
