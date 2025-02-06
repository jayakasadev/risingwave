// Copyright 2025 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use risingwave_common::telemetry::telemetry_cluster_type_from_env_var;
use risingwave_meta::controller::MetaStore;
use risingwave_meta_model::prelude::Cluster;
use risingwave_pb::meta::telemetry_info_service_server::TelemetryInfoService;
use risingwave_pb::meta::{GetTelemetryInfoRequest, TelemetryInfoResponse};
use sea_orm::{DatabaseConnection, EntityTrait};
use tonic::{Request, Response, Status};
use mongodb::Client;
use mongodb::bson::doc;
use crate::model::ClusterId;
use crate::MetaResult;

pub struct TelemetryInfoServiceImpl<T> {
    meta_store_impl: MetaStore<T>,
}

impl <T> TelemetryInfoServiceImpl<T> {
    pub fn new(meta_store_impl: MetaStore<T>) -> Self {
        Self { meta_store_impl }
    }
}

impl TelemetryInfoServiceImpl<DatabaseConnection> {
    async fn get_tracking_id(&self) -> MetaResult<Option<ClusterId>> {
        let cluster_id = Cluster::find()
            .one(&self.meta_store_impl.conn)
            .await?
            .map(|c| c.cluster_id.to_string().into());
        Ok(cluster_id)
    }
}

impl TelemetryInfoServiceImpl<Client> {
    async fn get_tracking_id(&self) -> MetaResult<Option<ClusterId>> {
        let cluster_id = self.meta_store_impl.conn
            .default_database()
            .unwrap()
            .collection("cluster")
            .find_one(doc!{})
            .await?
            .map(|c| c.get_str("_id").unwrap().to_string().into());
        Ok(cluster_id)
    }
}

#[async_trait::async_trait]
impl <T> TelemetryInfoService for TelemetryInfoServiceImpl<T> {
    async fn get_telemetry_info(
        &self,
        _request: Request<GetTelemetryInfoRequest>,
    ) -> Result<Response<TelemetryInfoResponse>, Status> {
        if telemetry_cluster_type_from_env_var().is_err() {
            return Ok(Response::new(TelemetryInfoResponse { tracking_id: None }));
        }
        match self.get_tracking_id().await? {
            Some(tracking_id) => Ok(Response::new(TelemetryInfoResponse {
                tracking_id: Some(tracking_id.into()),
            })),
            None => Ok(Response::new(TelemetryInfoResponse { tracking_id: None })),
        }
    }
}
