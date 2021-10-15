#![feature(try_blocks)]

use std::{
	path::Path,
	fs::File,
	io,
	thread,
	sync::mpsc,
	sync::mpsc::Receiver,
};

use async_std::task;

use sea_query::{
	TableCreateStatement,
};
use sea_orm::{
	EntityTrait,
	QueryFilter,
	ColumnTrait,
	Database,
	DatabaseConnection,
	DbErr,
	Schema,
};

#[derive(Debug)]
pub enum Error {
	DbError(DbErr),
	IOError(io::Error),
	InternalError(String),
	ParseIntError(std::num::ParseIntError)
}

pub enum Message {
	AddAccount(String, i64, bool),
	AddCategory(String),
	AddTransaction(String, String, String, String, i64),
	AccountSelected(i32),
	Terminate,
}

// fn main() {
// 	sixtyfps_build::compile("src/ui/app.60").unwrap();
// }

sixtyfps::include_modules!();
use sixtyfps::{
	VecModel,
	SharedString,
	Weak,
};

pub(crate) mod entities;
pub(crate) mod api;

fn main() {
	let (sender, receiver) = mpsc::channel::<Message>();
	
	let component = App::new();
	
	let sender_clone = sender.clone();
	component.on_add_account(move |name, balance, is_tracking_account| {
		sender_clone.send(Message::AddAccount(
			name.as_str().to_owned(),
			balance as i64,
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
			amount as i64,
		)).unwrap();
	});
	let sender_clone = sender.clone();
	component.on_account_selected(move |account_id| {
		sender_clone.send(Message::AccountSelected(account_id)).unwrap();
	});

	let component_weak = component.as_weak();
	thread::spawn(move || {
		task::block_on(worker_thread(component_weak, receiver))
	});

	component.run();
}

async fn load_accounts(component: Weak<App>, db: &DatabaseConnection) -> Result<(), Error> {
	let accounts = api::account::list(&db).await?;
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

async fn load_categories(component: Weak<App>, db: &DatabaseConnection) -> Result<(), Error> {
	let categories = api::category::list(&db).await?;
	component.upgrade_in_event_loop(move |component| {
		component.set_categories(VecModel::from_slice(&categories.iter()
			.map(|category| SharedString::from(&category.name)).collect::<Vec<_>>()));
	});

	Ok(())
}

async fn load_transactions(component: Weak<App>, account_id: Option<i32>, db: &DatabaseConnection) -> Result<(), Error> {
	let transactions = api::transaction::list(account_id, &db).await?;
	let mut transaction_views = Vec::new();
	for transaction in transactions {
		transaction_views.push(transaction.create_view(&db).await?);
	}
	component.upgrade_in_event_loop(move |component| {
		component.set_transactions(VecModel::from_slice(transaction_views.as_slice()));
	});

	Ok(())
}

async fn worker_thread(component: Weak<App>, receiver: Receiver<Message>) -> Result<(), Error> {
	let db_path = Path::new("./dev.db");
	let create_schema = if db_path.exists() {
		false
	} else {
		File::create(db_path).map_err(Error::IOError)?;
		true
	};

	let db: DatabaseConnection = Database::connect("sqlite://dev.db").await.map_err(Error::DbError)?;

	if create_schema {
		let stmt: TableCreateStatement = Schema::create_table_from_entity(entities::account::Entity);
		db.execute(db.get_database_backend().build(&stmt)).await.map_err(Error::DbError)?;
		
		let stmt: TableCreateStatement = Schema::create_table_from_entity(entities::transaction::Entity);
		db.execute(db.get_database_backend().build(&stmt)).await.map_err(Error::DbError)?;
		
		let stmt: TableCreateStatement = Schema::create_table_from_entity(entities::category::Entity);
		db.execute(db.get_database_backend().build(&stmt)).await.map_err(Error::DbError)?;
		
		let stmt: TableCreateStatement = Schema::create_table_from_entity(entities::payee::Entity);
		db.execute(db.get_database_backend().build(&stmt)).await.map_err(Error::DbError)?;
	}
	
	load_accounts(component.clone(), &db).await?;
	load_categories(component.clone(), &db).await?;
	load_transactions(component.clone(), None, &db).await?;

	let mut selected_account = None;
	loop {
		let message = receiver.recv().unwrap();
		let result: Result<(), Error> = try {
			match message {
				Message::AddAccount(name, balance, is_tracking_account) => {
					let account = entities::account::Model {
						name,
						is_tracking_account,
						balance,
						..Default::default()
					};
					api::account::add(account, &db).await?;
					load_accounts(component.clone(), &db).await?;
				},
				Message::AddCategory(name) => {
					let category = entities::category::Model {
						name,
						..Default::default()
					};
					api::category::add(category, &db).await?;
					load_categories(component.clone(), &db).await?;
				},
				Message::AddTransaction(account_name, payee_name, category_name, memo, amount) => {
					let account_match = entities::account::Entity::find()
						.filter(entities::account::Column::Name.eq(account_name.as_str()))
						.one(&db).await.map_err(Error::DbError)?.unwrap();
					let category_match = entities::category::Entity::find()
						.filter(entities::category::Column::Name.eq(category_name.as_str()))
						.one(&db).await.map_err(Error::DbError)?.unwrap();
					let payee_match = entities::payee::Entity::find()
						.filter(entities::payee::Column::Name.eq(payee_name.as_str()))
						.one(&db).await.map_err(Error::DbError)?;
					let payee_match = if let Some(payee_match) = payee_match {
						payee_match
					} else {
						let payee = entities::payee::Model {
							name: payee_name,
							..Default::default()
						};
						api::payee::add(payee, &db).await?
					};
					let transaction = entities::transaction::Model {
						account_id: account_match.id,
						payee_id: Some(payee_match.id),
						category_id: category_match.id,
						memo: memo.as_str().to_owned(),
						amount: amount as i64,
						..Default::default()
					};
					api::transaction::add(transaction, &db).await?;
					
					let mut account = api::account::find_by_id(account_match.id, &db).await?;
					account.balance += amount as i64;
					api::account::update(account, &db).await?;
					load_transactions(component.clone(), selected_account, &db).await?;
					load_accounts(component.clone(), &db).await?;
				},
				Message::AccountSelected(account_id) => {
					selected_account = if account_id > 0 { Some(account_id) } else { None };
					load_transactions(component.clone(), selected_account, &db).await?;
				},
				Message::Terminate => {
					break Ok(());
				}
			}
		};

		if let Err(err) = result {
			println!("{:?}", err);
		}
	}
}
