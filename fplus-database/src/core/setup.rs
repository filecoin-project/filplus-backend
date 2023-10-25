use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion, Tls, TlsOptions},
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
    let value = std::env::var(key).expect("Expected a MONGODB_URL in the environment");
    let mut client_options = ClientOptions::parse(value).await?;

	let mut tls_options = TlsOptions::default();
	tls_options.allow_invalid_hostnames = Some(true);
	tls_options.allow_invalid_certificates = Some(true);
	client_options.tls = Some(Tls::Enabled(tls_options));

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    Ok(client)
}
