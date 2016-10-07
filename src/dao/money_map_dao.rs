// src/dao/money_map_dao.rs

/// Money Map DAO
/// Handle all database interaction for Money Map collection

// Import
extern crate mongodb;

// Import Modules
// Common Utilities
use ::bson::{Bson, Document};
use ::bson::oid::ObjectId;
use ::mongodb::coll::options::FindOptions;
use ::mongodb::db::ThreadedDatabase;
use ::common::mm_result::{MMResult, MMError, MMErrorKind};
// Models
use ::models::money_map_model::{MoneyMapModel, MoneyMapUserModel};
use ::models::user_model::{OutUserModel};

// Constants
static MONEY_MAP_COLLECTION: &'static str = "money_maps";

/// Money Map DAO
pub struct MoneyMapDAO{
    db: mongodb::db::Database
}

// Money Map DAO Methods
impl MoneyMapDAO{
    /// Create MoneyMapDAO
    ///
    /// # Arguments
    /// db - mongodb::db::Database Cloned database connection
    ///
    /// # Returns
    /// `MoneyMapDAO`
    pub fn new(db: mongodb::db::Database) -> MoneyMapDAO{
        MoneyMapDAO{
            db: db
        }
    }

    /// Find All Money Maps belonging to a user
    ///
    /// # Arguments
    /// self
    /// filter - Option<Document> The find filter
    ///
    /// # Returns
    /// `Vec<MoneyMapModel>`
    pub fn find(self, filter: Option<Document>) -> Vec<MoneyMapModel>{
        let coll = self.db.collection(MONEY_MAP_COLLECTION);
        let mut money_maps = Vec::new();

        let mut find_options = FindOptions::new();
        find_options.projection = Some(doc!{
            "deleted" => 0//exclude password
        });

        match coll.find(filter, Some(find_options)){
            Ok(cursor) => {
                for result in cursor {
                    if let Ok(item) = result {
                        let money_map = document_to_model(item);
                        money_maps.push(money_map);
                    }
                }
            },
            Err(e) => {
                error!("Find All money_maps failed: {}", e)
            }
        }
        money_maps
    }// end find

    /// Find One Money Map
    ///
    /// # Arguments
    /// self
    /// filter - Option<Document> The find filter
    /// options - Option<FindOptions> The find options
    ///
    /// # Returns
    /// `Option<MoneyMapModel>` Some MoneyMapModel if found, None otherwise
    pub fn find_one(&self, filter: Option<Document>, options: Option<FindOptions>) -> Option<MoneyMapModel>{
        let coll = self.db.collection(MONEY_MAP_COLLECTION);

        match coll.find_one(filter, options){
            Ok(result) => {
                if let Some(document) = result{
                    Some(document_to_model(document))
                }else{
                    None
                }
            },
            Err(e) => {
                error!("Find User failed: {}", e);
                None
            }
        }
    }// end find_one

