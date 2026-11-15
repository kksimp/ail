//! The shop's state and the journal entries that change it (spec "Journal").
//! Each entry checks its rules in the spec's order and changes nothing when one fails.
use crate::catalog::{Catalog, Item};
use crate::customers::{self, Customers};
use crate::gifts::Cards;
use crate::journal::{Buy, Entry, Want};
use crate::orders::{Order, Orders, Status};
use crate::pricing::{self, Line};
use crate::promos::{Promo, Promos};
use crate::purchasing::{PoStatus, Purchase, Purchases};
use std::fmt;

pub enum Problem {
    CategoryChanged(String),
    UnknownItem(String),
    BelowReserved(String),
    DuplicatePromo(String),
    DuplicateOrder(String),
    UnknownCustomer(String),
    Short(String),
    UnknownOrder(String),
    WrongStatus(String, &'static str),
    UnknownPromo(String),
    PromoAlready(String),
    UsedUp(String),
    NotApplicable(String),
    NoCredit(String),
    DuplicatePurchase(String),
    UnknownPurchase(String),
    PurchaseStatus(String, &'static str),
    DuplicateGift(String),
    UnknownGift(String),
    EmptyGift(String),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Problem::CategoryChanged(sku) => write!(f, "category changed {}", sku),
            Problem::UnknownItem(sku) => write!(f, "unknown item {}", sku),
            Problem::BelowReserved(sku) => write!(f, "below reserved {}", sku),
            Problem::DuplicatePromo(code) => write!(f, "duplicate promo {}", code),
            Problem::DuplicateOrder(id) => write!(f, "duplicate order {}", id),
            Problem::UnknownCustomer(cust) => write!(f, "unknown customer {}", cust),
            Problem::Short(sku) => write!(f, "short {}", sku),
            Problem::UnknownOrder(id) => write!(f, "unknown order {}", id),
            Problem::WrongStatus(id, status) => write!(f, "order {} is {}", id, status),
            Problem::UnknownPromo(code) => write!(f, "unknown promo {}", code),
            Problem::PromoAlready(id) => write!(f, "promo already on {}", id),
            Problem::UsedUp(code) => write!(f, "promo {} used up", code),
            Problem::NotApplicable(code) => write!(f, "promo {} does not apply", code),
            Problem::NoCredit(cust) => write!(f, "no credit {}", cust),
            Problem::DuplicatePurchase(po) => write!(f, "duplicate purchase {}", po),
            Problem::UnknownPurchase(po) => write!(f, "unknown purchase {}", po),
            Problem::PurchaseStatus(po, status) => write!(f, "purchase {} is {}", po, status),
            Problem::DuplicateGift(card) => write!(f, "duplicate gift {}", card),
            Problem::UnknownGift(card) => write!(f, "unknown gift {}", card),
            Problem::EmptyGift(card) => write!(f, "gift {} is empty", card),
        }
    }
}

type Done = Result<(), Problem>;

pub struct Shop {
    pub stock: Catalog,
    pub people: Customers,
    pub deals: Promos,
    pub book: Orders,
    pub buying: Purchases,
    pub cards: Cards,
}

impl Shop {
    pub fn new() -> Shop {
        Shop {
            stock: Catalog::new(),
            people: Customers::new(),
            deals: Promos::new(),
            book: Orders::new(),
            buying: Purchases::new(),
            cards: Cards::new(),
        }
    }

    /// Applies one entry; a rejected entry changes nothing.
    pub fn apply(&mut self, e: Entry) -> Done {
        match e {
            Entry::Item { sku, cat, cents } => self.item(&sku, &cat, cents),
            Entry::Tax { cat, pct } => {
                self.stock.set_rate(&cat, pct);
                Ok(())
            }
            Entry::Restock { sku, qty } => self.restock(&sku, qty),
            Entry::Adjust { sku, qty } => self.adjust(&sku, qty),
            Entry::Customer { cust, tier } => {
                self.people.set_tier(&cust, &tier);
                Ok(())
            }
            Entry::Promo { code, cat, pct, uses } => self.promo(&code, &cat, pct, uses),
            Entry::Purchase { po, buys } => self.purchase(&po, buys),
            Entry::Receive { po } => self.receive(&po),
            Entry::Gift { card, cents } => self.gift(&card, cents),
            Entry::Order { id, cust, wants } => self.order(&id, &cust, &wants),
            Entry::Apply { id, code } => self.apply_promo(&id, &code),
            Entry::Pay { id, cents } => self.pay(&id, cents),
            Entry::PayCredit { id } => self.pay_credit(&id),
            Entry::PayGift { id, card } => self.pay_gift(&id, &card),
            Entry::Ship { id } => self.ship(&id),
            Entry::Cancel { id } => self.cancel(&id),
        }
    }

