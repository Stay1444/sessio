use std::{collections::HashMap, net::SocketAddr, time::Duration};

use crate::{common::{Packet, NewSession, PacketBase, ServerPacket, UpdateIp}, coordinator_client::CoordinatorClient};
use log::{error, info};

use quinn::{Connection, Endpoint};
use anyhow::Result;
use russh_keys::key::KeyPair;
use tokio::time;
use url::Url;
use uuid::Uuid;


pub struct HolepunchService{
    pub c_client: CoordinatorClient,
    pub endpoint: Endpoint,
    coordinator_url: Url,
    key_pair: KeyPair
}

impl HolepunchService {

    async fn connect(coordinator_url: &Url, endpoint: &Endpoint, key_pair: &KeyPair, ipv6: Option<SocketAddr>, 
        id_own: String) -> Result<(CoordinatorClient)> {
        let c_client = loop {
            match CoordinatorClient::connect(coordinator_url.clone(), id_own.clone(), endpoint.clone(),
                key_pair.clone(), ipv6).await {
                Ok(client) => break client,
                Err(e) => {
                    if let Some(_) = e.downcast_ref::<quinn::ConnectError>() {
                        error!("Failed to connect to coordination server {}\nRetrying in 5 seconds..", e);
                        time::sleep(Duration::from_secs(5)).await;
                    }
                    else {
                        error!("Failed to connect to coordination server: {}", e);
                        return Err(e.context("Possibly failed auth: check coord server logs"));
                    }
                }
            }
        };
        Ok(c_client)
    }

    ///Keypair is used to authenticate with the coordinator
    pub async fn new(coordinator_url: Url, endpoint: Endpoint, key_pair: KeyPair, ipv6: Option<SocketAddr>, 
    id_own: String) -> Result<Self> {
        
        let mut c_client = HolepunchService::connect(&coordinator_url, &endpoint, &key_pair, ipv6, id_own).await?;
        let mut service = HolepunchService {
            c_client,
            endpoint,
            coordinator_url,
            key_pair
        };
        service.start_connection_update_task();
        Ok(service)
    }

    pub async fn reconnect(&mut self) -> Result<()>{
        let ipv6 = CoordinatorClient::get_new_external_ipv6(self.endpoint.local_addr().unwrap().port()).await;
        self.c_client = HolepunchService::connect(&self.coordinator_url, &self.endpoint, &self.key_pair, ipv6, self.c_client.id_own.clone()).await?;
        Ok(())
    }

    pub async fn attempt_holepunch(&mut self, target: String) -> Result<Connection>{
        let c_client = &mut self.c_client;

        let mut receiver = c_client.subscribe_to_packets().await;

        let _ = c_client.send_server_packet(Packet::NewSession(NewSession {
            target_id: target
        })).await;

        let timeout_duration = Duration::from_secs(10);

        let timeout_future = tokio::time::sleep(timeout_duration);
        tokio::pin!(timeout_future);
        
        info!("attemtping hole pnch!");
        loop {
            
            tokio::select! {
                packet = receiver.recv() => {
                    let packet = match packet {
                        Ok(packet) => packet,
                        Err(e) => {
                            error!("Failed to receive packet: {}", e);
                            return Err(e.into());
                        }
                    };
                    match packet {
                        Packet::ConnectTo(data) => {
                            match self.endpoint.connect(data.target, "server").unwrap().await {
                                Ok(conn) => {
                                    return Ok(conn);
                                }
                                Err(e) => {
                                    info!("Connection failed: {}", e);
                                    return Err(e.into());
                                }
                            }
                        }
                        _ => {}
                    }
                },
                _ = &mut timeout_future => {
                    error!("Attempt to holepunch timed out after {:?}", timeout_duration);
                    return Err(anyhow::anyhow!("Holepunch attempt timed out"));
                } 
            }


        }
    }
    
    pub fn start_connection_update_task(&mut self){
        let mut update_interval = time::interval(Duration::from_secs(2));
        let bound_port = self.endpoint.local_addr().unwrap().port();

        let sender = self.c_client.new_packet_sender();

        let token = self.c_client.token.clone();
        let id = self.c_client.id_own.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = update_interval.tick() => {
                        let ipv6 = CoordinatorClient::get_new_external_ipv6(bound_port).await;
                        
                        let packet = ServerPacket {
                            base: Some(PacketBase {
                                own_id: id.clone(),
                                token: token.clone()
                            }),
                            packet: Packet::UpdateIp(UpdateIp {
                                ipv6
                            })
                        };

                        if let Err(e) = sender.send(packet).await {
                            log::warn!("Could not send update packet {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

}

