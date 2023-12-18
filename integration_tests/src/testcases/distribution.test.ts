import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoDistribution } from '../generated/contractLib';

import { join } from 'path';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupSingle } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { IdealDelegation } from '../generated/contractLib/lidoDistribution';

const SetClass = LidoDistribution.Client;

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

  beforeAll(async () => {
    context.park = await setupSingle('distribution', 'neutron');

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
        join(__dirname, '../../../artifacts/lido_distribution.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoDistribution.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {},
      'label',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new LidoDistribution.Client(
      client,
      context.contractAddress,
    );
  });

  it('query deposit calculation', async () => {
    const { contractClient } = context;
    let res: IdealDelegation[] = [];
    await waitFor(async () => {
      res = await contractClient.queryCalcDeposit({
        deposit: '100',
        delegations: [{ valoper_address: 'valoper1', stake: '0', weight: 10 }],
      });
      return res.length > 0;
    }, 60_000);

    expect(res).toEqual([
      {
        valoper_address: 'valoper1',
        ideal_stake: '100',
        current_stake: '0',
        stake_change: '100',
        weight: 10,
      },
    ]);
  });

  it('query withdraw calculation', async () => {
    const { contractClient } = context;
    let res: IdealDelegation[] = [];
    await waitFor(async () => {
      res = await contractClient.queryCalcWithdraw({
        withdraw: '50',
        delegations: [
          { valoper_address: 'valoper1', stake: '100', weight: 10 },
        ],
      });
      return res.length > 0;
    }, 60_000);

    expect(res).toEqual([
      {
        valoper_address: 'valoper1',
        ideal_stake: '50',
        current_stake: '100',
        stake_change: '50',
        weight: 10,
      },
    ]);
  });
});
