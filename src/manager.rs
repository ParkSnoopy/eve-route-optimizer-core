use crate::config;

use crate::cli;

use crate::system;

#[allow(unused)]
use crate::trace;
use crate::progress;

use system::{ SystemPair, CurrentShortest };

use futures::{ stream, StreamExt };
use std::sync::{ Arc, RwLock, RwLockReadGuard, RwLockWriteGuard };

use clap::Parser;



pub struct RouteOptimizeManager {
    request_client: reqwest::Client,

    cli_args: RwLock<cli::Args>,
    system_holder: RwLock<system::SystemHolder>,
    progress_holder: RwLock<progress::ProgressHolder>,
}

impl RouteOptimizeManager {
    fn _new_from_cli() -> Self {
        Self {
            request_client: reqwest::Client::builder()
                .cookie_store(true)
                .user_agent(config::USER_AGENT)
                .build()
                .unwrap(),

            cli_args: RwLock::new(cli::Args::parse()),
            system_holder: RwLock::new(system::SystemHolder::new()),
            progress_holder: RwLock::new(progress::ProgressHolder::new()),
        }
    }

    pub fn new(args: &str) -> Self {
        Self {
            request_client: reqwest::Client::builder()
                .cookie_store(true)
                .user_agent(config::USER_AGENT)
                .build()
                .unwrap(),

            cli_args: RwLock::new(cli::Args::parse_from(args.split_whitespace())),
            system_holder: RwLock::new(system::SystemHolder::new()),
            progress_holder: RwLock::new(progress::ProgressHolder::new()),
        }
    }
}

impl RouteOptimizeManager {
    pub fn client(&self) -> &reqwest::Client {
        &self.request_client
    }
    pub fn args(&self) -> RwLockReadGuard<cli::Args> {
        self.cli_args.read().unwrap()
    }
    pub fn system_holder_ref(&self) -> RwLockReadGuard<system::SystemHolder> {
        self.system_holder.read().unwrap()
    }
    pub fn system_holder_mut(&self) -> RwLockWriteGuard<system::SystemHolder> {
        self.system_holder.write().unwrap()
    }
    pub fn progress_holder_ref(&self) -> RwLockReadGuard<progress::ProgressHolder> {
        self.progress_holder.read().unwrap()
    }
    pub fn progress_holder_mut(&self) -> RwLockWriteGuard<progress::ProgressHolder> {
        self.progress_holder.write().unwrap()
    }

    pub async fn main(&self) -> color_eyre::Result<Arc<RwLock<CurrentShortest>>> {
        color_eyre::install()?;

        self.system_holder_mut().register_route(&self.args().route);
        self.system_holder_mut().register_system(&self.args().start);
        match &self.args().end {
            Some(system) => {
                self.system_holder_mut().register_system(&system);
            },
            _ => (),
        }

        let system_pairs: Vec<SystemPair> = self.system_holder_ref().all_inter_systems_iter().collect();

        trace::info("Fetching distances between system...");
        let bodies = stream::iter(system_pairs)
            .map(|system_pair| {
                let client = self.client();
                async move {
                    let url = self.make_url(&system_pair);
                    let resp = client
                        .get(url.clone())
                        .send()
                        .await
                        .expect(&trace::string::error(format!("Failed to fetch '{url}'")));
                    ( system_pair, resp.text().await )
                }
            })
            .buffer_unordered(self.args().concurrent);

        bodies
            .for_each(|(system_pair, resp_text_result)| async move {
                match resp_text_result {
                    Ok(resp_text) => {
                        let distance = self.parse_text_into_length(&resp_text);
                        system_pair.set_distance(distance).expect(&trace::string::error(format!("Failed to set distance between system {system_pair}")));
                    },
                    Err(e) => {
                        trace::error(format!("Error while processing request"));
                        trace::debug(e.to_string());
                    },
                }
            })
            .await;

        trace::ok("Fetch complete!");
        trace::info("Start to build Shortest Path...");

        let calculation_count: u128 = self.system_holder_ref().permutation_size_hint().unwrap_or(u128::MAX);
        self.progress_holder_mut().set_total(calculation_count);
        trace::info(format!("'{}' Calculation(s) to process", calculation_count));

        let feedback_step: usize = std::cmp::min(1_000_000, std::cmp::max(1, calculation_count/200) as usize);
        let current_shortest = self.system_holder_mut().build_shortest_path( self, feedback_step );

        trace::ok("shortest path build success!");
        Ok(current_shortest)
    }
}