    fn item(&mut self, sku: &str, cat: &str, cents: i64) -> Done {
        let mut it = match self.stock.get(sku) {
            Some(it) => it.clone(),
            None => Item { cat: cat.to_string(), price: cents, onhand: 0, reserved: 0, value: 0 },
        };
        if it.cat != cat {
            return Err(Problem::CategoryChanged(sku.to_string()));
        }
        it.price = cents;
        self.stock.put(sku, it);
        Ok(())
    }

    fn restock(&mut self, sku: &str, qty: i64) -> Done {
        let mut it = self.stock.get(sku).cloned().ok_or_else(|| Problem::UnknownItem(sku.to_string()))?;
        it.onhand += qty;
        self.stock.put(sku, it);
        Ok(())
    }

    fn adjust(&mut self, sku: &str, qty: i64) -> Done {
        let it = self.stock.get(sku).ok_or_else(|| Problem::UnknownItem(sku.to_string()))?;
        if qty < it.reserved {
            return Err(Problem::BelowReserved(sku.to_string()));
        }
        self.stock.stocktake(sku, qty);
        Ok(())
    }

    fn promo(&mut self, code: &str, cat: &str, pct: i64, uses: i64) -> Done {
        if self.deals.has(code) {
            return Err(Problem::DuplicatePromo(code.to_string()));
        }
        self.deals.add(code, Promo { cat: cat.to_string(), pct, uses, used: 0 });
        Ok(())
    }

    fn purchase(&mut self, po: &str, buys: Vec<Buy>) -> Done {
        if self.buying.has(po) {
            return Err(Problem::DuplicatePurchase(po.to_string()));
        }
        for b in &buys {
            if !self.stock.has(&b.sku) {
                return Err(Problem::UnknownItem(b.sku.clone()));
            }
        }
        self.buying.put(Purchase { po: po.to_string(), status: PoStatus::Ordered, buys });
        Ok(())
    }

    fn receive(&mut self, po: &str) -> Done {
        let mut p = self.buying.find(po).cloned().ok_or_else(|| Problem::UnknownPurchase(po.to_string()))?;
        if p.status != PoStatus::Ordered {
            return Err(Problem::PurchaseStatus(po.to_string(), p.status.text()));
        }
        for b in &p.buys {
            self.stock.receive(&b.sku, b.qty, b.qty * b.cents);
        }
        p.status = PoStatus::Received;
        self.buying.put(p);
        Ok(())
    }

    fn gift(&mut self, card: &str, cents: i64) -> Done {
        if self.cards.has(card) {
            return Err(Problem::DuplicateGift(card.to_string()));
        }
        self.cards.sell(card, cents);
        Ok(())
    }

    fn order(&mut self, id: &str, cust: &str, wants: &[Want]) -> Done {
        if self.book.has(id) {
            return Err(Problem::DuplicateOrder(id.to_string()));
        }
        if !self.people.has(cust) {
            return Err(Problem::UnknownCustomer(cust.to_string()));
        }
        if let Some(w) = wants.iter().find(|w| !self.stock.has(&w.sku)) {
            return Err(Problem::UnknownItem(w.sku.clone()));
        }
        if let Some(w) = wants.iter().find(|w| self.stock.item(&w.sku).available() < w.qty) {
            return Err(Problem::Short(w.sku.clone()));
        }
        let tier = customers::tier_pct(&self.people.get(cust).tier);
        let mut lines: Vec<Line> = Vec::new();
        for w in wants {
            let it = self.stock.item(&w.sku);
            lines.push(pricing::fresh_line(&w.sku, w.qty, it.price, &it.cat, self.stock.rate(&it.cat), tier));
            self.stock.reserve(&w.sku, w.qty);
        }
        let placed = Order { id: id.to_string(), cust: cust.to_string(), status: Status::Open, lines, promo: String::new(), paid: 0 };
        let priced = self.repriced(&placed);
        self.book.put(priced);
        Ok(())
    }

