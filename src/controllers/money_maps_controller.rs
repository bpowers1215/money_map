// src/controllers/money_maps_controller.rs

/// Money Maps Controller

// Import
// External
use ::chrono::{DateTime, Duration, Local};
use ::nickel::{JsonBody, Request};
use ::bson::{Bson, Document};
use ::bson::oid::ObjectId;
use ::std::default::Default;
use ::crypto::sha2::Sha256;
use ::jwt::{Header, Registered, Token};
use ::rustc_serialize::hex::ToHex;
// Utilities
use ::common::api_result::ApiResult;
use ::common::config::Config;
use ::common::data_access::ServerData;
use ::common::session as Session;
// DAO
use ::dao::dao_manager::DAOManager;
// Models
use ::models::user_model::{OutUserModel};
use ::models::money_map_model::{MoneyMapModel};

#[derive(Clone)]
pub struct MoneyMapsController{
    dao_manager: DAOManager,
    config: Config
}

impl MoneyMapsController{

    pub fn new(dao_manager: DAOManager, config: Config) -> MoneyMapsController{
        MoneyMapsController{
            dao_manager: dao_manager,
            config: config
        }
    }

    /// Find All Money Maps belonging to the requesting user
    ///
    /// # Arguments
    /// &self
    /// req - nickel::Request
    ///
    /// # Returns
    /// `ApiResult<Vec<MoneyMapModel>>` - ApiResult including a vector of money maps
    pub fn find(&self, req: &mut Request<ServerData>) -> ApiResult<Vec<MoneyMapModel>>{

        let user_id = match Session::get_session_id(req){
            Ok(id) => id,
            Err(e) => {
                error!("{}",e.get_message().to_string());
                return ApiResult::Failure{msg:"Unable to retrieve session data."};
            }
        };

        match self.dao_manager.get_money_map_dao(){
            Ok(dao) => {
                match self.dao_manager.get_user_dao(){
                    Ok(user_dao) => {
                        //Get list of money maps for this user
                        let mut money_maps = dao.find(Some(doc!{
                            "users" => user_id,
                            "deleted" => {
                                "$ne" => true
                            }
                        }));

                        // Get list of user details for each money map
                        for i in 0..money_maps.len(){
                            if let Some(users) = money_maps[i].get_users(){
                                let mut users_list = Vec::new();
                                for user in users{
                                    if let Some(id) = user.id{
                                        users_list.push(Bson::ObjectId(id));
                                    }
                                }
                                let users = &user_dao.find(Some(doc!{
                                    "_id" => {
                                        "$in" => users_list
                                    }
                                }));
                                money_maps[i].set_users(Some(users.clone()));
                            }
                        }

                        ApiResult::Success{result:money_maps}
                    },
                    Err(e) => {
                        error!("{}",e.get_message().to_string());
                        ApiResult::Failure{msg:"Unable to interact with database"}
                    }
                }
            },
            Err(e) => {
                error!("{}",e.get_message().to_string());
                ApiResult::Failure{msg:"Unable to interact with database"}
            }
        }
    }// end find_all

    /// Create Money Map
    ///
    /// # Arguments
    /// &self
    /// req - nickel::Request
    ///
    /// # Returns
    /// `ApiResult<MoneyMapModel>` - ApiResult including the create money map
    pub fn create(&self, req: &mut Request<ServerData>) -> ApiResult<MoneyMapModel>{

        let user_id = match Session::get_session_id(req){
            Ok(id) => id,
            Err(e) => {
                error!("{}",e.get_message().to_string());
                return ApiResult::Failure{msg:"Unable to retrieve session data."};
            }
        };

        match self.dao_manager.get_money_map_dao(){
            Ok(dao) => {
                match self.dao_manager.get_user_dao(){
                    Ok(user_dao) => {

                        match req.json_as::<MoneyMapModel>(){
                            Ok(mut money_map) => {
                                // Validate
                                let validation_result = money_map.validate();
                                if validation_result.is_valid(){
                                    // Save User
                                    match dao.create(&money_map, user_id.clone()){
                                        Ok(result) => {
                                            // Set user ID
                                            match result.inserted_id{
                                                Some(id_wrapper) => {
                                                    match id_wrapper{
                                                        Bson::ObjectId(id) => money_map.set_id(id),
                                                        _ => {}
                                                    }
                                                },
                                                None => {}
                                            }
                                            // Add user details
                                            if let Ok(id) = ObjectId::with_string(user_id.as_str()){
                                                if let Some(user) = user_dao.find_one(Some(doc!{"_id" => id}), None){
                                                    money_map.set_users(Some(vec![OutUserModel::new(user)]));
                                                }
                                            }

                                            ApiResult::Success{result:money_map}
                                        },
                                        Err(e) => {
                                            error!("{}",e);
                                            ApiResult::Failure{msg:"Unable to create money map"}
                                        }
                                    }
                                }else{
                                    ApiResult::Invalid{validation:validation_result, request:money_map}
                                }
                            },
                            Err(e) => {
                                error!("{}",e);
                                ApiResult::Failure{msg:"Invalid format. Unable to parse data."}
                            }
                        }
                    },
                    Err(e) => {
                        error!("{}",e.get_message().to_string());
                        ApiResult::Failure{msg:"Unable to interact with database"}
                    }
                }
            },
            Err(e) => {
                error!("{}",e.get_message().to_string());
                ApiResult::Failure{msg:"Unable to interact with database"}
            }
        }
    }// end create

    /// Delete a Money Map
    ///
    /// # Arguments
    /// &self
    /// id - String
    ///
    /// # Returns
    /// `ApiResult<String>` - ApiResult
    pub fn delete(&self, id: &str) -> ApiResult<String>{
        match self.dao_manager.get_money_map_dao(){
            Ok(dao) => {
                match dao.delete(id){
                    Ok(result) => {
                        if result.acknowledged && result.modified_count > 0 {
                            ApiResult::Success{result:"Successfully deleted money map".to_string()}
                        }else{
                            ApiResult::Failure{msg:"Unable to delete money map"}
                        }
                    },
                    Err(e) => {
                        error!("{}",e);
                        ApiResult::Failure{msg:"Malformed ID"}
                    }
                }
            },
            Err(e) => {
                error!("{}",e.get_message().to_string());
                ApiResult::Failure{msg:"Unable to interact with database"}
            }
        }
    }// end delete

}
