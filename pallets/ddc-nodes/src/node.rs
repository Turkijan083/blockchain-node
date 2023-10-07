use crate::{
	cdn_node::{CDNNode, CDNNodeParams, CDNNodeProps, CDNNodePubKey},
	pallet::Error,
	storage_node::{StorageNode, StorageNodeParams, StorageNodeProps, StorageNodePubKey},
};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum Node<ProviderId> {
	Storage(StorageNode<ProviderId>),
	CDN(CDNNode<ProviderId>),
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeParams {
	StorageParams(StorageNodeParams),
	CDNParams(CDNNodeParams),
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodePubKey {
	StoragePubKey(StorageNodePubKey),
	CDNPubKey(CDNNodePubKey),
}

#[derive(Clone, RuntimeDebug, PartialEq)]
pub enum NodePubKeyRef<'a> {
	StoragePubKeyRef(&'a StorageNodePubKey),
	CDNPubKeyRef(&'a CDNNodePubKey),
}

impl<'a> NodePubKeyRef<'a> {
	pub fn to_owned(&self) -> NodePubKey {
		match &self {
			NodePubKeyRef::StoragePubKeyRef(pub_key_ref) =>
				NodePubKey::StoragePubKey((**pub_key_ref).clone()),
			NodePubKeyRef::CDNPubKeyRef(pub_key_ref) =>
				NodePubKey::CDNPubKey((**pub_key_ref).clone()),
		}
	}
}

#[derive(Clone, RuntimeDebug, PartialEq)]
pub enum NodePropsRef<'a> {
	StoragePropsRef(&'a StorageNodeProps),
	CDNPropsRef(&'a CDNNodeProps),
}

pub trait NodeTrait<ProviderId> {
	fn get_pub_key<'a>(&'a self) -> NodePubKeyRef<'a>;
	fn get_provider_id(&self) -> &ProviderId;
	fn get_props<'a>(&'a self) -> NodePropsRef<'a>;
	fn get_type(&self) -> NodeType;
	fn from_params(
		provider_id: ProviderId,
		params: NodeParams,
	) -> Result<Node<ProviderId>, NodeError>;
}

impl<ProviderId> NodeTrait<ProviderId> for Node<ProviderId> {
	fn get_pub_key<'a>(&'a self) -> NodePubKeyRef<'a> {
		match &self {
			Node::Storage(node) => node.get_pub_key(),
			Node::CDN(node) => node.get_pub_key(),
		}
	}
	fn get_provider_id(&self) -> &ProviderId {
		match &self {
			Node::Storage(node) => node.get_provider_id(),
			Node::CDN(node) => node.get_provider_id(),
		}
	}
	fn get_props<'a>(&'a self) -> NodePropsRef<'a> {
		match &self {
			Node::Storage(node) => node.get_props(),
			Node::CDN(node) => node.get_props(),
		}
	}
	fn get_type(&self) -> NodeType {
		match &self {
			Node::Storage(node) => node.get_type(),
			Node::CDN(node) => node.get_type(),
		}
	}
	fn from_params(
		provider_id: ProviderId,
		params: NodeParams,
	) -> Result<Node<ProviderId>, NodeError> {
		match params {
			NodeParams::StorageParams(_) => StorageNode::from_params(provider_id, params),
			NodeParams::CDNParams(_) => CDNNode::from_params(provider_id, params),
		}
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeType {
	Storage = 1,
	CDN = 2,
}

impl From<NodeType> for u8 {
	fn from(node_type: NodeType) -> Self {
		match node_type {
			NodeType::Storage => 1,
			NodeType::CDN => 2,
		}
	}
}

impl TryFrom<u8> for NodeType {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(NodeType::Storage),
			2 => Ok(NodeType::CDN),
			_ => Err(()),
		}
	}
}

pub enum NodeError {
	InvalidStorageNodeParams,
	InvalidCDNNodeParams,
}

impl<T> From<NodeError> for Error<T> {
	fn from(error: NodeError) -> Self {
		match error {
			NodeError::InvalidStorageNodeParams => Error::<T>::InvalidNodeParams,
			NodeError::InvalidCDNNodeParams => Error::<T>::InvalidNodeParams,
		}
	}
}
