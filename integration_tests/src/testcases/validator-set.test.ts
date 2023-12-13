import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoValidatorsSet } from '../generated/contractLib';

import { join } from 'path';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupSingle } from '../testSuite';
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
    context.park = await setupSingle('validatorset', 'neutron');

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
        },
      ]),
    );
  });
});
