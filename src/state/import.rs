use futures::future::*;
use tower_grpc::client;
use tower_grpc::generic::client::GrpcService;
use tower_grpc::{Request, Status};

use crate::proto::import::client::ImportService;
use crate::proto::import::{ImportIntent, ImportIntentResult};

pub struct ImportIndex<T> {
    client: ImportService<T>,
}

impl<T> ImportIndex<T> {
    fn import<R>(&mut self)
    where
        T: GrpcService<R>,
        client::unary::Once<ImportIntent>: client::Encodable<R>,
    {
        let intent = ImportIntent {
            id: "".to_string(),
            source: 0,
            dump_url: "".to_string(),
            reimport_ids: vec![],
            callback_url: "".to_string(),
        };

        self.client.start_import(Request::new(intent)).then(|r| {
            dbg!(r);
            Result::<(), Status>::Ok(())
        });
    }
}
