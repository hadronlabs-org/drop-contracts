import { afterAll, beforeAll, it, describe, expect } from 'vitest';
import Cosmopark from '@neutron-org/cosmopark';
import { setupPark } from '../testSuite';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { join } from 'path';
import fs from 'fs';
import { DropSplitter } from 'drop-ts-client';

const DropSplitterClass = DropSplitter.Client;

describe('Splitter', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    account?: AccountData;
    neutronClient?: InstanceType<typeof NeutronClient>;
    client?: SigningCosmWasmClient;

    contractClient?: InstanceType<typeof DropSplitterClass>;
    receiver1?: string;
    receiver2?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron'],
      {},
      {
        hermes: false,
      },
    );
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
    context.receiver1 = (
      await (
        await DirectSecp256k1HdWallet.generate(12, {
          prefix: 'neutron',
        })
      ).getAccounts()
    )[0].address;
    context.receiver2 = (
      await (
        await DirectSecp256k1HdWallet.generate(12, {
          prefix: 'neutron',
        })
      ).getAccounts()
    )[0].address;
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    const { codeId } = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/drop_splitter.wasm')),
      1.5,
    );
    const { contractAddress } = await DropSplitter.Client.instantiate(
      client,
      account.address,
      codeId,
      {
        config: {
          denom: 'untrn',
          receivers: [
            [context.receiver1, '33'],
            [context.receiver2, '67'],
          ],
        },
      },
      'label',
      'auto',
      [],
    );
    context.contractClient = new DropSplitter.Client(client, contractAddress);
  });

  it('distribute ideal ratio', async () => {
    const { contractClient, account, neutronClient } = context;
    await contractClient.distribute(account.address, 1.5, '', [
      { amount: String(100), denom: 'untrn' },
    ]);

    await assertBalance(neutronClient, contractClient.contractAddress, 0);
    await assertBalance(neutronClient, context.receiver1, 33);
    await assertBalance(neutronClient, context.receiver2, 67);
  });

  it('distribute ideal ratio x5', async () => {
    const { contractClient, account, neutronClient } = context;
    await contractClient.distribute(account.address, 1.5, '', [
      { amount: String(100 * 5), denom: 'untrn' },
    ]);

    await assertBalance(neutronClient, contractClient.contractAddress, 0);
    await assertBalance(neutronClient, context.receiver1, 33 * (1 + 5));
    await assertBalance(neutronClient, context.receiver2, 67 * (1 + 5));
  });

  it('distribute uneven ratio', async () => {
    const { contractClient, account, neutronClient } = context;
    await contractClient.distribute(account.address, 1.5, '', [
      { amount: String(120), denom: 'untrn' },
    ]);

    await assertBalance(neutronClient, contractClient.contractAddress, 20);
    await assertBalance(neutronClient, context.receiver1, 33 * (1 + 5 + 1));
    await assertBalance(neutronClient, context.receiver2, 67 * (1 + 5 + 1));
  });

  it('distribute less than ratio', async () => {
    const { contractClient, account, neutronClient } = context;
    const res = contractClient.distribute(account.address, 1.5);

    await expect(res).rejects.to.toMatch(/Insufficient funds/);

    await assertBalance(neutronClient, contractClient.contractAddress, 20);
    await assertBalance(neutronClient, context.receiver1, 33 * (1 + 5 + 1));
    await assertBalance(neutronClient, context.receiver2, 67 * (1 + 5 + 1));
  });

  it('update ratio', async () => {
    const { contractClient, account } = context;
    await contractClient.updateConfig(account.address, {
      new_config: {
        denom: 'untrn',
        receivers: [
          [context.receiver1, '1'],
          [context.receiver2, '4'],
        ],
      },
    });
  });

  it('distribute new ideal ratio', async () => {
    const { contractClient, account, neutronClient } = context;
    await contractClient.distribute(account.address, 1.5);

    await assertBalance(neutronClient, contractClient.contractAddress, 0);
    await assertBalance(neutronClient, context.receiver1, 33 * (1 + 5 + 1) + 4);
    await assertBalance(
      neutronClient,
      context.receiver2,
      67 * (1 + 5 + 1) + 16,
    );
  });
});

const assertBalance = async (
  neutronClient: InstanceType<typeof NeutronClient>,
  address: string,
  expectedBalance: number,
) => {
  expect(
    (
      await neutronClient.CosmosBankV1Beta1.query.queryBalance(address, {
        denom: 'untrn',
      })
    ).data.balance.amount,
  ).toEqual(String(expectedBalance));
};
