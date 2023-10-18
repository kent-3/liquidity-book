import type { SecretNetworkClient } from "secretjs";
import type { CustomToken, NativeToken } from '../misc_types'
import { LB_FACTORY } from "$lib/contracts";
import type * as LBFactory from "./types"

const getLBPairImplementationQuery: LBFactory.GetLBPairImplementationQuery = {
  get_lb_pair_implementation: {}
}

const getLBTokenImplementationQuery: LBFactory.GetLBTokenImplementationQuery = {
  get_lb_token_implementation: {}
}

const getPresetQuery: LBFactory.GetPresetQuery = {
  get_preset: {
    bin_step: 100,
  }
}

export async function queryLBPairImplementation(
  client: SecretNetworkClient,
): Promise<LBFactory.LBPairImplementationResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: LB_FACTORY.address,
    code_hash: LB_FACTORY.code_hash,
    query: getLBPairImplementationQuery,
  })) as LBFactory.LBPairImplementationResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryLBTokenImplementation(
  client: SecretNetworkClient,
): Promise<LBFactory.LBTokenImplementationResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: LB_FACTORY.address,
    code_hash: LB_FACTORY.code_hash,
    query: getLBTokenImplementationQuery,
  })) as LBFactory.LBTokenImplementationResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryPreset(
  client: SecretNetworkClient,
): Promise<LBFactory.PresetResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: LB_FACTORY.address,
    code_hash: LB_FACTORY.code_hash,
    query: getPresetQuery,
  })) as LBFactory.PresetResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryLBPairInformation(
  client: SecretNetworkClient,
  tokenX: CustomToken,
  tokenY: CustomToken,
  bin_step: number,
): Promise<LBFactory.LBPairInformationResponse> {

  const getAllLBPairsQuery: LBFactory.GetLBPairInformationQuery = {
    get_lb_pair_information: {
      token_a: tokenX,
      token_b: tokenY,
      bin_step: bin_step,
    }
  }

  const response = (await client.query.compute.queryContract({
    contract_address: LB_FACTORY.address,
    code_hash: LB_FACTORY.code_hash,
    query: getAllLBPairsQuery,
  })) as LBFactory.LBPairInformationResponse;

  console.log(JSON.stringify(response));
  return response;
}