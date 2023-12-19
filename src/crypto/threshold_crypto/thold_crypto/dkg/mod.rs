//#[cfg(feature = "serialize_serde")]
//mod other;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use thiserror::Error;
use threshold_crypto::{Fr, G1Affine};
#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};
use threshold_crypto::ff::Field;
use threshold_crypto::group::CurveAffine;
use threshold_crypto::poly::{BivarCommitment, BivarPoly, Poly};
use threshold_crypto::serde_impl::FieldWrap;
use super::PrivateKeyPart;
use super::PublicKeySet;
use crate::Err;
use crate::error::*;

/// The parameters for a distributed key generation algorithm.
#[derive(Getters, CopyGetters)]
pub struct DKGParams {
    // Get the amount of dealers in the system
    #[getset(get_copy)]
    dealers: usize,
    // Get the amount of faulty nodes to be tolerated <= dealers / 2
    #[getset(get_copy)]
    faulty_nodes: usize,
}

/// The parameters sent by a dealer
/// The BiVar Commitment is the "public key" from the bivar polynomial generated by each of the dealers
/// The vector of polynomials are the private key shares, generated from the bivar polynomial
///
/// These messages must be delivered in order to the other nodes
#[derive(Clone)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct DealerPart {
    // The author of the Dealer Part
    author: usize,
    commitment: BivarCommitment,
    share_values: Vec<Poly>,
}

/// Acknowledgement sent by a node after receiving a dealer part
/// Contains the ID of the dealer part along with a vector with all of the
/// commitment values
///
/// Similarly to [DealerPart], this must be delivered in order to the other nodes
#[derive(Clone)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct Ack {
    author: usize,
    part_being_acked: usize,
    commitments: Vec<Fr>,
}

/// The parameter sent by a dealer to a specific node.
/// This is derived from [DealerPart] and is meant to be delivered to a specific node.
#[derive(Clone)]
pub struct DirectedDealerPart(BivarCommitment, Poly);

/// A directed acknowledgement sent by a dealer to a specific node.
/// This is derived from [Ack] and is meant to be delivered to a specific node.
/// Analogous to [DirectedDealerPart]
#[derive(Clone)]
pub struct DirectedAck(usize, Fr);

/// A complaint, sent by a given node about another node, which is suspected to be faulty
#[derive(Clone)]
pub struct Complaint(usize);

/// The state of a given node in the distributed key generation algorithm
#[derive(Getters, MutGetters, Setters)]
pub struct NodeState {
    #[getset(get_copy)]
    id: usize,
    // The dealer commitment that we have received
    #[getset(get)]
    commit: BivarCommitment,
    // Verified values received from Ack
    #[getset(get, get_mut)]
    values: BTreeMap<usize, Fr>,
    // Nodes that have already ACKed the part sent by this node
    #[getset(get, get_mut)]
    acks: BTreeSet<usize>,
}

/// The distributed key generation algorithm for
/// a threshold cryptographic scheme.
#[derive(CopyGetters, Getters, MutGetters)]
pub struct DistributedKeyGenerator {
    #[getset(get)]
    params: DKGParams,
    #[getset(get_copy = "pub")]
    our_id: usize,
    #[getset(get)]
    own_generator: BivarPoly,
    received_parts: BTreeMap<usize, NodeState>,
    #[getset(get, get_mut)]
    pending_acks: BTreeMap<usize, VecDeque<(usize, Ack)>>,
    #[getset(get_copy = "pub")]
    complete: usize,
}

impl DistributedKeyGenerator {
    pub fn new(params: DKGParams, our_id: usize) -> Result<(Self, DealerPart)> {
        let mut rng = rand::rngs::OsRng::new()?;
        let my_gen = BivarPoly::random(params.faulty_nodes(), &mut rng);

        let my_part = DealerPart {
            author: our_id,
            commitment: my_gen.commitment(),
            share_values: (1..=params.dealers())
                .map(|i| my_gen.row(i))
                .collect(),
        };

        let generator = Self {
            params,
            our_id,
            own_generator: my_gen,
            received_parts: Default::default(),
            pending_acks: Default::default(),
            complete: 0,
        };

        Ok((generator, my_part))
    }

    pub fn is_ready(&self) -> bool {
        self.complete() > self.params.faulty_nodes()
    }

