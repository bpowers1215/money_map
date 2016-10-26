// src/controllers/accounts_controller.rs

/// Accounts Controller

// Import
// External
use ::nickel::{JsonBody, Request};
use ::bson::{Bson};
use ::bson::oid::ObjectId;
// Utilities
use ::common::api_result::ApiResult;
use ::common::config::Config;
use ::common::data_access::ServerData;
use ::common::session as Session;
// Models
use ::models::account_model::{AccountModel};
// DAO
use ::dao::dao_manager::DAOManager;

#[derive(Clone)]
pub struct AccountsController{
    dao_manager: DAOManager,
    config: Config
}

impl AccountsController{

    pub fn new(dao_manager: DAOManager, config: Config) -> AccountsController{
        AccountsController{
            dao_manager: dao_manager,
            config: config
        }
    }

    /// Create Account
    ///
    /// # Arguments
    /// &self
    /// req - nickel::Request
    /// mm_id - String
    ///
    /// # Returns
    /// `ApiResult<AccountModel, AccountModel>` - ApiResult including the created account
    pub fn create(&self, req: &mut Request<ServerData>, mm_id: String) -> ApiResult<AccountModel, AccountModel>{

        let user_id = match Session::get_session_id(req){
            Ok(id) => id,
            Err(e) => {
                error!("{}",e.get_message().to_string());
                return ApiResult::Failure{msg:"Unable to retrieve session data."};
            }
        };

        // START Retrieve DAO ---------------------------------------------------------------------
        match self.dao_manager.get_money_map_dao(){
            Ok(mm_dao) => {
                match self.dao_manager.get_account_dao(){
                    Ok(account_dao) => {
                        // END Retrieve DAO -------------------------------------------------------

                        match ObjectId::with_string(&mm_id){
                            Ok(mm_obj_id) => {
                                match ObjectId::with_string(&user_id){
                                    Ok(user_obj_id) => {

                                        // TODO Verify Money Map / User membership
                                        let filter = doc!{
                                            "_id" => (mm_obj_id.clone()),
                                            "users.user_id" => user_obj_id,
                                            "deleted" => {
                                                "$ne" => true
                                            }
                                        };
                                        match mm_dao.find_one(Some(filter), None){
                                            Some(money_map) => {

                                                // Parse body to AccountModel
                                                match req.json_as::<AccountModel>(){
                                                    Ok(mut account) => {

                                                        // Validate
                                                        let validation_result = account.validate();
                                                        if validation_result.is_valid(){
                                                            // Save Account
                                                            match account_dao.create(mm_obj_id, &account){
                                                                Ok(new_account) => {

                                                                    ApiResult::Success{result:new_account}
                                                                },
                                                                Err(e) => {
                                                                    error!("{}",e);
                                                                    ApiResult::Failure{msg:"Unable to create account"}
                                                                }
                                                            }
                                                        }else{
                                                            ApiResult::Invalid{validation:validation_result, request:account}
                                                        }

                                                    },
                                                    Err(e) => {
                                                        error!("{}",e);
                                                        ApiResult::Failure{msg:"Invalid format. Unable to parse data."}
                                                    }
                                                }
                                            },
                                            None => {
                                                ApiResult::Failure{msg:"Unable to find money map."}
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        error!("{}", e);
                                        ApiResult::Failure{msg:"Failed to find money map. Invalid user ID."}
                                    }
                                }
                            },
                            Err(e) => {
                                error!("{}", e);
                                ApiResult::Failure{msg:"Failed to find money map. Invalid ID."}
                            }
                        }

                        // START Retrieve DAO Error Handling --------------------------------------
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
        // END Retrieve DAO Error Handling --------------------------------------------------------
    }// end create

}
