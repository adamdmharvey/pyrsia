/*
   Copyright 2021 JFrog Ltd

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use crate::network::artifact_protocol::{ArtifactExchangeCodec, ArtifactRequest, ArtifactResponse};
use crate::network::idle_metric_protocol::{
    IdleMetricExchangeCodec, IdleMetricRequest, IdleMetricResponse,
};

use libp2p::dcutr;
use libp2p::identify::{Identify, IdentifyEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::relay::v2::client::{self, Client};
use libp2p::request_response::{RequestResponse, RequestResponseEvent};
use libp2p::NetworkBehaviour;

/// Defines the [`NetworkBehaviour`] to be used in the libp2p
/// Swarm. The PyrsiaNetworkBehaviour consists of the following
/// behaviours:
///
/// * [`Identify`]
/// * [`Kademlia`]
/// * [`RequestResponse`] for exchanging artifacts
/// * [`relay_client`]
/// * [`dcutr`] for direct connecrion upgrade through relay (hole punching)
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "PyrsiaNetworkEvent")]
pub struct PyrsiaNetworkBehaviour {
    pub identify: Identify,
    pub kademlia: Kademlia<MemoryStore>,
    pub request_response: RequestResponse<ArtifactExchangeCodec>,
    pub idle_metric_request_response: RequestResponse<IdleMetricExchangeCodec>,
    pub relay_client: Client,
    pub dcutr: dcutr::behaviour::Behaviour,
}

/// Each event in the `PyrsiaNetworkBehaviour` is wrapped in a
/// `PyrsiaNetworkEvent`.
#[derive(Debug)]
pub enum PyrsiaNetworkEvent {
    Identify(IdentifyEvent),
    Kademlia(KademliaEvent),
    RequestResponse(RequestResponseEvent<ArtifactRequest, ArtifactResponse>),
    IdleMetricRequestResponse(RequestResponseEvent<IdleMetricRequest, IdleMetricResponse>),
    Relay(client::Event),
    Dcutr(dcutr::behaviour::Event),
}

impl From<IdentifyEvent> for PyrsiaNetworkEvent {
    fn from(event: IdentifyEvent) -> Self {
        PyrsiaNetworkEvent::Identify(event)
    }
}

impl From<KademliaEvent> for PyrsiaNetworkEvent {
    fn from(event: KademliaEvent) -> Self {
        PyrsiaNetworkEvent::Kademlia(event)
    }
}

impl From<RequestResponseEvent<ArtifactRequest, ArtifactResponse>> for PyrsiaNetworkEvent {
    fn from(event: RequestResponseEvent<ArtifactRequest, ArtifactResponse>) -> Self {
        PyrsiaNetworkEvent::RequestResponse(event)
    }
}

impl From<RequestResponseEvent<IdleMetricRequest, IdleMetricResponse>> for PyrsiaNetworkEvent {
    fn from(event: RequestResponseEvent<IdleMetricRequest, IdleMetricResponse>) -> Self {
        PyrsiaNetworkEvent::IdleMetricRequestResponse(event)
    }
}

impl From<client::Event> for PyrsiaNetworkEvent {
    fn from(e: client::Event) -> Self {
        PyrsiaNetworkEvent::Relay(e)
    }
}

impl From<dcutr::behaviour::Event> for PyrsiaNetworkEvent {
    fn from(e: dcutr::behaviour::Event) -> Self {
        PyrsiaNetworkEvent::Dcutr(e)
    }
}
