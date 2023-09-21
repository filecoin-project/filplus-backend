use dotenv;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};

pub async fn db_health_check(client: Client) -> mongodb::error::Result<()> {
    // Ping the server to see if you can connect to the cluster
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    Ok(())
}

pub async fn setup() -> mongodb::error::Result<Client> {
    let key = "MONGODB_URL";
    let value = dotenv::var(key).expect("Expected a MONGODB_URL in the environment");
    let mut client_options = ClientOptions::parse(value).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    Ok(client)
}
