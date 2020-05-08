use std::{
    collections::{BTreeMap, BTreeSet},
    convert::{TryFrom, TryInto},
};
use types::{
    Contract, ContractPackage, EntryPoint, EntryPointAccess, EntryPointType, Group, Parameter,
};

use crate::engine_server::{mappings::ParsingError, state};

impl From<ContractPackage> for state::ContractMetadata {
    fn from(value: ContractPackage) -> state::ContractMetadata {
        let mut metadata = state::ContractMetadata::new();
        metadata.set_access_key(value.access_key().into());

        for &removed_version in value.removed_versions().iter() {
            metadata.mut_removed_versions().push(removed_version.into())
        }

        for (existing_group, urefs) in value.groups().iter() {
            let mut entrypoint_group = state::ContractHeader_EntryPoint_Group::new();
            entrypoint_group.set_name(existing_group.value().to_string());

            let mut metadata_group = state::ContractMetadata_Group::new();
            metadata_group.set_group(entrypoint_group);

            for &uref in urefs {
                metadata_group.mut_urefs().push(uref.into());
            }

            metadata.mut_groups().push(metadata_group);
        }

        for (version, contract_header) in value.take_active_versions().into_iter() {
            let mut active_version = state::ContractMetadata_ActiveVersion::new();
            active_version.set_version(version.into());
            active_version.set_contract_header(contract_header.into());
            metadata.mut_active_versions().push(active_version)
        }

        metadata
    }
}

impl TryFrom<state::ContractMetadata> for ContractPackage {
    type Error = ParsingError;
    fn try_from(mut value: state::ContractMetadata) -> Result<ContractPackage, Self::Error> {
        let access_uref = value.take_access_key().try_into()?;
        let mut metadata = ContractPackage::new(access_uref);
        for mut active_version in value.take_active_versions().into_iter() {
            let version = active_version.take_version().into();
            let header = active_version.take_contract_header().try_into()?;
            metadata.active_versions_mut().insert(version, header);
        }
        for removed_version in value.take_removed_versions().into_iter() {
            metadata
                .removed_versions_mut()
                .insert(removed_version.into());
        }

        let groups = metadata.groups_mut();
        for mut group in value.take_groups().into_iter() {
            let group_name = group.take_group().take_name();
            let mut urefs = BTreeSet::new();
            for uref in group.take_urefs().into_iter() {
                urefs.insert(uref.try_into()?);
            }
            groups.insert(Group::new(group_name), urefs);
        }
        Ok(metadata)
    }
}

impl From<Contract> for state::ContractHeader {
    fn from(value: Contract) -> Self {
        let mut res = state::ContractHeader::new();
        res.set_contract_key(value.contract_key().into());
        res.set_protocol_version(value.protocol_version().into());

        for (name, entrypoint) in value.take_methods().into_iter() {
            let mut method_entry = state::ContractHeader_MethodEntry::new();
            method_entry.set_name(name.to_string());
            method_entry.set_entrypoint(entrypoint.into());
            res.mut_methods().push(method_entry);
        }
        res
    }
}

impl TryFrom<state::ContractHeader> for Contract {
    type Error = ParsingError;
    fn try_from(mut value: state::ContractHeader) -> Result<Contract, Self::Error> {
        let mut methods = BTreeMap::new();
        for mut method_entry in value.take_methods().into_iter() {
            methods.insert(
                method_entry.take_name(),
                method_entry.take_entrypoint().try_into()?,
            );
        }

        let contract_key = value.take_contract_key().try_into()?;
        Ok(Contract::new(
            methods,
            contract_key,
            value.take_protocol_version().into(),
        ))
    }
}

impl From<EntryPoint> for state::ContractHeader_EntryPoint {
    fn from(value: EntryPoint) -> Self {
        let (args, ret, entry_point_access, entry_point_type) = value.into();

        let mut res = state::ContractHeader_EntryPoint::new();

        for arg in args.into_iter() {
            let (name, cl_type) = arg.into();
            let mut state_arg = state::ContractHeader_EntryPoint_Arg::new();

            state_arg.set_name(name);
            state_arg.set_cl_type(cl_type.into());

            res.mut_args().push(state_arg)
        }

        res.set_ret(ret.into());

        match entry_point_access {
            EntryPointAccess::Public => {
                res.set_public(state::ContractHeader_EntryPoint_Public::new())
            }
            EntryPointAccess::Groups(groups) => {
                let mut state_groups = state::ContractHeader_EntryPoint_Groups::new();
                for group in groups.into_iter() {
                    let mut state_group = state::ContractHeader_EntryPoint_Group::new();
                    let name = group.into();
                    state_group.set_name(name);
                    state_groups.mut_groups().push(state_group);
                }
                res.set_groups(state_groups)
            }
        }

        match entry_point_type {
            EntryPointType::Session => {
                res.set_session(state::ContractHeader_EntryPoint_Session::new())
            }
            EntryPointType::Contract => {
                res.set_contract(state::ContractHeader_EntryPoint_Contract::new())
            }
        }
        res
    }
}

impl TryFrom<state::ContractHeader_EntryPoint> for EntryPoint {
    type Error = ParsingError;
    fn try_from(mut value: state::ContractHeader_EntryPoint) -> Result<EntryPoint, Self::Error> {
        let mut args = Vec::new();

        let ret = value.take_ret().try_into()?;

        for mut arg in value.take_args().into_iter() {
            args.push(Parameter::new(
                arg.take_name(),
                arg.take_cl_type().try_into()?,
            ));
        }

        let entry_point_access = match value.access {
            Some(state::ContractHeader_EntryPoint_oneof_access::public(_)) => {
                EntryPointAccess::Public
            }
            Some(state::ContractHeader_EntryPoint_oneof_access::groups(mut groups)) => {
                let mut vec = Vec::new();
                for mut group in groups.take_groups().into_iter() {
                    vec.push(Group::new(group.take_name()));
                }
                EntryPointAccess::Groups(vec)
            }
            None => return Err("Unable to parse Protobuf entry point access".into()),
        };
        let entry_point_type = match value.entry_point_type {
            Some(state::ContractHeader_EntryPoint_oneof_entry_point_type::session(_)) => {
                EntryPointType::Session
            }
            Some(state::ContractHeader_EntryPoint_oneof_entry_point_type::contract(_)) => {
                EntryPointType::Contract
            }
            None => return Err("Unable to parse Protobuf entry point type".into()),
        };
        Ok(EntryPoint::new(
            args,
            ret,
            entry_point_access,
            entry_point_type,
        ))
    }
}
