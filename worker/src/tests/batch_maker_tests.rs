// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crypto::ed25519::Ed25519PublicKey;
use test_utils::transaction;
use tokio::sync::mpsc::channel;

#[tokio::test]
async fn make_batch() {
    let (tx_transaction, rx_transaction) = channel(1);
    let (tx_message, mut rx_message) = channel(1);
    let dummy_addresses = vec![(
        Ed25519PublicKey::default(),
        "/ip4/127.0.0.1/tcp/0/http".parse().unwrap(),
    )];

    // Spawn a `BatchMaker` instance.
    BatchMaker::spawn(
        /* max_batch_size */ 200,
        /* max_batch_delay */
        Duration::from_millis(1_000_000), // Ensure the timer is not triggered.
        rx_transaction,
        tx_message,
        /* workers_addresses */ dummy_addresses,
    );

    // Send enough transactions to seal a batch.
    let tx = transaction();
    tx_transaction.send(tx.clone()).await.unwrap();
    tx_transaction.send(tx.clone()).await.unwrap();

    // Ensure the batch is as expected.
    let expected_batch = Batch(vec![tx.clone(), tx.clone()]);
    let QuorumWaiterMessage { batch, handlers: _ } = rx_message.recv().await.unwrap();
    match bincode::deserialize(&batch).unwrap() {
        WorkerMessage::<Ed25519PublicKey>::Batch(batch) => assert_eq!(batch, expected_batch),
        _ => panic!("Unexpected message"),
    }
}

#[tokio::test]
async fn batch_timeout() {
    let (tx_transaction, rx_transaction) = channel(1);
    let (tx_message, mut rx_message) = channel(1);
    let dummy_addresses = vec![(
        Ed25519PublicKey::default(),
        "/ip4/127.0.0.1/tcp/0/http".parse().unwrap(),
    )];

    // Spawn a `BatchMaker` instance.
    BatchMaker::spawn(
        /* max_batch_size */ 200,
        /* max_batch_delay */
        Duration::from_millis(50), // Ensure the timer is triggered.
        rx_transaction,
        tx_message,
        /* workers_addresses */ dummy_addresses,
    );

    // Do not send enough transactions to seal a batch..
    let tx = transaction();
    tx_transaction.send(tx.clone()).await.unwrap();

    // Ensure the batch is as expected.
    let expected_batch = Batch(vec![tx]);
    let QuorumWaiterMessage { batch, handlers: _ } = rx_message.recv().await.unwrap();
    match bincode::deserialize(&batch).unwrap() {
        WorkerMessage::<Ed25519PublicKey>::Batch(batch) => assert_eq!(batch, expected_batch),
        _ => panic!("Unexpected message"),
    }
}
