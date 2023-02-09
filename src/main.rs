use axum::Json;
use axum::{extract::Path, routing::get, Router};
use dotenv;
use propelauth::axum::PropelAuthLayer;
use propelauth::propelauth::auth::PropelAuth;
use propelauth::propelauth::errors::UnauthorizedOrForbiddenError;
use propelauth::propelauth::options::{AuthOptions, RequiredOrg, UserRequirementsInOrg};
use propelauth::propelauth::token_models::{OrgMemberInfo, User};

#[tokio::main]
async fn main() {
    // Load variables from .env
    dotenv::dotenv().ok();
    let auth_url = std::env::var("PROPELAUTH_AUTH_URL")
        .expect("Couldn't find env variable PROPELAUTH_AUTH_URL");
    let api_key =
        std::env::var("PROPELAUTH_API_KEY").expect("Couldn't find env variable PROPELAUTH_API_KEY");

    // Initialize our crate. This performs a one time fetch to get the public key, which
    //   the library uses to validate tokens
    let auth = PropelAuth::fetch_and_init(AuthOptions { auth_url, api_key })
        .await
        .expect("Unable to initialize authentication");

    let auth_layer = PropelAuthLayer::new(auth);

    let app = Router::new()
        .route("/whoami", get(whoami))
        .route("/org/:org_id", get(org_whoami))
        .layer(auth_layer);

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// User will automatically return a 401 (Unauthorized) if a valid access token wasn't provided
async fn whoami(user: User) -> Json<User> {
    Json(user)
}

// If the user isn't in the provided organization, a 403 is returned
async fn org_whoami(
    user: User,
    Path(org_id): Path<String>,
) -> Result<Json<OrgMemberInfo>, UnauthorizedOrForbiddenError> {
    let org =
        user.validate_org_membership(RequiredOrg::OrgId(&org_id), UserRequirementsInOrg::None)?;
    Ok(Json(org))
}
