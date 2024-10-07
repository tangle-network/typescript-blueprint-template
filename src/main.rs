use color_eyre::{eyre::eyre, Result};
use gadget_sdk::keystore::sp_core_subxt::Pair;
use gadget_sdk::tangle_subxt::subxt::tx::Signer;
use gadget_sdk::{
    config::{ContextConfig, GadgetConfiguration, Protocol, StdGadgetConfiguration},
    event_listener::{EventListener, IntoTangleEventListener},
    events_watcher::tangle::TangleEventsWatcher,
    info,
    run::GadgetRunner,
    tangle_subxt::tangle_testnet_runtime::api::{
        self,
        runtime_types::{
            sp_core::ecdsa,
            tangle_primitives::services::{self, PriceTargets},
        },
    },
    tx,
};
use structopt::StructOpt;

pub use typescript_blueprint_template as blueprint;

#[tokio::main]
async fn main() -> Result<()> {
    gadget_sdk::logging::setup_log();
    // Load the environment and create the gadget runner
    let config = ContextConfig::from_args();

    let (env, mut runner) = create_gadget_runner(config.clone()).await;

    info!("~~~ Executing the incredible squaring blueprint ~~~");

    // Register the operator if needed
    if env.should_run_registration() {
        // Execute any custom registration hook
        runner.register().await?;
    }

    // Run the gadget / AVS
    runner.run().await?;

    Ok(())
}

struct TangleGadgetRunner {
    env: GadgetConfiguration<parking_lot::RawRwLock>,
}

#[async_trait::async_trait]
impl GadgetRunner for TangleGadgetRunner {
    type Error = color_eyre::eyre::Report;

    fn config(&self) -> &StdGadgetConfiguration {
        todo!()
    }

    async fn register(&mut self) -> Result<()> {
        // TODO: Use the function in blueprint-test-utils
        if self.env.test_mode {
            info!("Skipping registration in test mode");
            return Ok(());
        }

        let client = self.env.client().await.map_err(|e| eyre!(e))?;
        let signer = self
            .env
            .first_sr25519_signer()
            .map_err(|e| eyre!(e))
            .map_err(|e| eyre!(e))?;
        let ecdsa_pair = self.env.first_ecdsa_signer().map_err(|e| eyre!(e))?;

        let xt = api::tx().services().register(
            self.env.blueprint_id,
            services::OperatorPreferences {
                key: ecdsa::Public(ecdsa_pair.signer().public().0),
                approval: services::ApprovalPrefrence::None,
                price_targets: PriceTargets {
                    cpu: 0,
                    mem: 0,
                    storage_hdd: 0,
                    storage_ssd: 0,
                    storage_nvme: 0,
                },
            },
            Default::default(),
        );

        // send the tx to the tangle and exit.
        let result = tx::tangle::send(&client, &signer, &xt).await?;
        info!("Registered operator with hash: {:?}", result);
        Ok(())
    }

    async fn benchmark(&self) -> std::result::Result<(), Self::Error> {
        todo!()
    }

    async fn run(&self) -> Result<()> {
        let client = self.env.client().await.map_err(|e| eyre!(e))?;
        let signer = self.env.first_sr25519_signer().map_err(|e| eyre!(e))?;

        info!("Starting the event watcher for {} ...", signer.account_id());

        let program = TangleEventsWatcher {
            span: self.env.span.clone(),
            client,
            handlers: vec![
            ],
        };

        program.into_tangle_event_listener().execute().await;

        Ok(())
    }
}

async fn create_gadget_runner(
    config: ContextConfig,
) -> (
    GadgetConfiguration<parking_lot::RawRwLock>,
    Box<dyn GadgetRunner<Error = color_eyre::Report>>,
) {
    let env = gadget_sdk::config::load(config).expect("Failed to load environment");
    match env.protocol {
        Protocol::Tangle => (env.clone(), Box::new(TangleGadgetRunner { env })),
        _ => panic!("Unsupported protocol Eigenlayer. Gadget/Tangle need U256 support."),
    }
}