    /// Handles a dealer part received from another dealer.
    /// What goes on here is basically step 1 of the DKG algorithm.
    pub fn handle_part(&mut self, sender: usize, part: DealerPart) -> Result<Ack> {
        let row = self.inner_handle_dealer_part(sender, part)?;
        let mut values = Vec::with_capacity(self.params.dealers());

        // Don't share row 0, as that is what we will use to calculate our own private key share
        for node in 1..=self.params.dealers() {
            let node = row.evaluate(node);

            values.push(node);
        }

        Ok(Ack {
            author: self.our_id,
            part_being_acked: sender,
            commitments: values,
        })
    }

    /// Handle an ack received from another dealer
    ///
    pub fn handle_ack(&mut self, sender: usize, ack: Ack) -> Result<()> {
        self.inner_handle_dealer_ack(sender, ack)?;

        Ok(())
    }

    /// Handle the protocol having been finished and return the public key set and the private key share
    pub fn finalize(self) -> Result<(PublicKeySet, PrivateKeyPart)> {
        let mut pk = Poly::zero().commitment();

        let mut sk = Fr::zero();

        let completed_node_vec: Vec<_> = self.received_parts.values().filter(|part| part.is_complete(self.params.faulty_nodes())).collect();

        let completed_nodes = completed_node_vec.len();

        if completed_nodes <= self.params.faulty_nodes() {
            return Err!(DKGError::NotEnoughFinishedDealers(completed_nodes, self.params.faulty_nodes()));
        }

        for node_info in completed_node_vec {
            pk += node_info.commit.row(0);

            let row = Poly::interpolate(node_info.values.iter()
                .take(self.params.faulty_nodes() + 1)
                .map(|(i, v)| {
                    (i, v)
                }));

            sk.add_assign(&row.evaluate(0));
        }

        let secret_key_share = PrivateKeyPart::from_mut(&mut sk);

        let public_key = PublicKeySet::from(pk);

        Ok((public_key, secret_key_share))
    }

    /// Handle a part from another dealer, verify it and return our corresponding row from the received part,
    /// which we have already verified is valid.
    //TODO: Since we only require the row that is meant for us, does the dealer part have to contain all rows? Seems like
    // a waste of bandwidth
    fn inner_handle_dealer_part(&mut self, sender: usize,
                                DealerPart { commitment: commit, share_values: mut rows, .. }: DealerPart) -> Result<Poly> {
        if rows.len() != self.params.dealers() {
            return Err!(DealerPartError::WrongPartCount);
        }

        if let Some(state) = self.received_parts.get(&sender) {
            if *state.commit() != commit {
                return Err!(DealerPartError::MultipleParts(sender));
            }

            // Already processed a part from this sender
            return Err!(DealerPartError::AlreadyReceived(sender));
        }

        // Get our row from the commitment that we have received from the bivar polynomial
        let commit_row = commit.row(self.our_id);
        eprintln!("Dealer part {}: Getting row from myself: {:?} our id {}", self.our_id, commit_row, self.our_id);

        self.received_parts.insert(sender, NodeState::initialize(sender, commit));

        // Get the row that is meant for us
        let row = rows.swap_remove(self.our_id - 1);

        let row_ser = bincode::serialize(&row).unwrap();

        let row: Poly = bincode::deserialize(&row_ser).unwrap();

        eprintln!("Dealer part {}: Received row from dealer {}: {:?} in ID {}", self.our_id, sender, row, self.our_id - 1);

        // If the row's commitment does not equal the commitment that we have received calculated for our row
        // then the part is invalid
        if row.commitment() != commit_row {
            return Err!(DealerPartError::WrongCommitment(sender));
        }

        if self.pending_acks.contains_key(&sender) {
            let acks = self.pending_acks.remove(&sender).unwrap();

            for (sender, ack) in acks.into_iter() {
                self.handle_ack(sender, ack)?;
            }
        }

        Ok(row.clone())
    }

