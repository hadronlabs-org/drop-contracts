import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoValidatorsSet } from '../generated/contractLib';

import { join } from 'path';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';

const SetClass = LidoValidatorsSet.Client;

describe('Validator set', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof SetClass>;
    account?: AccountData;
    client?: SigningCosmWasmClient;
    neutronClient?: InstanceType<typeof NeutronClient>;
  } = {};

  beforeAll(async () => {
    context.park = await setupPark('validatorset', ['neutron'], false);
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );

    context.account = (await context.wallet.getAccounts())[0];
    context.neutronClient = new NeutronClient({
      apiURL: `http://127.0.0.1:${context.park.ports.neutron.rest}`,
      rpcURL: `127.0.0.1:${context.park.ports.neutron.rpc}`,
      prefix: 'neutron',
    });

    context.client = await SigningCosmWasmClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.neutron.rpc}`,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/lido_validators_set.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoValidatorsSet.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        owner: account.address,
        stats_contract: account.address,
      },
      'label',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new LidoValidatorsSet.Client(
      client,
      context.contractAddress,
    );
  });

  it('Add single validator', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.updateValidator(
      account.address,
      {
        validator: {
          valoper_address: 'valoper1',
          weight: 1,
        },
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();

    const validators = await contractClient.queryValidators();

    expect(validators).toEqual(
      expect.arrayContaining([
        {
          valoper_address: 'valoper1',
          weight: 1,
          last_processed_remote_height: null,
          last_processed_local_height: null,
          last_validated_height: null,
          last_commission_in_range: null,
          uptime: '0',
          tombstone: false,
          jailed_number: null,
          init_proposal: null,
          total_passed_proposals: 0,
          total_voted_proposals: 0,
        },
      ]),
    );
  });

  it('Add bunch of validators', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.updateValidators(
      account.address,
      {
        validators: [
          {
            valoper_address: 'valoper2',
            weight: 2,
          },
          {
            valoper_address: 'valoper3',
            weight: 3,
          },
        ],
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();

    const validators = await contractClient.queryValidators();

    expect(validators).toEqual(
      expect.arrayContaining([
        {
          valoper_address: 'valoper2',
          weight: 2,
          last_processed_remote_height: null,
          last_processed_local_height: null,
          last_validated_height: null,
          last_commission_in_range: null,
          uptime: '0',
          tombstone: false,
          jailed_number: null,
          init_proposal: null,
          total_passed_proposals: 0,
          total_voted_proposals: 0,
        },
        {
          valoper_address: 'valoper3',
          weight: 3,
          last_processed_remote_height: null,
          last_processed_local_height: null,
          last_validated_height: null,
          last_commission_in_range: null,
          uptime: '0',
          tombstone: false,
          jailed_number: null,
          init_proposal: null,
          total_passed_proposals: 0,
          total_voted_proposals: 0,
        },
      ]),
    );
  });

  it('Update validator info', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.updateValidatorsInfo(
      account.address,
      {
        validators: [
          {
            valoper_address: 'valoper2',
            tombstone: true,
            uptime: '0.5',
            jailed_number: 1,
            last_commission_in_range: 1234,
            last_processed_local_height: 2345,
            last_processed_remote_height: 3456,
            last_validated_height: 4567,
          },
          {
            valoper_address: 'valoper3',
            tombstone: false,
            uptime: '0.96',
            jailed_number: 3,
          },
        ],
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();

    const validators = await contractClient.queryValidators();

    expect(validators).toEqual(
      expect.arrayContaining([
        {
          valoper_address: 'valoper2',
          weight: 2,
          last_processed_remote_height: 3456,
          last_processed_local_height: 2345,
          last_validated_height: 4567,
          last_commission_in_range: 1234,
          uptime: '0.5',
          tombstone: true,
          jailed_number: 1,
          init_proposal: null,
          total_passed_proposals: 0,
          total_voted_proposals: 0,
        },
        {
          valoper_address: 'valoper3',
          weight: 3,
          last_processed_remote_height: null,
          last_processed_local_height: null,
          last_validated_height: null,
          last_commission_in_range: null,
          uptime: '0.96',
          tombstone: false,
          jailed_number: 3,
          init_proposal: null,
          total_passed_proposals: 0,
          total_voted_proposals: 0,
        },
      ]),
    );
  });
});
