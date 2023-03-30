use mongodb::options::ClientOptions;
use mongodb::options::Credential;
use std::error::Error;

use crate::db::DB;
use crate::Args;
//use crate::error::Error as RestError;

type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Debug)]
pub struct State {
    pub db: DB,
}

impl State {
    pub async fn new(args: Args) -> BoxResult<Self> {
        let client = match args.username {
            Some(user) => {
                let cred = Credential::builder()
                    .username(Some(user))
                    .source(Some("admin".to_string()))
                    .password(args.password)
                    .build();

                let mut client_options = ClientOptions::parse(&args.uri).await?;
                client_options.credential = Some(cred);
                client_options
            }
            None => ClientOptions::parse(&args.uri).await?,
        };

        let db = DB::init(client).await?;

        Ok(State { db })
    }
}
