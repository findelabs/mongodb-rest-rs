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
        let db = DB::init(&args.uri).await?;

        Ok(State { db })
    }
}