    /// Create Money Map
    /// Save new money mmap to the money maps collection
    ///
    /// # Arguments
    /// self
    /// &money_map - models::money_map_model::MoneyMapModel The Money Map
    /// user_id - The user ID this money map belongs to
    ///
    /// # Returns
    /// `MMResult<()>`
    pub fn create(&self, money_map: &MoneyMapModel, user_id: String) -> MMResult<mongodb::coll::results::InsertOneResult>{
        let coll = self.db.collection(MONEY_MAP_COLLECTION);

        /*let doc = doc! {
            "name" => (match money_map.get_name(){Some(val) => val, None => "".to_string()}),
            "users" => (match money_map.get_users(){Some(val) => val, None =>)}),
            "deleted" => false
        };*/
        // Build the document
        let mut doc = doc!{ "deleted" => false};
        if let Some(name) = money_map.get_name(){
            doc.insert_bson("name".to_string(), Bson::String(name));
        }
        let mm_user = doc!{
            "user_id" => user_id,
            "owner" => true
        };
        doc.insert_bson("users".to_string(), Bson::Array(vec![ Bson::Document(mm_user) ]));

        // Insert document into `money_maps` collection
        match coll.insert_one(doc.clone(), None){
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("{}", e);
                Err(MMError::new("Failed to insert money_map", MMErrorKind::DAO))
            }
        }
    }// end create

    /// Update an existing money map
    ///
    /// # Arguments
    /// self
    /// &money_map - models::money_map_models::MoneyMapModel The money map
    ///
    /// # Returns
    /// `MMResult<MoneyMapModel>` The updated money map if successful, None otherwise
    pub fn update(&self, money_map: &MoneyMapModel) -> MMResult<MoneyMapModel>{
        let coll = self.db.collection(MONEY_MAP_COLLECTION);

        let filter = doc! {
            "_id" => ( money_map.get_id().unwrap() )
        };

        // Build `$set` document to update document
        let mut set_doc = doc!{};
        if let Some(name) = money_map.get_name(){
            set_doc.insert_bson("name".to_string(), Bson::String(name));
        }
        let update_doc = doc! {"$set" => set_doc};

        // Update the money map
        match coll.update_one(filter.clone(), update_doc.clone(), None){
            Ok(result) => {
                if result.acknowledged && result.matched_count > 0 {
                    Ok(self.find_one(Some(filter), None).unwrap())
                }else{
                    Err(MMError::new("Unable to save money map", MMErrorKind::DAO))
                }
            },
            Err(e) => {
                error!("{}", e);
                Err(MMError::new("Failed to update money map.", MMErrorKind::DAO))
            }
        }
    }// end update

    /// Delete a money map
    /// Only allow deleting a money map owned by the current user
    ///
    /// # Arguments
    /// self
    /// money_map_id - String User identifier
    ///
    /// # Returns
    /// `MMResult<()>`
    pub fn delete(self, money_map_id: &str) -> MMResult<mongodb::coll::results::UpdateResult>{
        let coll = self.db.collection(MONEY_MAP_COLLECTION);

        match ObjectId::with_string(money_map_id){
            Ok(id) => {
                //TODO: Add filter for user - only allow deleting a map owned by current user
                let filter = doc! {
                    "_id" => id
                };

                // Build `$set` document to update document
                let mut set_doc = doc!{};
                set_doc.insert_bson("deleted".to_string(), Bson::Boolean(true));
                let update_doc = doc! {"$set" => set_doc};

                // Soft delete money map
                match coll.update_one(filter.clone(), update_doc.clone(), None){
                    Ok(result) => Ok(result),
                    Err(e) => {
                        error!("{}", e);
                        Err(MMError::new("Failed to delete money map.", MMErrorKind::DAO))
                    }
                }
            },
            Err(e) => {
                error!("{}", e);
                Err(MMError::new("Failed to delete money map.", MMErrorKind::DAO))
            }
        }
    }// end delete
}

fn document_to_model(doc: Document) -> MoneyMapModel{
    MoneyMapModel{
        id: match doc.get("_id"){
            Some(obj_id) => match obj_id{ &Bson::ObjectId(ref id) => Some(id.clone()), _ => None},
            _ => None
        },
        name: match doc.get("name"){
            Some(&Bson::String(ref name)) => Some(name.clone()),
            _ => None
        },
        users: match doc.get("users"){
            Some(&Bson::Array(ref users)) => {
                let mut user_mods = Vec::new();
                for user in users{
                    if let &Bson::Document(ref mm_user_bson) = user{
                        if let Some(&Bson::String(ref user_id)) = mm_user_bson.get("user_id"){
                            let user = OutUserModel{
                                id: Some(ObjectId::with_string(user_id.as_str()).unwrap()),
                                first_name:None,
                                last_name:None,
                                email:None
                            };

                            let mut owner = false;
                            if let Some(&Bson::Boolean(is_owner)) = mm_user_bson.get("owner"){
                                owner = is_owner;
                            }

                            user_mods.push(MoneyMapUserModel{
                                user: Some(user),
                                owner: owner
                            });
                        }
                    }
                }
                Some(user_mods)
            },
            _ => None
        }
    }
}
