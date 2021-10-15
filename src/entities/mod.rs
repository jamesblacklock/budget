pub mod account {
	use sea_orm::prelude::*;
	
	#[derive(Debug, Clone, Default, PartialEq, DeriveEntityModel)]
	#[sea_orm(table_name = "account")]
	pub struct Model {
		#[sea_orm(primary_key)]
		pub id: i32,
		#[sea_orm(unique)]
		pub name: String,
		pub is_tracking_account: bool,
		pub balance: i64,
	}

	#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
	pub enum Relation {
	    #[sea_orm(has_many = "super::transaction::Entity")]
		Transaction,
	}

	impl Related<super::transaction::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Transaction.def()
		}
	}

	impl ActiveModelBehavior for ActiveModel {}

	use crate::{AccountView};
	use sixtyfps::{
		SharedString,
	};
	
	impl Model {
		pub fn create_view(&self) -> AccountView {
			AccountView {
				id: self.id,
				name: SharedString::from(self.name.as_str()),
				balance: self.balance as f32 / 100.0,
			}
		}
	}
}

pub mod payee {
	use sea_orm::prelude::*;

	#[derive(Debug, Clone, Default, PartialEq, DeriveEntityModel)]
	#[sea_orm(table_name = "payee")]
	pub struct Model {
		#[sea_orm(primary_key)]
		pub id: i32,
		#[sea_orm(unique)]
		pub name: String,
	}

	#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
	pub enum Relation {
	    #[sea_orm(has_many = "super::transaction::Entity")]
		Transaction,
	}

	impl Related<super::transaction::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Transaction.def()
		}
	}

	impl ActiveModelBehavior for ActiveModel {}
}

pub mod category {
	use sea_orm::prelude::*;

	#[derive(Debug, Clone, Default, PartialEq, DeriveEntityModel)]
	#[sea_orm(table_name = "category")]
	pub struct Model {
		#[sea_orm(primary_key)]
		pub id: i32,
		pub group_id: i32,
		#[sea_orm(unique)]
		pub name: String,
	}

	#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
	pub enum Relation {
	    #[sea_orm(has_many = "super::transaction::Entity")]
		Transaction,
	}

	impl Related<super::transaction::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Transaction.def()
		}
	}

	impl ActiveModelBehavior for ActiveModel {}
}

pub mod transaction {
	use sea_orm::prelude::*;
	use smart_default::SmartDefault;
	use chrono::Local;

	#[derive(Debug, Clone, SmartDefault, PartialEq, DeriveEntityModel)]
	#[sea_orm(table_name = "transaction")]
	pub struct Model {
		#[sea_orm(primary_key)]
		pub id: i32,
		#[default(Local::now().naive_local())]
		pub timestamp: DateTime,
		pub account_id: i32,
		#[sea_orm(nullable)]
		pub payee_id: Option<i32>,
		#[sea_orm(nullable)]
		pub transfer_account_id: Option<i32>,
		pub category_id: i32,
		pub memo: String,
		pub amount: i64,
	}

	#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
	pub enum Relation {
	    #[sea_orm(
			belongs_to = "super::account::Entity", 
			from = "Column::AccountId", 
			to = "super::account::Column::Id")]
		Account,
	    #[sea_orm(
			belongs_to = "super::category::Entity", 
			from = "Column::CategoryId", 
			to = "super::category::Column::Id")]
		Category,
	    #[sea_orm(
			belongs_to = "super::payee::Entity", 
			from = "Column::PayeeId", 
			to = "super::payee::Column::Id")]
		Payee,
	}

	impl Related<super::account::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Account.def()
		}
	}
	
	impl Related<super::category::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Category.def()
		}
	}
	
	impl Related<super::payee::Entity> for Entity {
		fn to() -> RelationDef {
			Relation::Payee.def()
		}
	}

	impl ActiveModelBehavior for ActiveModel {}

	use sixtyfps::{
		SharedString,
	};
	use sea_orm::DatabaseConnection;
	use crate::{Error, TransactionView};

	impl Model {
		pub async fn create_view(&self, db: &DatabaseConnection) -> Result<TransactionView, Error> {
			let account = self.find_related(super::account::Entity)
				.one(db).await.map_err(Error::DbError)?.unwrap();
			let category = self.find_related(super::category::Entity)
				.one(db).await.map_err(Error::DbError)?.unwrap();
			let payee = self.find_related(super::payee::Entity)
				.one(db).await.map_err(Error::DbError)?;
			
			Ok(TransactionView {
				account: SharedString::from(account.name),
				category: SharedString::from(category.name),
				payee: SharedString::from(payee.unwrap().name),
				memo: SharedString::from(&self.memo),
				timestamp: SharedString::from(self.timestamp.format("%Y-%m-%d").to_string()),
				amount: self.amount as f32 / 100.0,
				id: self.id,
			})
		}
	}
}