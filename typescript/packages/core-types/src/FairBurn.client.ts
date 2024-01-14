/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.30.1.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg, Addr, Decimal, Config } from "./FairBurn.types";
export interface FairBurnReadOnlyInterface {
  contractAddress: string;
  config: () => Promise<Config>;
}
export class FairBurnQueryClient implements FairBurnReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.config = this.config.bind(this);
  }

  config = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
}
export interface FairBurnInterface extends FairBurnReadOnlyInterface {
  contractAddress: string;
  sender: string;
  fairBurn: ({
    recipient
  }: {
    recipient?: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class FairBurnClient extends FairBurnQueryClient implements FairBurnInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.fairBurn = this.fairBurn.bind(this);
  }

  fairBurn = async ({
    recipient
  }: {
    recipient?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      fair_burn: {
        recipient
      }
    }, fee, memo, _funds);
  };
}