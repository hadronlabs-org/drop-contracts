import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropDistribution } from 'drop-ts-client';

import { join } from 'path';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';

const SetClass = DropDistribution.Client;

describe('Distribution', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof SetClass>;
    account?: AccountData;
    client?: SigningCosmWasmClient;
    neutronClient?: InstanceType<typeof NeutronClient>;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(t, ['neutron']);

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
        join(__dirname, '../../../artifacts/drop_distribution.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropDistribution.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {},
      'label',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new DropDistribution.Client(
      client,
      context.contractAddress,
    );
  });

  it('query deposit calculation', async () => {
    const { contractClient } = context;
    const res = await contractClient.queryCalcDeposit({
      deposit: '70',
      delegations: {
        total_stake: '110',
        total_on_top: '40',
        total_weight: 70,
        delegations: [
          { valoper_address: 'val1', stake: '10', on_top: '0', weight: 10 },
          { valoper_address: 'val2', stake: '70', on_top: '40', weight: 20 },
          { valoper_address: 'val3', stake: '30', on_top: '0', weight: 40 },
        ],
      },
    });
    res.sort((a, b) => a[0].localeCompare(b[0]));

    expect(res).toEqual([
      ['val1', '10'],
      ['val2', '10'],
      ['val3', '50'],
    ]);
  });

  it('query withdraw calculation', async () => {
    const { contractClient } = context;
    const res = await contractClient.queryCalcWithdraw({
      withdraw: '50',
      delegations: {
        total_stake: '750',
        total_on_top: '100',
        total_weight: 70,
        delegations: [
          {
            valoper_address: 'val1',
            stake: '100',
            on_top: '100',
            weight: 10,
          },
          {
            valoper_address: 'val2',
            stake: '250',
            on_top: '0',
            weight: 20,
          },
          {
            valoper_address: 'val3',
            stake: '400',
            on_top: '0',
            weight: 40,
          },
        ],
      },
    });

    expect(res).toEqual([['val2', '50']]);
  });
});
