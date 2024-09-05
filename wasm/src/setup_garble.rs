use anyhow::Error as Anyhow;
use mpz_common::{Allocate, Context, Preprocess};
use mpz_garble::config::Role as DEAPRole;
use mpz_garble::protocol::deap::DEAPThread;
use mpz_ot::{
    chou_orlandi::{
        Receiver as BaseReceiver, ReceiverConfig as BaseReceiverConfig, Sender as BaseSender,
        SenderConfig as BaseSenderConfig,
    },
    kos::{Receiver, ReceiverConfig, Sender, SenderConfig},
    OTSetup,
};

pub enum Role {
    Alice,
    Bob,
}

/// Sets up a VM for garbled circuits.
///
/// # Arguments
///
/// * `role` - Set up the vm for either Alice or Bob.
/// * `context` - A context for IO.
/// * `ot_count` - How many OTs to set up.
pub async fn setup_garble<Ctx: Context>(
    role: Role,
    mut context: Ctx,
    ot_count: usize,
) -> Result<DEAPThread<Ctx, Sender<BaseReceiver>, Receiver<BaseSender>>, Anyhow> {
    // Create base OT sender and receiver.
    let base_sender_config = BaseSenderConfig::builder().build()?;
    let base_sender = BaseSender::new(base_sender_config);

    let base_receiver_config = BaseReceiverConfig::builder().build()?;
    let base_receiver = BaseReceiver::new(base_receiver_config);

    // Create OT sender and receiver and set them up.
    let sender_config = SenderConfig::builder().build()?;
    let mut sender = Sender::new(sender_config, base_receiver);

    let receiver_config = ReceiverConfig::builder().build()?;
    let mut receiver = Receiver::new(receiver_config, base_sender);

    let deap_role = match role {
        Role::Alice => DEAPRole::Leader,
        Role::Bob => DEAPRole::Follower,
    };

    sender.alloc(ot_count);
    receiver.alloc(ot_count);

    if let Role::Alice = role {
        sender.setup(&mut context).await?;
        sender.preprocess(&mut context).await?;
    } else {
        receiver.setup(&mut context).await?;
        receiver.preprocess(&mut context).await?;
    }

    if let Role::Bob = role {
        sender.setup(&mut context).await?;
        sender.preprocess(&mut context).await?;
    } else {
        receiver.setup(&mut context).await?;
        receiver.preprocess(&mut context).await?;
    }

    Ok(DEAPThread::new(
        deap_role, [0; 32], context, sender, receiver,
    ))
}
