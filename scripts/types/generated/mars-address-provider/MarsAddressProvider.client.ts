// @ts-nocheck
/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.16.5.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from '@cosmjs/cosmwasm-stargate'
import { Coin, StdFee } from '@cosmjs/amino'
import {
  InstantiateMsg,
  ExecuteMsg,
  MarsContract,
  QueryMsg,
  AddressResponseItem,
  ArrayOfAddressResponseItem,
} from './MarsAddressProvider.types'
export interface MarsAddressProviderReadOnlyInterface {
  contractAddress: string
  config: () => Promise<InstantiateMsg>
  address: () => Promise<AddressResponseItem>
  addresses: () => Promise<ArrayOfAddressResponseItem>
  allAddresses: ({
    limit,
    startAfter,
  }: {
    limit?: number
    startAfter?: MarsContract
  }) => Promise<ArrayOfAddressResponseItem>
}
export class MarsAddressProviderQueryClient implements MarsAddressProviderReadOnlyInterface {
  client: CosmWasmClient
  contractAddress: string

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client
    this.contractAddress = contractAddress
    this.config = this.config.bind(this)
    this.address = this.address.bind(this)
    this.addresses = this.addresses.bind(this)
    this.allAddresses = this.allAddresses.bind(this)
  }

  config = async (): Promise<InstantiateMsg> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {},
    })
  }
  address = async (): Promise<AddressResponseItem> => {
    return this.client.queryContractSmart(this.contractAddress, {
      address: {},
    })
  }
  addresses = async (): Promise<ArrayOfAddressResponseItem> => {
    return this.client.queryContractSmart(this.contractAddress, {
      addresses: {},
    })
  }
  allAddresses = async ({
    limit,
    startAfter,
  }: {
    limit?: number
    startAfter?: MarsContract
  }): Promise<ArrayOfAddressResponseItem> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_addresses: {
        limit,
        start_after: startAfter,
      },
    })
  }
}
export interface MarsAddressProviderInterface extends MarsAddressProviderReadOnlyInterface {
  contractAddress: string
  sender: string
  setAddress: (
    {
      address,
      contract,
    }: {
      address: string
      contract: MarsContract
    },
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>
  transferOwnership: (
    {
      newOwner,
    }: {
      newOwner: string
    },
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>
}
export class MarsAddressProviderClient
  extends MarsAddressProviderQueryClient
  implements MarsAddressProviderInterface
{
  client: SigningCosmWasmClient
  sender: string
  contractAddress: string

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress)
    this.client = client
    this.sender = sender
    this.contractAddress = contractAddress
    this.setAddress = this.setAddress.bind(this)
    this.transferOwnership = this.transferOwnership.bind(this)
  }

  setAddress = async (
    {
      address,
      contract,
    }: {
      address: string
      contract: MarsContract
    },
    fee: number | StdFee | 'auto' = 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        set_address: {
          address,
          contract,
        },
      },
      fee,
      memo,
      funds,
    )
  }
  transferOwnership = async (
    {
      newOwner,
    }: {
      newOwner: string
    },
    fee: number | StdFee | 'auto' = 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        transfer_ownership: {
          new_owner: newOwner,
        },
      },
      fee,
      memo,
      funds,
    )
  }
}
