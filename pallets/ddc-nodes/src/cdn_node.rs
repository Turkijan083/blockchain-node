use crate::node::{Node, NodeError, NodeParams, NodeProps, NodePropsRef, NodePubKeyRef, NodeTrait};
use codec::{Decode, Encode};
use ddc_primitives::{CDNNodePubKey, ClusterId, NodePubKey, NodeType};
use frame_support::{parameter_types, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

parameter_types! {
	pub MaxCDNNodeParamsLen: u16 = 2048;
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct CDNNode<T: frame_system::Config> {
	pub pub_key: CDNNodePubKey,
	pub provider_id: T::AccountId,
	pub cluster_id: Option<ClusterId>,
	pub props: CDNNodeProps,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct CDNNodeProps {
	// this is a temporal way of storing node parameters as a stringified json,
	// should be replaced with specific properties for this type of node once they are defined
	pub params: BoundedVec<u8, MaxCDNNodeParamsLen>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub struct CDNNodeParams {
	pub params: Vec<u8>, // should be replaced with specific parameters for this type of node
}

impl<T: frame_system::Config> NodeTrait<T> for CDNNode<T> {
	fn get_pub_key<'a>(&'a self) -> NodePubKeyRef<'a> {
		NodePubKeyRef::CDNPubKeyRef(&self.pub_key)
	}
	fn get_provider_id(&self) -> &T::AccountId {
		&self.provider_id
	}
	fn get_props<'a>(&'a self) -> NodePropsRef<'a> {
		NodePropsRef::CDNPropsRef(&self.props)
	}
	fn set_props(&mut self, props: NodeProps) -> Result<(), NodeError> {
		self.props = match props {
			NodeProps::CDNProps(props) => props,
			_ => return Err(NodeError::InvalidCDNNodeProps),
		};
		Ok(())
	}
	fn set_params(&mut self, node_params: NodeParams) -> Result<(), NodeError> {
		self.props.params = match node_params {
			NodeParams::CDNParams(cdn_params) => match cdn_params.params.try_into() {
				Ok(vec) => vec,
				Err(_) => return Err(NodeError::CDNNodeParamsExceedsLimit),
			},
			_ => return Err(NodeError::InvalidCDNNodeParams),
		};
		Ok(())
	}
	fn get_cluster_id(&self) -> &Option<ClusterId> {
		&self.cluster_id
	}
	fn set_cluster_id(&mut self, cluster_id: Option<ClusterId>) {
		self.cluster_id = cluster_id;
	}
	fn get_type(&self) -> NodeType {
		NodeType::CDN
	}
	fn new(
		node_pub_key: NodePubKey,
		provider_id: T::AccountId,
		node_params: NodeParams,
	) -> Result<Node<T>, NodeError> {
		match node_pub_key {
			NodePubKey::CDNPubKey(pub_key) => match node_params {
				NodeParams::CDNParams(node_params) => Ok(Node::CDN(CDNNode::<T> {
					provider_id,
					pub_key,
					cluster_id: None,
					props: CDNNodeProps {
						params: match node_params.params.try_into() {
							Ok(vec) => vec,
							Err(_) => return Err(NodeError::CDNNodeParamsExceedsLimit),
						},
					},
				})),
				_ => Err(NodeError::InvalidCDNNodeParams),
			},
			_ => Err(NodeError::InvalidCDNNodePubKey),
		}
	}
}