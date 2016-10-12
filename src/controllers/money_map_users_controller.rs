// src/controllers/money_map_users_controller.rs

/// Money Map Users Controller

// Import
// External
use ::nickel::{JsonBody, Request};
use ::bson::{Bson};
use ::bson::oid::ObjectId;
// Utilities
use ::common::api_result::ApiResult;
use ::common::config::Config;
use ::common::data_access::ServerData;
use ::common::mm_result::{MMResult, MMError, MMErrorKind};
use ::common::session as Session;
// DAO
use ::dao::dao_manager::DAOManager;
// Models
use ::models::user_model::{OutUserModel};
use ::models::money_map_model::{MoneyMapModel};
use ::models::money_map_user_model::{MoneyMapUserModel, InMoneyMapUserModel};

#[derive(Clone)]
pub struct MoneyMapUsersController{
    dao_manager: DAOManager,
    config: Config
}

impl MoneyMapUsersController{

    pub fn new(dao_manager: DAOManager, config: Config) -> MoneyMapUsersController{
        MoneyMapUsersController{
            dao_manager: dao_manager,
            config: config
        }
    }

    /// Retrieve list of users for a money map with following conditions
    ///     Money map information only available for money maps belonging to current user
    ///
    /// # Arguments
    /// &self
    /// req - nickel::Request
    ///
    /// # Returns
    /// `ApiResult<Vec<MoneyMapUserModel>>` - ApiResult including the list of money map users
    pub fn find(&self, req: &Request<ServerData>, mm_id: &str) -> ApiResult<Vec<MoneyMapUserModel>, ()>{

        let user_id = match Session::get_session_id(req){
            Ok(id) => id,
            Err(e) => {
                error!("{}",e.get_message().to_string());
                return ApiResult::Failure{msg:"Unable to retrieve session data."};
            }
        };

        match self.dao_manager.get_money_map_dao(){
            Ok(dao) => {

                match ObjectId::with_string(mm_id){
                    Ok(id) => {
                        //Get list of money maps for this user
                        let filter = doc!{
                            "_id" => id,
                            "users.user_id" => user_id,
                            "deleted" => {
                                "$ne" => true
                            }
                        };
                        match dao.find_one(Some(filter), None){
                            Some(mut money_map) => {

                                // Get list of user details for money map
                                match MoneyMapUsersController::get_users_for_mm(&self.dao_manager, &money_map){
                                    Ok(users_list) => {
                                        // Add the new list of user details to the money map
                                        money_map.set_users(Some(users_list));
                                    },
                                    Err(e) => {
                                        return ApiResult::Failure{msg:e.get_message()};
                                    }
                                }

                                // Return the list of money maps
                                match money_map.get_users(){
                                    Some(users) => ApiResult::Success{result:users},
                                    None => ApiResult::Failure{msg:"Unable to find user details for money map"}
                                }
                            },
                            None => {
                                ApiResult::Failure{msg:"Unable to find money map."}
                            }
                        }
                    },
                    Err(e) => {
                        error!("{}", e);
                        ApiResult::Failure{msg:"Failed to find money map. Invalid ID."}
                    }
                }
            },
            Err(e) => {
                error!("{}",e.get_message().to_string());
                ApiResult::Failure{msg:"Unable to interact with database"}
            }
        }
    }// end find

    /// Add User to Money Map with following conditions
    ///     Users can only be added to a money map by the money map owner
    ///
    /// # Arguments
    /// &self
    /// req - nickel::Request
    ///
    /// # Returns
    /// `ApiResult<Vec<MoneyMapUserModel>>` - ApiResult including the updated list of money map users
    pub fn add(&self, req: &mut Request<ServerData>, mm_id: String) -> ApiResult<Vec<MoneyMapUserModel>, InMoneyMapUserModel>{

        let user_id = match Session::get_session_id(req){
            Ok(id) => id,
            Err(e) => {
                error!("{}",e.get_message().to_string());
                return ApiResult::Failure{msg:"Unable to retrieve session data."};
            }
        };

        match self.dao_manager.get_money_map_dao(){
            Ok(dao) => {

                match req.json_as::<InMoneyMapUserModel>(){
                    Ok(mut money_map_user) => {

                        match ObjectId::with_string(&mm_id){
                            Ok(id) => {
                                //Get list of money maps for this user
                                let filter = doc!{
                                    "_id" => id,
                                    "users.user_id" => user_id,
                                    "deleted" => {
                                        "$ne" => true
                                    }
                                };
                                match dao.find_one(Some(filter), None){
                                    Some(mut money_map) => {

                                        // Validate
                                        let validation_result = money_map_user.validate(&self.dao_manager);
                                        if validation_result.is_valid(){

                                            ApiResult::Failure{msg:"SAVE IT"}
                                        }else{
                                            ApiResult::Invalid{validation:validation_result, request:money_map_user}
                                        }
                                    },
                                    None => {
                                        ApiResult::Failure{msg:"Unable to find money map."}
                                    }
                                }
                            },
                            Err(e) => {
                                error!("{}", e);
                                ApiResult::Failure{msg:"Failed to find money map. Invalid ID."}
                            }
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
    }// end add

    /// Delete a Money Map User
    /// Remove a user from money map using the following rules:
    ///     Only an owner can remove users from a money map.
    ///     Owner should not be allowed to be deleted.
    ///
    /// # Arguments
    /// &self
    /// id - String
    ///
    /// # Returns
    /// `ApiResult<Vec<MoneyMapUserModel>>` - ApiResult including the updated list of money map users
    pub fn delete(&self, mm_id: &str, user_id: &str) -> ApiResult<Vec<MoneyMapUserModel>, ()>{
        ApiResult::Failure{msg:"Delete needs to be implemented"}
    }// end delete

    /// Get a list of user details for money map
    ///
    /// # Arguments
    /// &self
    /// money_map - &MoneyMapModel
    ///
    /// # Returns
    /// `ApiResult<Vec<MoneyMapUserModel>>` - MMResult including the list of money map users
    pub fn get_users_for_mm(dao_manager: &DAOManager, money_map: &MoneyMapModel) -> MMResult<Vec<MoneyMapUserModel>>{
        // Initialze a list of user details for this money map
        let mut users_list = Vec::new();
        if let Some(mm_users) = money_map.get_users(){

            // For each user associated with this money map
            for mm_user in mm_users{
                match dao_manager.get_user_dao(){
                    Ok(user_dao) => {

                        // Fetch the user's details
                        let user_id = Bson::ObjectId(mm_user.user.unwrap().id.unwrap());
                        let found_user = user_dao.find_one(Some(doc!{
                            "_id" => user_id
                        }), None);
                        if let Some(user) = found_user{
                            // Add the user details to the list
                            users_list.push(
                                MoneyMapUserModel::new(OutUserModel::new(user), mm_user.owner)
                            );
                        }
                    },
                    Err(e) => {
                        error!("{}",e.get_message().to_string());
                        return Err(MMError::new("Unable to interact with database", MMErrorKind::Controller));
                    }
                }
            }
        }
        Ok(users_list)
    }// end get_users_for_mm
}
