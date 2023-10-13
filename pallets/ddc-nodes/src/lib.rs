//! # DDC Nodes Pallet
//!
//! The DDC Nodes pallet is used to manage nodes in DDC Cluster
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## GenesisConfig
//!
//! The DDC Nodes pallet depends on the [`GenesisConfig`]. The
//! `GenesisConfig` is optional and allow to set some initial nodes in DDC.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_core::hash::H160;
use sp_std::prelude::*;

pub use pallet::*;
mod cdn_node;
mod node;
mod storage_node;

pub use crate::{
	cdn_node::{CDNNode, CDNNodePubKey},
	node::{Node, NodeError, NodeParams, NodePubKey, NodeTrait},
	storage_node::{StorageNode, StorageNodePubKey},
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		NodeCreated { node_type: u8, node_pub_key: NodePubKey },
		NodeRemoved { node_type: u8, node_pub_key: NodePubKey },
	}

	#[pallet::error]
	pub enum Error<T> {
		NodeAlreadyExists,
		NodeDoesNotExist,
		InvalidNodeParams,
		NodeParamsExceedsLimit,
		OnlyNodeProvider,
		NodeIsAssignedToCluster,
	}

	#[pallet::storage]
	#[pallet::getter(fn storage_nodes)]
	pub type StorageNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, StorageNodePubKey, StorageNode<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn cdn_nodes)]
	pub type CDNNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, CDNNodePubKey, CDNNode<T::AccountId>>;

	// todo: add the type to the Config
	pub type ClusterId = H160;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_node(origin: OriginFor<T>, node_params: NodeParams) -> DispatchResult {
			let provider_id = ensure_signed(origin)?;
			let node = Node::<T::AccountId>::new(provider_id, node_params)
				.map_err(|e| Into::<Error<T>>::into(NodeError::from(e)))?;
			let node_type = node.get_type();
			let node_pub_key = node.get_pub_key().to_owned();
			Self::create(node).map_err(|e| Into::<Error<T>>::into(NodeRepositoryError::from(e)))?;
			Self::deposit_event(Event::<T>::NodeCreated {
				node_pub_key,
				node_type: node_type.into(),
			});
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_node(origin: OriginFor<T>, node_pub_key: NodePubKey) -> DispatchResult {
			let provider_id = ensure_signed(origin)?;
			let node = Self::get(node_pub_key.clone())
				.map_err(|e| Into::<Error<T>>::into(NodeRepositoryError::from(e)))?;
			ensure!(node.get_provider_id() == &provider_id, Error::<T>::OnlyNodeProvider);
			ensure!(node.get_cluster_id().is_none(), Error::<T>::NodeIsAssignedToCluster);
			Self::remove(node_pub_key.clone())
				.map_err(|e| Into::<Error<T>>::into(NodeRepositoryError::from(e)))?;
			Self::deposit_event(Event::<T>::NodeCreated {
				node_pub_key,
				node_type: node.get_type().into(),
			});
			Ok(())
		}
	}

	pub trait NodeRepository<T: frame_system::Config> {
		fn create(node: Node<T::AccountId>) -> Result<(), NodeRepositoryError>;
		fn get(node_pub_key: NodePubKey) -> Result<Node<T::AccountId>, NodeRepositoryError>;
		fn update(node: Node<T::AccountId>) -> Result<(), NodeRepositoryError>;
		fn remove(node_pub_key: NodePubKey) -> Result<(), NodeRepositoryError>;
	}

	pub enum NodeRepositoryError {
		StorageNodeAlreadyExists,
		CDNNodeAlreadyExists,
		StorageNodeDoesNotExist,
		CDNNodeDoesNotExist,
	}

	impl<T> From<NodeRepositoryError> for Error<T> {
		fn from(error: NodeRepositoryError) -> Self {
			match error {
				NodeRepositoryError::StorageNodeAlreadyExists => Error::<T>::NodeAlreadyExists,
				NodeRepositoryError::CDNNodeAlreadyExists => Error::<T>::NodeAlreadyExists,
				NodeRepositoryError::StorageNodeDoesNotExist => Error::<T>::NodeDoesNotExist,
				NodeRepositoryError::CDNNodeDoesNotExist => Error::<T>::NodeDoesNotExist,
			}
		}
	}

	impl<T: Config> NodeRepository<T> for Pallet<T> {
		fn create(node: Node<T::AccountId>) -> Result<(), NodeRepositoryError> {
			match node {
				Node::Storage(storage_node) => {
					if StorageNodes::<T>::contains_key(&storage_node.pub_key) {
						return Err(NodeRepositoryError::StorageNodeAlreadyExists)
					}
					StorageNodes::<T>::insert(storage_node.pub_key.clone(), storage_node);
					Ok(())
				},
				Node::CDN(cdn_node) => {
					if CDNNodes::<T>::contains_key(&cdn_node.pub_key) {
						return Err(NodeRepositoryError::CDNNodeAlreadyExists)
					}
					CDNNodes::<T>::insert(cdn_node.pub_key.clone(), cdn_node);
					Ok(())
				},
			}
		}

		fn get(node_pub_key: NodePubKey) -> Result<Node<T::AccountId>, NodeRepositoryError> {
			match node_pub_key {
				NodePubKey::StoragePubKey(pub_key) => match StorageNodes::<T>::try_get(pub_key) {
					Ok(storage_node) => Ok(Node::Storage(storage_node)),
					Err(_) => Err(NodeRepositoryError::StorageNodeDoesNotExist),
				},
				NodePubKey::CDNPubKey(pub_key) => match CDNNodes::<T>::try_get(pub_key) {
					Ok(cdn_node) => Ok(Node::CDN(cdn_node)),
					Err(_) => Err(NodeRepositoryError::CDNNodeDoesNotExist),
				},
			}
		}

		fn update(node: Node<T::AccountId>) -> Result<(), NodeRepositoryError> {
			match node {
				Node::Storage(storage_node) => {
					if !StorageNodes::<T>::contains_key(&storage_node.pub_key) {
						return Err(NodeRepositoryError::StorageNodeDoesNotExist)
					}
					StorageNodes::<T>::insert(storage_node.pub_key.clone(), storage_node);
				},
				Node::CDN(cdn_node) => {
					if !CDNNodes::<T>::contains_key(&cdn_node.pub_key) {
						return Err(NodeRepositoryError::CDNNodeDoesNotExist)
					}
					CDNNodes::<T>::insert(cdn_node.pub_key.clone(), cdn_node);
				},
			}
			Ok(())
		}

		fn remove(node_pub_key: NodePubKey) -> Result<(), NodeRepositoryError> {
			match node_pub_key {
				NodePubKey::StoragePubKey(pub_key) => {
					StorageNodes::<T>::remove(pub_key);
					Ok(())
				},
				NodePubKey::CDNPubKey(pub_key) => {
					CDNNodes::<T>::remove(pub_key);
					Ok(())
				},
			}
		}
	}
}
