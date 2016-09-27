// src/common/routes.rs

/// MM Routes

// Import Modules
// External
use nickel::{HttpRouter, MediaType, Nickel, Router};
// Common Utilities
use ::common::api_result::{JsonEncoder};
use ::common::data_access::ServerData;

pub fn get_routes() -> Router<ServerData> {
    let mut router = Nickel::router();
    router.get("/", middleware! { |_request, mut response|
        info!("API Endpoint: /");
        response.set(MediaType::Json);
        format!(r#"{{"name":"Money Map", "version":"{}", "status":"ok"}}"#, env!("CARGO_PKG_VERSION"))
    });

    // Test Actions
    router.get("/test/retrieve", middleware! { |request, mut response|
        info!("API Endpoint: POST /test/retrieve");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.test_controller.retrieve();

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.post("/test/save", middleware! { |request, mut response|
        info!("API Endpoint: POST /test/save");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.test_controller.save(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.get("/test/failure", middleware! { |request, mut response|
        info!("API Endpoint: GET /test/failure");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.test_controller.failure();

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    // Users Actions
    router.get("/users", middleware! { |request, mut response|
        info!("API Endpoint: GET /users");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.users_controller.find_all();

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.get("/account", middleware! { |request, mut response|
        info!("API Endpoint: POST /account");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.users_controller.get_account(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.post("/account", middleware! { |request, mut response|
        info!("API Endpoint: POST /account");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.users_controller.create(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.patch("/account", middleware! { |request, mut response|
        info!("API Endpoint: PATCH /account");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.users_controller.modify(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    router.post("/account/login", middleware! { |request, mut response|
        info!("API Endpoint: POST /account/login");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.users_controller.login(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    // Money Maps
    router.post("/money_maps", middleware! { |request, mut response|
        info!("API Endpoint: POST /money_maps");
        let sd: &ServerData = request.server_data();
        let result = &sd.controller_manager.money_maps_controller.create(request);

        response.set(MediaType::Json);
        JsonEncoder::encode(result)
    });

    // Return router
    router
}