    fn inner_handle_dealer_ack(&mut self, sender: usize, Ack { author, mut commitments, part_being_acked: confirmed }: Ack) -> Result<()> {
        let mut part = self.received_parts.get_mut(&confirmed);

        if part.is_none() {
            let ack = Ack { author, part_being_acked: confirmed, commitments };


            self.pending_acks.entry(confirmed).or_insert_with(VecDeque::new).push_back((sender,
                                                                                        ack));

            return Ok(());
        }

        let part = part.unwrap();

        let received_value = commitments.swap_remove(self.our_id - 1);

        let val_ser = bincode::serialize(&FieldWrap(received_value)).unwrap();

        let received_value = bincode::deserialize::<FieldWrap<Fr>>(&val_ser).unwrap().into_inner();

        eprintln!("Dealer ack {}: Received ack from dealer {}: {:?} in ID {}", self.our_id, sender, received_value, self.our_id - 1);

        if part.commit.evaluate(self.our_id, sender) != G1Affine::one().mul(received_value) {
            return Err!(AckError::WrongCommitment(sender, confirmed));
        }

        if !part.acks.insert(sender) {
            // We have already received an ack from this sender
            return Err!(AckError::AlreadyReceivedAck(sender, confirmed));
        }

        part.values.insert(sender, received_value);

        if part.has_just_completed(self.params.faulty_nodes()) {
            self.complete += 1;
        }

        Ok(())
    }
}

impl NodeState {
    pub fn initialize(id: usize, commitment: BivarCommitment) -> Self {
        Self {
            id,
            commit: commitment,
            values: Default::default(),
            acks: Default::default(),
        }
    }

    pub fn is_complete(&self, faulty_nodes: usize) -> bool {
        self.acks.len() > 2 * faulty_nodes
    }

    fn has_just_completed(&self, faulty_nodes: usize) -> bool {
        self.acks.len() == (2 * faulty_nodes) + 1
    }
}

impl Ack {
    /// Split this Ack into various directed Acks
    /// The resulting indexes should read 1..=dealers, not 0..dealers, as
    /// the regular Vec structure would indicate
    pub fn split_into_directed(self) -> Vec<DirectedAck> {
        let mut acks = Vec::with_capacity(self.commitments.len());

        for value in self.commitments.into_iter() {
            acks.push(DirectedAck(self.part_being_acked, value));
        }

        acks
    }
}

impl DealerPart {
    /// Split this DealerPart into various directed DealerParts
    /// The resulting indexes should read 1..=dealers, not 0..dealers, as
    /// the regular Vec structure would indicate.
    /// Analogous to [Ack::split_into_directed]
    pub fn split_into_directed(self) -> Vec<DirectedDealerPart> {
        let mut directed_dealer_parts = Vec::with_capacity(self.share_values.len());

        for row in self.share_values.into_iter() {
            directed_dealer_parts.push(DirectedDealerPart(self.commitment.clone(), row));
        }

        directed_dealer_parts
    }
}

#[derive(Debug, Error)]
pub enum DKGError {
    #[error("There were not enough finished dealers to finalize the DKG protocol {0} (needed {1})")]
    NotEnoughFinishedDealers(usize, usize)
}

#[derive(Debug, Error)]
pub enum AckError {
    #[error("Received an Ack relating to a dealer which we have not received a part from")]
    MissingPart,
    #[error("We have already received an Ack from {0} about dealer {1}")]
    AlreadyReceivedAck(usize, usize),
    #[error("The Ack received from {0} about dealer {1} is invalid as the commitment does not match up")]
    WrongCommitment(usize, usize),
}

#[derive(Debug, Error)]
pub enum DealerPartError {
    #[error("We have received a dealer part with an amount of rows that is not equal to the amount of dealers")]
    WrongPartCount,
    #[error("We have already received another part from this dealer {0}")]
    AlreadyReceived(usize),
    #[error("We have received a duplicate part from this dealer {0}")]
    MultipleParts(usize),
    #[error("The commitment data does not match up with the expected")]
    WrongCommitment(usize),
}

#[cfg(test)]
pub mod dkg_test {
    use std::io::stderr;
    use std::iter;
    use std::sync::Arc;
    use std::thread::JoinHandle;
    use std::time::Duration;
    use anyhow::anyhow;
    use getset::{CopyGetters, Getters};
    use rand::Rng;
    use crate::error::*;
    use crate::channel;
    use crate::channel::{ChannelSyncRx, ChannelSyncTx};
    use crate::crypto::threshold_crypto::thold_crypto::dkg::{Ack, DealerPart, DistributedKeyGenerator, DKGParams};
    use crate::crypto::threshold_crypto::thold_crypto::{PrivateKeyPart, PublicKeySet, SecretKeySet};

