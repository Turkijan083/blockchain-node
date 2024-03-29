use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_system::{pallet_prelude::BlockNumberFor, Config};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{
	ClusterBondingParams, ClusterFeesParams, ClusterGovParams, ClusterId, ClusterParams,
	ClusterPricingParams, NodePubKey, NodeType,
};

pub trait ClusterVisitor<T: Config> {
	fn ensure_cluster(cluster_id: &ClusterId) -> Result<(), ClusterVisitorError>;

	fn get_bond_size(
		cluster_id: &ClusterId,
		node_type: NodeType,
	) -> Result<u128, ClusterVisitorError>;

	fn get_pricing_params(
		cluster_id: &ClusterId,
	) -> Result<ClusterPricingParams, ClusterVisitorError>;

	fn get_fees_params(cluster_id: &ClusterId) -> Result<ClusterFeesParams, ClusterVisitorError>;

	fn get_reserve_account_id(cluster_id: &ClusterId) -> Result<T::AccountId, ClusterVisitorError>;

	fn get_chill_delay(
		cluster_id: &ClusterId,
		node_type: NodeType,
	) -> Result<BlockNumberFor<T>, ClusterVisitorError>;

	fn get_unbonding_delay(
		cluster_id: &ClusterId,
		node_type: NodeType,
	) -> Result<BlockNumberFor<T>, ClusterVisitorError>;

	fn get_bonding_params(
		cluster_id: &ClusterId,
	) -> Result<ClusterBondingParams<BlockNumberFor<T>>, ClusterVisitorError>;
}

pub trait ClusterCreator<T: Config, Balance> {
	fn create_new_cluster(
		cluster_id: ClusterId,
		cluster_manager_id: T::AccountId,
		cluster_reserve_id: T::AccountId,
		cluster_params: ClusterParams<T::AccountId>,
		cluster_gov_params: ClusterGovParams<Balance, BlockNumberFor<T>>,
	) -> DispatchResult;
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum ClusterVisitorError {
	ClusterDoesNotExist,
	ClusterGovParamsNotSet,
}

pub trait ClusterManager<T: Config> {
	fn contains_node(cluster_id: &ClusterId, node_pub_key: &NodePubKey) -> bool;
	fn add_node(
		cluster_id: &ClusterId,
		node_pub_key: &NodePubKey,
	) -> Result<(), ClusterManagerError>;
	fn remove_node(
		cluster_id: &ClusterId,
		node_pub_key: &NodePubKey,
	) -> Result<(), ClusterManagerError>;
}

pub enum ClusterManagerError {
	AttemptToAddNonExistentNode,
	AttemptToAddAlreadyAssignedNode,
	AttemptToRemoveNotAssignedNode,
	AttemptToRemoveNonExistentNode,
}
