#![feature(try_blocks)]

#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::{
	io,
	thread,
	sync::mpsc,
	sync::mpsc::Receiver,
};

pub mod schema;
pub mod models;

use models::*;
use diesel::prelude::*;
use chrono::prelude::*;

#[derive(Debug)]
pub enum Error {
	DbError(diesel::result::Error),
	IOError(io::Error),
	InternalError(String),
	ParseIntError(std::num::ParseIntError)
}

pub enum Message {
	AddAccount(String, i32, bool),
	AddCategory(String),
	AddTransaction(String, String, String, String, i32),
	AccountSelected(i32),
	MonthChanged(i32, i32),
	Terminate,
}

// fn main() {
// 	use slint_interpreter::{ComponentDefinition, ComponentCompiler};

// 	let mut compiler = ComponentCompiler::default();
// 	let definition = task::block_on(compiler.build_from_path("src/ui/app.slint"));
// 	slint_interpreter::print_diagnostics(&compiler.diagnostics());
// 	if let Some(definition) = definition {
// 		let instance = definition.create();
// 		instance.run();
// 	}
// }

// fn main() {
// 	slint_build::compile("src/ui/app.slint").unwrap();
// }

slint::include_modules!();
use slint::{
	VecModel,
	SharedString,
	Weak,
};

use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
	SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


fn main() {
	let (sender, receiver) = mpsc::channel::<Message>();
	
	let component = App::new();
	
	let sender_clone = sender.clone();
	component.on_add_account(move |name, balance, is_tracking_account| {
		sender_clone.send(Message::AddAccount(
			name.as_str().to_owned(),
			balance as i32,
			is_tracking_account,
		)).unwrap();
	});
	let sender_clone = sender.clone();
	component.on_add_category(move |name| {
		sender_clone.send(Message::AddCategory(name.as_str().to_owned())).unwrap();
	});
	let sender_clone = sender.clone();
	component.on_add_tx(move |account_name, payee_name, category_name, memo, amount| {
		sender_clone.send(Message::AddTransaction(
			account_name.as_str().to_owned(),
			payee_name.as_str().to_owned(),
			category_name.as_str().to_owned(),
			memo.as_str().to_owned(),
			amount as i32,
		)).unwrap();
	});
	let sender_clone = sender.clone();
	component.on_account_selected(move |account_id| {
		sender_clone.send(Message::AccountSelected(account_id)).unwrap();
	});
	let sender_clone = sender.clone();
	component.on_month_changed(move |month, year| {
		sender_clone.send(Message::MonthChanged(month, year)).unwrap();
	});

	let component_weak = component.as_weak();
	thread::spawn(move || worker_thread(component_weak, receiver));

	component.run();
}

fn load_accounts(component: Weak<App>, conn: &SqliteConnection) -> Result<(), Error> {
	let accounts = schema::accounts::table
		.load::<Account>(conn).map_err(|e| Error::DbError(e))?;
	
	component.upgrade_in_event_loop(move |component| {
		let mut account_views = Vec::new();
		for account in accounts.iter() {
			account_views.push(account.create_view());
		}
		component.set_accounts(VecModel::from_slice(&account_views));
		component.set_account_names(VecModel::from_slice(&accounts.iter()
			.map(|account| SharedString::from(&account.name)).collect::<Vec<_>>()));
	});

	Ok(())
}

fn load_categories(component: Weak<App>, conn: &SqliteConnection) -> Result<(), Error> {
	let categories = schema::categories::table
		.order(schema::categories::order)
		.load::<Category>(conn).map_err(|e| Error::DbError(e))?;
	
	component.upgrade_in_event_loop(move |component| {
		let all_categories = categories.iter()
			.map(|category| SharedString::from(&category.name)).collect::<Vec<_>>();
		component.set_categories(VecModel::from_slice(&all_categories));
	});

	Ok(())
}

fn load_transactions(component: Weak<App>, account_id: Option<i32>, conn: &SqliteConnection) -> Result<(), Error> {
	let query = schema::txs::table
		.left_join(schema::accounts::table)
		.left_join(schema::categories::table)
		.left_join(schema::payees::table);
	
	let txs = if let Some(account_id) = account_id {
		query
			.filter(schema::txs::account_id.eq(account_id))
			.load::<(Tx, Option<Account>, Option<Category>, Option<Payee>)>(conn).map_err(|e| Error::DbError(e))?
	} else {
		query
			.load::<(Tx, Option<Account>, Option<Category>, Option<Payee>)>(conn).map_err(|e| Error::DbError(e))?
	};

	let tx_views: Vec<_> = txs.into_iter()
		.map(|(tx, account, category, payee)| tx.create_view(account, category, payee))
		.collect();

	component.upgrade_in_event_loop(move |component| {
		component.set_transactions(VecModel::from_slice(tx_views.as_slice()));
	});

	Ok(())
}