    /// The order with every line priced again under its promo, if it has one.
    fn repriced(&self, o: &Order) -> Order {
        let (cat, pct) = match self.deals.get(&o.promo) {
            Some(p) => (p.cat.clone(), p.pct),
            None => (String::new(), 0),
        };
        let mut out = o.clone();
        out.lines = o.lines.iter().map(|l| pricing::priced(l, &cat, pct)).collect();
        out
    }

    /// The order `id`, which must have status `want`.
    fn order_in(&self, id: &str, want: Status) -> Result<Order, Problem> {
        let o = self.book.find(id).ok_or_else(|| Problem::UnknownOrder(id.to_string()))?;
        if o.status != want {
            return Err(Problem::WrongStatus(id.to_string(), o.status.text()));
        }
        Ok(o.clone())
    }

    fn apply_promo(&mut self, id: &str, code: &str) -> Done {
        let o = self.order_in(id, Status::Open)?;
        let p = self.deals.get(code).ok_or_else(|| Problem::UnknownPromo(code.to_string()))?;
        if !o.promo.is_empty() {
            return Err(Problem::PromoAlready(id.to_string()));
        }
        if p.used >= p.uses {
            return Err(Problem::UsedUp(code.to_string()));
        }
        if !o.has_cat(&p.cat) {
            return Err(Problem::NotApplicable(code.to_string()));
        }
        self.deals.count_use(code, 1);
        let mut with_promo = o;
        with_promo.promo = code.to_string();
        let mut done = self.repriced(&with_promo);
        if done.paid >= done.total() {
            self.people.add_credit(&done.cust, done.paid - done.total());
            done.paid = done.total();
            done.status = Status::Paid;
        }
        self.book.put(done);
        Ok(())
    }

    fn pay(&mut self, id: &str, cents: i64) -> Done {
        let o = self.order_in(id, Status::Open)?;
        self.settle(o, cents);
        Ok(())
    }

    fn pay_credit(&mut self, id: &str) -> Done {
        let o = self.order_in(id, Status::Open)?;
        let credit = self.people.get(&o.cust).credit;
        if credit == 0 {
            return Err(Problem::NoCredit(o.cust.clone()));
        }
        let amount = credit.min(o.due());
        self.people.add_credit(&o.cust, -amount);
        self.settle(o, amount);
        Ok(())
    }

    fn pay_gift(&mut self, id: &str, card: &str) -> Done {
        let o = self.order_in(id, Status::Open)?;
        let c = self.cards.find(card).ok_or_else(|| Problem::UnknownGift(card.to_string()))?;
        if c.balance == 0 {
            return Err(Problem::EmptyGift(card.to_string()));
        }
        let amount = c.balance.min(o.due());
        self.cards.draw(card, amount);
        self.settle(o, amount);
        Ok(())
    }

    /// Counts `amount` towards what is due on `o`; any amount above it goes to the customer's credit.
    fn settle(&mut self, o: Order, amount: i64) {
        let applied = amount.min(o.due());
        let mut after = o;
        after.paid += applied;
        self.people.add_credit(&after.cust, amount - applied);
        if after.paid == after.total() {
            after.status = Status::Paid;
        }
        self.book.put(after);
    }

    fn ship(&mut self, id: &str) -> Done {
        let mut o = self.order_in(id, Status::Paid)?;
        for l in o.lines.iter_mut() {
            l.cost = self.stock.take(&l.sku, l.qty);
        }
        o.status = Status::Shipped;
        self.book.put(o);
        Ok(())
    }

    fn cancel(&mut self, id: &str) -> Done {
        let mut o = self.book.find(id).cloned().ok_or_else(|| Problem::UnknownOrder(id.to_string()))?;
        if o.status == Status::Shipped || o.status == Status::Cancelled {
            return Err(Problem::WrongStatus(id.to_string(), o.status.text()));
        }
        for l in &o.lines {
            self.stock.reserve(&l.sku, -l.qty);
        }
        self.people.add_credit(&o.cust, o.paid);
        o.paid = 0;
        if !o.promo.is_empty() {
            self.deals.count_use(&o.promo, -1);
        }
        o.status = Status::Cancelled;
        self.book.put(o);
        Ok(())
    }
}