    const DEALERS: usize = 4;
    const FAULTY_NODES: usize = 1;
    const NODES: usize = DEALERS;
    const QUEUE_SIZE: usize = DEALERS * DEALERS * 2;

    const DATA_TO_SIGN: &[u8] = b"Hello world!";

    struct ChannelDB {
        channels: Vec<ChannelSyncTx<NodeMessage>>,
    }

    struct NodeMessage {
        from: usize,
        msg_type: NodeMessageType,
    }

    enum NodeMessageType {
        DealerPart(DealerPart),
        Ack(Ack),
    }

    /// A node participating the DKG protocol and his info
    #[derive(Getters, CopyGetters)]
    struct Node {
        #[getset(get_copy)]
        id: usize,
        dkg: DistributedKeyGenerator,
        rx_channel: ChannelSyncRx<NodeMessage>,
    }

    #[test]
    fn test_central_dist() {
        let sec_key_set = SecretKeySet::generate_random(DEALERS).unwrap();

        let pk_set = sec_key_set.public_key_set();

        for node in 1..=DEALERS {
            let pk_part = sec_key_set.get_key_share(node).unwrap();

            assert_eq!(pk_part.public_key_part(), pk_set.get_public_key_part(node).unwrap());
        }
    }

    #[test]
    fn test_threshold_signatures() {
        //let node_keys = generate_keys_for_nodes().into_iter().map(|(pk, sk, _, _)| (pk, sk)).collect::<Vec<_>>();
        let node_keys = generate_keys_for_nodes_sync();

        let mut sigs = vec![];

        for (node, (pk, sk)) in node_keys.iter().enumerate() {
            let signature = sk.partially_sign(DATA_TO_SIGN).unwrap();

            assert_eq!(pk.get_public_key_part(node).unwrap(), sk.public_key_part());

            pk.verify_partial_signature(node, DATA_TO_SIGN, &signature).unwrap();

            sigs.push(signature);
        }

        let combined_signatures = sigs.iter().enumerate().map(|(i, sig)| (i, sig.clone())).collect::<Vec<_>>();

        for node in 1..=NODES {
            let (pk, sk) = &node_keys[node - 1];

            let signature = pk.combine_signatures(combined_signatures.iter().cloned().take(FAULTY_NODES + 1).collect::<Vec<_>>().as_slice()).unwrap();

            pk.verify_combined_signature(DATA_TO_SIGN, &signature).unwrap();
        }
    }

    #[test]
    fn test_dkg() {
        let results = generate_keys_for_nodes();

        let (mut pks, mut sks, mut rxs) = (vec![], vec![], vec![]);

        for (node, (pk, sk, rx, _)) in results.into_iter().enumerate() {
            assert_eq!(sk.public_key_part(), pk.get_public_key_part(node).unwrap());

            pks.push(pk);
            sks.push(sk);
            rxs.push(rx);
        }

        let pk = pks.first().unwrap();

        pks.iter().skip(1).for_each(|other_pk| {
            assert_eq!(pk, other_pk);
        });
    }

    fn generate_keys_for_nodes_sync() -> Vec<(PublicKeySet, PrivateKeyPart)> {
        let mut parts = Vec::new();

        let mut participating_nodes = (1..=NODES).map(|node_id| {
            let (tx, rx) = channel::new_bounded_sync(QUEUE_SIZE, None);

            let (dkg, part) = DistributedKeyGenerator::new(DKGParams {
                dealers: DEALERS,
                faulty_nodes: FAULTY_NODES,
            }, node_id).unwrap();

            let node = Node {
                id: node_id,
                dkg,
                rx_channel: rx,
            };

            parts.push((node_id, part));

            node
        }).collect::<Vec<_>>();

        println!("Parts: {:?}", parts.len());

        let mut acks = Vec::new();

        parts[..=FAULTY_NODES].iter().for_each(|part| {
            let (dealer_id, mut part) = part.clone();

            participating_nodes.iter_mut().for_each(|(node)| {
                let ack = node.dkg.handle_part(dealer_id, part.clone()).expect("Failed to handle dealer part");

                if node.id <= (FAULTY_NODES * 2) + 1 {
                    acks.push((node.id, ack));
                }
            })
        });

        acks.iter().for_each(|(node_id, ack)| {
            participating_nodes.iter_mut().for_each(|(node)| {
                assert!(!node.dkg.is_ready());

                node.dkg.handle_ack(*node_id, ack.clone()).expect("Failed to handle ack");
            })
        });

        participating_nodes.into_iter().map(|(node)| {
            assert!(node.dkg.is_ready());

            let (pk, sk) = node.dkg.finalize().unwrap();

            (pk, sk)
        }).collect()
    }

