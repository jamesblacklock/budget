use crate::{ AccountView, TxView, BudgetCategoryView };
use chrono::NaiveDateTime;
use crate::schema::{ accounts, categories, budgets, payees, txs };

pub struct SharedString;

pub mod shared_string {
	pub fn dollars(n: Option<i32>) -> slint::SharedString {
		if let Some(n) = n {
			format!("{}${:02.2}", if n < 0 {"-"} else {""}, n.abs() as f32 / 100.0).into()
		} else {
			slint::SharedString::default()
		}
	}

	pub fn nonzero_dollars(n: Option<i32>) -> slint::SharedString {
		let n = n.unwrap_or_default();
		if n != 0 {
			dollars(Some(n))
		} else {
			slint::SharedString::default()
		}
	}

	pub fn option<S: Into<String>>(s: Option<S>) -> slint::SharedString {
		if let Some(s) = s {
			s.into().into()
		} else {
			slint::SharedString::default()
		}
	}

	pub fn from(s: &str) -> slint::SharedString {
		slint::SharedString::from(s)
	}

	pub fn timestamp(t: chrono::NaiveDateTime) -> slint::SharedString {
		t.format("%Y-%m-%d").to_string().into()
	}
}

#[derive(Queryable)]
pub struct BudgetCategoryViewQueryable {
	id: i32,
	name: String,
	assigned: Option<i32>,
	activity: Option<i32>,
	available: Option<i32>,
}

impl BudgetCategoryViewQueryable {
	pub fn into_view(self) -> BudgetCategoryView {
		BudgetCategoryView {
			id: self.id,
			name: self.name.into(),
			assigned: shared_string::nonzero_dollars(self.assigned),
			activity: shared_string::dollars(self.activity.or(Some(0))),
			available: shared_string::dollars(self.available.or(Some(0))),
		}
	}
}

#[derive(Queryable)]
pub struct Account {
	pub id: i32,
	pub name: String,
	pub is_tracking_account: bool,
	pub balance: i32,
}

impl Account {
	pub fn create_view(&self) -> AccountView {
		AccountView {
			id: self.id,
			name: shared_string::from(self.name.as_str()),
			balance: self.balance as f32 / 100.0,
		}
	}
}

#[derive(Insertable)]
#[table_name="accounts"]
pub struct NewAccount<'a> {
	pub name: &'a str,
	pub is_tracking_account: bool,
	pub balance: i32,
}

#[derive(Queryable)]
pub struct Budget {
	pub id: i32,
	pub month: i32,
	pub year: i32,
	pub category_id: i32,
	pub assigned: i32,
	pub activity: i32,
	pub available: i32,
}

impl Budget {
	pub fn update_view(&self, view: BudgetCategoryView) -> BudgetCategoryView {
		BudgetCategoryView {
			assigned: shared_string::nonzero_dollars(Some(self.assigned)),
			activity: shared_string::dollars(Some(self.activity)),
			available: shared_string::dollars(Some(self.available)),
			..view
		}
	}
}

#[derive(Insertable)]
#[table_name="budgets"]
pub struct NewBudget {
	pub month: i32,
	pub year: i32,
	pub category_id: i32,
	pub assigned: i32,
	pub activity: i32,
	pub available: i32,
}

#[derive(Queryable)]
pub struct Category {
	pub id: i32,
	pub group_id: i32,
	pub name: String,
	pub order: i32,
}

#[derive(Insertable)]
#[table_name="categories"]
pub struct NewCategory<'a> {
	pub group_id: i32,
	pub name: &'a str,
}

#[derive(Queryable)]
pub struct Payee {
	pub id: i32,
	pub name: String,
}

#[derive(Insertable)]
#[table_name="payees"]
pub struct NewPayee<'a> {
	pub name: &'a str,
}

#[derive(Queryable)]
pub struct Tx {
	pub id: i32,
	pub timestamp: NaiveDateTime,
	pub month: i32,
	pub year: i32,
	pub account_id: i32,
	pub payee_id: Option<i32>,
	pub transfer_account_id: Option<i32>,
	pub category_id: Option<i32>,
	pub memo: String,
	pub amount: i32,
	pub cleared: bool,
}

impl Tx {
	pub fn create_view(
		&self,
		account: Option<Account>,
		category: Option<Category>,
		payee: Option<Payee>) -> TxView {
		
		TxView {
			id:        self.id,
			account:   shared_string::option(account.map(|e| e.name)),
			category:  shared_string::option(category.map(|e| e.name)),
			payee:     shared_string::option(payee.map(|e| e.name)),
			memo:      shared_string::from(&self.memo),
			timestamp: shared_string::timestamp(self.timestamp),
			inflow:    shared_string::dollars(if self.amount > 0 { Some(self.amount) } else { None }),
			outflow:   shared_string::dollars(if self.amount < 0 { Some(-self.amount) } else { None }),
			cleared:   self.cleared,
		}
	}
}

#[derive(Insertable)]
#[table_name="txs"]
pub struct NewTx<'a> {
	pub timestamp: NaiveDateTime,
	pub month: i32,
	pub year: i32,
	pub account_id: i32,
	pub payee_id: Option<i32>,
	pub transfer_account_id: Option<i32>,
	pub category_id: Option<i32>,
	pub memo: &'a str,
	pub amount: i32,
	pub cleared: bool,
}