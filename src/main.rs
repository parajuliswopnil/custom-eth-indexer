use std::{
    str::FromStr, time::{Duration, SystemTime, UNIX_EPOCH}
};

use alloy_eips::eip7685::RequestsOrHash;
use alloy_primitives::{Address, FixedBytes, hex::FromHex};
use alloy_rpc_types_engine::{
    ExecutionPayloadEnvelopeV4,
    ExecutionPayloadV3, ForkchoiceState,
    ForkchoiceUpdated, PayloadAttributes, PayloadId, PayloadStatus, CAPABILITIES,
};
use http::HeaderMap;
use http::header::AUTHORIZATION;
use jsonrpsee::
    http_client::{HttpClient, HttpClientBuilder}
;

use reth_ethereum_engine_primitives::EthEngineTypes;
use reth_rpc_api::clients::EngineApiClient;
use reth_rpc_layer::{secret_to_bearer_header, JwtSecret};
use tokio::time::sleep;

/// Twine Batch Client
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct CustomClient {
    inner: HttpClient,
}

impl CustomClient {
    fn new(url: String) -> Self {
        let jwt_hex = std::fs::read_to_string("jwt.hex").unwrap();
        let secret = JwtSecret::from_str(jwt_hex.trim()).unwrap();
        let secret = secret_to_bearer_header(&secret);

        // jsonrpsee 0.25.1 builder
        let rpc_client = HttpClientBuilder::default()
            .set_headers({
                let mut map = HeaderMap::new();
                map.insert(AUTHORIZATION, secret);
                map
            })
            .build(url)
            .unwrap();

        Self { inner: rpc_client }
    }

    pub async fn exchange_capabilities(&self) {
        let capabilities = CAPABILITIES.into_iter().map(|c| c.to_string()).collect();
        let result = <HttpClient as EngineApiClient<EthEngineTypes>>::exchange_capabilities(
            &self.inner,
            capabilities,
        )
        .await;
        println!("{:?}", result);
    }

    pub async fn fork_choice_updated_v3(
        &self,
        fork_choice_state: ForkchoiceState,
        payload_attributes: Option<PayloadAttributes>,
    ) -> Result<ForkchoiceUpdated, String> {
        let result = <HttpClient as EngineApiClient<EthEngineTypes>>::fork_choice_updated_v3(
            &self.inner,
            fork_choice_state,
            payload_attributes.into(),
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub async fn new_payload_v4(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<FixedBytes<32>>,
        parent_beacon_block_root: FixedBytes<32>,
        execution_requests: RequestsOrHash, 
    ) -> Result<PayloadStatus, String> {
        let result = <HttpClient as EngineApiClient<EthEngineTypes>>::new_payload_v4(
            &self.inner,
            payload,
            versioned_hashes,
            parent_beacon_block_root,
            execution_requests, 
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub async fn get_payload_v4(
        &self,
        payload_id: PayloadId,
    ) -> Result<ExecutionPayloadEnvelopeV4, String> {
        let result = <HttpClient as EngineApiClient<EthEngineTypes>>::get_payload_v4(
            &self.inner,
            payload_id,
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[tokio::main]
async fn main() {
    let client = CustomClient::new("http://127.0.0.1:8551".into());

    client.exchange_capabilities().await;

    let mut first_block = FixedBytes::<32>::from_hex(
        "0x3312786df0691c689c73025d1657ffa32102f776e2bdbdac7e6c509e63a3b547",
    )
    .unwrap();

    let mut genesis_block = false;

    loop {
       
        let client = CustomClient::new("http://127.0.0.1:8551".into());
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let fork_choice_state = ForkchoiceState {
            head_block_hash: first_block,
            safe_block_hash: first_block,
            finalized_block_hash: first_block,
        };
        let payload_attributes = if genesis_block {
            genesis_block = false;
            None
        } else {
            Some(PayloadAttributes {
                timestamp: time_now + 2,
                prev_randao: [0u8; 32].into(),
                suggested_fee_recipient: Address::from_hex(
                    "0x0000000000000000000000000000000000000000",
                )
                .unwrap(),
                withdrawals: vec![].into(),
                parent_beacon_block_root: Some([0u8; 32].into()),
            })
        };

        let forkchoice = client
            .fork_choice_updated_v3(fork_choice_state, payload_attributes)
            .await
            .unwrap();

        println!("{:#?}", forkchoice);

        sleep(Duration::from_micros(500)).await;

        let built = client
            .get_payload_v4(forkchoice.payload_id.unwrap())
            .await
            .unwrap();

        let ex_payload = built.execution_payload.clone();
        let new_head = built
            .execution_payload
            .payload_inner
            .payload_inner
            .block_hash;

        let status = client
            .new_payload_v4(
                ex_payload,
                vec![],
                [0u8; 32].into(), 
                RequestsOrHash::Requests(built.execution_requests)
            )
            .await;

        println!("status is {:#?}", status);

        assert!(status.unwrap().is_valid(), "EL did not accept new payload");

        let fcu2 = ForkchoiceState {
            head_block_hash: new_head,
            safe_block_hash: new_head,
            finalized_block_hash: new_head,
        };

        let fork_choice = client.fork_choice_updated_v3(fcu2, None).await.unwrap();

        first_block = new_head;
        sleep(Duration::from_secs(2)).await;
    }
}
