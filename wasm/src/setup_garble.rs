use anyhow::Error as Anyhow;
use mpz_common::{Allocate, Context, Preprocess};
use mpz_garble::protocol::deap::DEAPThread;
use mpz_garble::{config::Role as DEAPRole, DecodePrivate, Execute, Memory};
use mpz_ot::{
    chou_orlandi::{
        Receiver as BaseReceiver, ReceiverConfig as BaseReceiverConfig, Sender as BaseSender,
        SenderConfig as BaseSenderConfig,
    },
    kos::{Receiver, ReceiverConfig, Sender, SenderConfig},
    OTSetup,
};
use web_sys::console;

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
pub async fn setup_garble(
    role: Role,
    mut context: impl Context,
    ot_count: usize,
) -> Result<impl Memory + Execute + DecodePrivate, Anyhow> {
    // Create base OT sender and receiver.
    let base_sender_config = BaseSenderConfig::builder().build()?;
    let base_sender = BaseSender::new(base_sender_config);

    console::log_1(&"test2.1".into());

    let base_receiver_config = BaseReceiverConfig::builder().build()?;
    let base_receiver = BaseReceiver::new(base_receiver_config);

    console::log_1(&"test2.2".into());

    // Create OT sender and receiver and set them up.
    let sender_config = SenderConfig::builder().build()?;
    let mut sender = Sender::new(sender_config, base_receiver);

    console::log_1(&"test2.3".into());

    let receiver_config = ReceiverConfig::builder().build()?;
    let mut receiver = Receiver::new(receiver_config, base_sender);

    console::log_1(&"test2.4".into());

    let deap_role = match role {
        Role::Alice => DEAPRole::Leader,
        Role::Bob => DEAPRole::Follower,
    };

    sender.alloc(ot_count);
    receiver.alloc(ot_count);

    console::log_1(&"test2.5".into());

    if let Role::Alice = role {
        console::log_1(&"blarg_a".into());
        sender.setup(&mut context).await?;
        console::log_1(&"test2.5.1a".into());
        sender.preprocess(&mut context).await?;
    } else {
        console::log_1(&"blarg_b".into());
        receiver.setup(&mut context).await?;
        console::log_1(&"test2.5.1b".into());
        receiver.preprocess(&mut context).await?;
    }

    console::log_1(&"test2.6".into());

    if let Role::Bob = role {
        sender.setup(&mut context).await?;
        sender.preprocess(&mut context).await?;
    } else {
        receiver.setup(&mut context).await?;
        receiver.preprocess(&mut context).await?;
    }

    console::log_1(&"test2.7".into());

    Ok(DEAPThread::new(
        deap_role, [0; 32], context, sender, receiver,
    ))
}
