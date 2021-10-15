pub mod account {
	use sea_orm::{
		DatabaseConnection,
		EntityTrait,
		ActiveModelTrait,
		Set,
	};
	use crate::Error;
	use crate::entities::account as model;
	
	pub async fn add(acc: model::Model, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let active_model = model::ActiveModel {
			name: Set(acc.name.clone()),
			balance: Set(acc.balance),
			is_tracking_account: Set(acc.is_tracking_account),
			..Default::default()
		};
		
		let result = active_model.insert(db).await.map_err(Error::DbError)?;

		Ok(model::Model {
			id: result.id.unwrap(),
			..acc
		})
	}

	pub async fn find_by_id(id: i32, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let model = model::Entity::find_by_id(id).one(db).await.map_err(Error::DbError)?;
		let model = model.ok_or(Error::InternalError(format!("account not found: {}", id).to_owned()))?;
		
		Ok(model)
	}

	pub async fn update(acc: model::Model, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let mut active_model: model::ActiveModel = acc.clone().into();
		active_model.name = Set(acc.name.clone());
		active_model.balance = Set(acc.balance);
		active_model.is_tracking_account = Set(acc.is_tracking_account);
		active_model.update(&db).await.map_err(Error::DbError)?;
		
		Ok(acc)
	}

	pub async fn remove(id: i32, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let model = model::Entity::find_by_id(id).one(db).await.map_err(Error::DbError)?;
		let model = model.ok_or(Error::InternalError(format!("account not found: {}", id).to_owned()))?;
		let active_model: model::ActiveModel = model.clone().into();
		let result = active_model.delete(db).await.map_err(Error::DbError)?;
		assert!(result.rows_affected == 1);
		
		Ok(model)
	}

	pub async fn list(db: &DatabaseConnection) -> Result<Vec<model::Model>, Error> {
		model::Entity::find().all(db).await.map_err(Error::DbError)
	}
}

pub mod category {
	use sea_orm::{
		DatabaseConnection,
		EntityTrait,
		ActiveModelTrait,
		Set,
	};
	use crate::Error;
	use crate::entities::category as model;
	
	pub async fn add(cat: model::Model, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let active_model = model::ActiveModel {
			name: Set(cat.name.clone()),
			group_id: Set(0),
			..Default::default()
		};
		
		let result = active_model.insert(db).await.map_err(Error::DbError)?;

		Ok(model::Model {
			id: result.id.unwrap(),
			..cat
		})
	}

	pub async fn remove(id: i32, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let model = model::Entity::find_by_id(id).one(db).await.map_err(Error::DbError)?;
		let model = model.ok_or(Error::InternalError(format!("category not found: {}", id).to_owned()))?;
		let active_model: model::ActiveModel = model.clone().into();
		let result = active_model.delete(db).await.map_err(Error::DbError)?;
		assert!(result.rows_affected == 1);
		
		Ok(model)
	}

	pub async fn list(db: &DatabaseConnection) -> Result<Vec<model::Model>, Error> {
		model::Entity::find().all(db).await.map_err(Error::DbError)
	}
}

pub mod payee {
	use sea_orm::{
		DatabaseConnection,
		EntityTrait,
		ActiveModelTrait,
		Set,
	};
	use crate::Error;
	use crate::entities::payee as model;
	
	pub async fn add(payee: model::Model, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let active_model = model::ActiveModel {
			name: Set(payee.name.clone()),
			..Default::default()
		};
		
		let result = active_model.insert(db).await.map_err(Error::DbError)?;

		Ok(model::Model {
			id: result.id.unwrap(),
			..payee
		})
	}

	pub async fn remove(id: i32, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let model = model::Entity::find_by_id(id).one(db).await.map_err(Error::DbError)?;
		let model = model.ok_or(Error::InternalError(format!("payee not found: {}", id).to_owned()))?;
		let active_model: model::ActiveModel = model.clone().into();
		let result = active_model.delete(db).await.map_err(Error::DbError)?;
		assert!(result.rows_affected == 1);
		
		Ok(model)
	}

	pub async fn list(db: &DatabaseConnection) -> Result<Vec<model::Model>, Error> {
		model::Entity::find().all(db).await.map_err(Error::DbError)
	}
}

pub mod transaction {
	use sea_orm::{
		DatabaseConnection,
		EntityTrait,
		ActiveModelTrait,
		Set,
		QueryFilter,
		ColumnTrait,
	};
	use crate::Error;
	use crate::entities::transaction as model;

	use chrono::Local;
	
	pub async fn add(tx: model::Model, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let active_model = model::ActiveModel {
			account_id: Set(tx.account_id),
			payee_id: Set(tx.payee_id),
			// transfer_account_id: Set(tx.transfer_account_id),
			category_id: Set(tx.category_id),
			memo: Set(tx.memo.clone()),
			amount: Set(tx.amount),
			timestamp: Set(Local::now().naive_local()),
			..Default::default()
		};
		
		let result = active_model.insert(db).await.map_err(Error::DbError)?;

		Ok(model::Model {
			id: result.id.unwrap(),
			..tx
		})
	}

	pub async fn remove(id: i32, db: &DatabaseConnection) -> Result<model::Model, Error> {
		let model = model::Entity::find_by_id(id).one(db).await.map_err(Error::DbError)?;
		let model = model.ok_or(Error::InternalError(format!("payee not found: {}", id).to_owned()))?;
		let active_model: model::ActiveModel = model.clone().into();
		let result = active_model.delete(db).await.map_err(Error::DbError)?;
		assert!(result.rows_affected == 1);
		
		Ok(model)
	}

	pub async fn list(account_id: Option<i32>, db: &DatabaseConnection) -> Result<Vec<model::Model>, Error> {
		let mut query = model::Entity::find();
		if let Some(account_id) = account_id {
			query = query.filter(model::Column::AccountId.eq(account_id));
		}
		query.all(db).await.map_err(Error::DbError)
	}
}