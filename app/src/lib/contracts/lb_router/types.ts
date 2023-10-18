import type { CustomToken, NativeToken, TokenType } from '../misc_types'

export interface LBPair {
  token_x: string;
  token_y: string;
  bin_step: number;
  contract: {
    address: string;
    code_hash: string;
  };
}

export interface LBPairInformation {
  bin_step: number;
  lb_pair: LBPair;
  created_by_owner: boolean;
  ignored_for_routing: boolean;
}

export interface CreateLBPairMsg {
  create_lb_pair: {
    token_x: TokenType;
    token_y: TokenType;
    active_id: number;
    bin_step: number;
  }
}
