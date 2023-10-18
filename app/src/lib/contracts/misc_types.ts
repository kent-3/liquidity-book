export interface CustomToken {
    custom_token: {
      contract_addr: string;
      token_code_hash: string;
      // viewing_key: string;
    };
  }
  
  export interface NativeToken {
    native_token: {
      denom: string;
    };
  }
  
  export type TokenType = CustomToken | NativeToken;