fn load_budget(component: Weak<App>, month: i32, year: i32, conn: &SqliteConnection) -> Result<(), Error> {
	use schema::budgets::columns as b;
	use schema::categories::columns as c;
	let rows: Vec<_> = schema::categories::table
		.left_join(schema::budgets::table
			.on(
				b::category_id.eq(c::id)
				.and(b::year.eq(year))
				.and(b::month.eq(month))
			)
		)
		.order(c::order)
		.select((c::id, c::name, b::assigned.nullable(), b::activity.nullable(), b::available.nullable()))
		.load::<BudgetCategoryViewQueryable>(conn)
		.map_err(|e| Error::DbError(e))?
		.into_iter()
		.map(|row| row.into_view())
		.collect();

	component.upgrade_in_event_loop(move |component| {
		component.set_current_year(year);
		component.set_current_month(month as i32);
		component.set_budget_categories(VecModel::from_slice(&rows[1..]));
	});

	Ok(())
}

fn worker_thread(component: Weak<App>, receiver: Receiver<Message>) -> Result<(), Error> {
	let conn = establish_connection();
	
	load_accounts(component.clone(), &conn)?;
	load_categories(component.clone(), &conn)?;
	load_transactions(component.clone(), None, &conn)?;
	let now = Local::now();
	load_budget(component.clone(), now.month0() as i32, now.year(), &conn)?;

	let mut selected_account = None;
	loop {
		let message = match receiver.recv() {
			Ok(message) => message,
			Err(_err) => Message::Terminate,
		};

		let result: Result<(), Error> = try {
			match message {
				Message::AddAccount(name, balance, is_tracking_account) => {
					let account = NewAccount {
						name: &name,
						is_tracking_account,
						balance,
					};

					// 
					diesel::insert_into(schema::accounts::table)
						.values(&account)
						.execute(&conn)
						.map_err(|e| Error::DbError(e))?;
					
					let account = schema::accounts::table
						.filter(schema::accounts::name.eq(name))
						.first::<Account>(&conn)
						.map_err(|e| Error::DbError(e))?;
					
					if balance != 0 {
						let now = Local::now();
						let tx = NewTx {
							timestamp: now.naive_local(),
							month: now.month0() as i32,
							year: now.year(),
							account_id: account.id,
							payee_id: Some(0),
							transfer_account_id: None,
							category_id: Some(0),
							memo: "",
							amount: balance,
							cleared: true,
						};

						diesel::insert_into(schema::txs::table)
							.values(&tx)
							.execute(&conn)
							.map_err(|e| Error::DbError(e))?;
						
						load_transactions(component.clone(), selected_account, &conn)?;
					}
					
					load_accounts(component.clone(), &conn)?;
				},
				Message::AddCategory(name) => {
					let category = NewCategory { name: &name, group_id: 0 };
					diesel::insert_into(schema::categories::table)
							.values(&category)
							.execute(&conn)
							.map_err(|e| Error::DbError(e))?;
					load_categories(component.clone(), &conn)?;
				},
				Message::AddTransaction(account_name, payee_name, category_name, memo, amount) => {
					let account = schema::accounts::table
						.filter(schema::accounts::name.eq(account_name))
						.first::<Account>(&conn)
						.map_err(|e| Error::DbError(e))?;
					let category = schema::categories::table
						.filter(schema::categories::name.eq(category_name))
						.first::<Category>(&conn)
						.map_err(|e| Error::DbError(e))?;
					let payee: Option<Payee> = schema::payees::table
						.filter(schema::payees::name.eq(&payee_name))
						.first(&conn)
						.optional()
						.map_err(|e| Error::DbError(e))?;
					let payee = if let Some(payee) = payee {
						payee
					} else {
						let payee = NewPayee { name: &payee_name };
						diesel::insert_into(schema::payees::table)
							.values(&payee)
							.execute(&conn)
							.map_err(|e| Error::DbError(e))?;
						schema::payees::table
							.filter(schema::payees::name.eq(payee_name))
							.first(&conn)
							.map_err(|e| Error::DbError(e))?
					};
					let now = Local::now();
					let tx = NewTx {
						timestamp: now.naive_local(),
						month: now.month0() as i32,
						year: now.year(),
						account_id: account.id,
						payee_id: Some(payee.id),
						transfer_account_id: None,
						category_id: Some(category.id),
						memo: &memo,
						amount,
						cleared: false,
					};
					
					diesel::insert_into(schema::txs::table)
						.values(&tx)
						.execute(&conn)
						.map_err(|e| Error::DbError(e))?;
					
					diesel::update(schema::accounts::table.find(account.id))
						.set(schema::accounts::balance.eq(account.balance + amount))
						.execute(&conn)
						.map_err(|e| Error::DbError(e))?;

					load_transactions(component.clone(), selected_account, &conn)?;
					load_accounts(component.clone(), &conn)?;
				},
				Message::AccountSelected(account_id) => {
					selected_account = if account_id > 0 { Some(account_id) } else { None };
					load_transactions(component.clone(), selected_account, &conn)?;
				},
				Message::MonthChanged(month, year) => {
					load_budget(component.clone(), month, year, &conn)?;
				},
				Message::Terminate => {
					break Ok(());
				}
			}
		};

		if let Err(err) = result {
			eprintln!("{:?}", err);
		}
	}
}
