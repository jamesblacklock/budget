use crate::{ AccountView, TxView, BudgetCategoryView };
use slint::SharedString;
use chrono::NaiveDateTime;
use crate::schema::{ accounts, categories, payees, txs };

fn dollars(n: Option<i32>) -> SharedString {
	if let Some(n) = n {
		format!("{}${:02.2}", if n < 0 {"-"} else {""}, n.abs() as f32 / 100.0).into()
	} else {
		SharedString::default()
	}
}

#[derive(Queryable)]
pub struct BudgetCategoryViewQueryable {
	category_id: i32,
	name: String,
	assigned: Option<i32>,
	activity: Option<i32>,
	available: Option<i32>,
}

impl BudgetCategoryViewQueryable {
	pub fn into_view(self) -> BudgetCategoryView {
		BudgetCategoryView {
			category_id: self.category_id,
			name: self.name.into(),
			assigned: dollars(self.assigned),
			activity: dollars(self.activity.or(Some(0))),
			available: dollars(self.available.or(Some(0))),
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
			name: SharedString::from(self.name.as_str()),
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
			account: SharedString::from(account.map(|e| e.name).unwrap_or("".to_owned())),
			category: SharedString::from(category.map(|e| e.name).unwrap_or("".to_owned())),
			payee: SharedString::from(payee.map(|e| e.name).unwrap_or("".to_owned())),
			memo: SharedString::from(&self.memo),
			timestamp: SharedString::from(self.timestamp.format("%Y-%m-%d").to_string()),
			inflow: dollars(if self.amount > 0 { Some(self.amount) } else { None }),
			outflow: dollars(if self.amount < 0 { Some(-self.amount) } else { None }),
			cleared: self.cleared,
			id: self.id,
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