    fn generate_keys_for_nodes() -> Vec<(PublicKeySet, PrivateKeyPart, ChannelSyncRx<NodeMessage>, Vec<(usize, usize)>)> {
        let participating_nodes = (1..=NODES).map(|node_id| {
            let (tx, rx) = channel::new_bounded_sync(QUEUE_SIZE, None);

            let (dkg, part) = DistributedKeyGenerator::new(DKGParams {
                dealers: DEALERS,
                faulty_nodes: FAULTY_NODES,
            }, node_id).unwrap();

            let node = Node {
                id: node_id,
                dkg,
                rx_channel: rx,
            };

            (node, part, tx)
        }).collect::<Vec<_>>();

        let channel_db = Arc::new(ChannelDB {
            channels: participating_nodes.iter().map(|(_, _, tx)| tx.clone()).collect(),
        });

        let mut threads: Vec<_> = participating_nodes.into_iter()
            .zip(iter::repeat_with(|| channel_db.clone()))
            .map(|((node, part, tx), db)| {
                std::thread::spawn(|| run_node(node, part, db))
            }).collect();

        threads.into_iter().map(JoinHandle::join).map(|r| r.unwrap()).collect()
    }

    fn run_node(mut node: Node, dealer_part: DealerPart, txs: Arc<ChannelDB>) -> (PublicKeySet, PrivateKeyPart, ChannelSyncRx<NodeMessage>, Vec<(usize, usize)>) {
        println!("Running node {}", node.id);

        //std::thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(0, 100)));

        let result: Result<()> = txs.channels.iter().map(|tx| {
            tx.send(NodeMessage {
                from: node.id,
                msg_type: NodeMessageType::DealerPart(dealer_part.clone()),
            })
        }).collect();

        result.expect("Failed to send dealer part");

        let mut ack_reception_order = Vec::new();

        loop {
            for x in node.rx_channel.recv() {
                let sender_id = x.from;

                match x.msg_type {
                    NodeMessageType::DealerPart(part) => {
                        match node.dkg.handle_part(sender_id, part) {
                            Ok(ack) => {
                                let res: Result<()> = txs.channels.iter().map(|tx| {
                                    tx.send(NodeMessage {
                                        from: node.id,
                                        msg_type: NodeMessageType::Ack(ack.clone()),
                                    })
                                }).collect();

                                res.expect("Failed to send ack");
                                eprintln!("Client {} received dealer part from {}", node.id, sender_id);
                            }
                            Err(err) => {
                                panic!("Client {} experienced error: {:?}", node.id, err);
                            }
                        }
                    }
                    NodeMessageType::Ack(ack) => {
                        let about = ack.part_being_acked;
                        match node.dkg.handle_ack(sender_id, ack) {
                            Ok(_) => {
                                eprintln!("Client {} received ack from {} about {}", node.id, sender_id, about);
                                ack_reception_order.push((sender_id, about));
                            }
                            Err(err) => {
                                panic!("Client {} experienced error: {:?}", node.id, err);
                            }
                        }
                    }
                }

                if node.dkg.complete() >= DEALERS {
                    eprintln!("Client {} has finished the DKG protocol with {}, needed {}", node.id, node.dkg.complete(), FAULTY_NODES);

                    let Node {
                        id,
                        dkg,
                        rx_channel,
                    } = node;

                    let (pk, sk) = dkg.finalize().unwrap();

                    eprintln!("Client {} has finished the DKG protocol", id);
                    eprintln!("Public key: {:?}", pk);
                    eprintln!("Private key share: {:?}", sk.key.reveal());

                    return (pk, sk, rx_channel, ack_reception_order);
                }
            }
        }
    }


    impl ChannelDB
    {
        pub fn get_channel(&self, to: usize) -> &ChannelSyncTx<NodeMessage> {
            &self.channels[to - 1]
        }
    }
}