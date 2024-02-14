import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoPump } from '../generated/contractLib';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  SigningStargateClient,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';

const LidoPumpClass = LidoPump.Client;

describe('Pump', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof LidoPumpClass>;
    account?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    neutronUserAddress?: string;
    neutronSecondUserAddress?: string;
    validatorAddress?: string;
    secondValidatorAddress?: string;
    tokenizedDenomOnNeutron?: string;
    coreCoreId?: number;
    tokenCodeId?: number;
    exchangeRate?: number;
    tokenContractAddress?: string;
    neutronIBCDenom?: string;
  } = {};

  beforeAll(async () => {
    context.park = await setupPark('pump', ['neutron', 'gaia'], true);
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.gaiaWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'cosmos',
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
    context.gaiaClient = await SigningStargateClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
      context.gaiaWallet,
      {
        gasPrice: GasPrice.fromString('0.025stake'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
    );
    context.gaiaQueryClient = QueryClient.withExtensions(
      tmClient,
      setupStakingExtension,
      setupBankExtension,
    );
    const secondWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo2.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.neutronSecondUserAddress = (
      await secondWallet.getAccounts()
    )[0].address;
    context.gaiaUserAddress = (
      await context.gaiaWallet.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account, neutronSecondUserAddress } = context;
    const res = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/lido_pump.wasm')),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoPump.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        connection_id: 'connection-0',
        dest_address: neutronSecondUserAddress,
        dest_channel: 'channel-0',
        dest_port: 'transfer',
        ibc_fees: {
          timeout_fee: '10000',
          ack_fee: '10000',
          recv_fee: '0',
          register_fee: '1000000',
        },
        local_denom: 'untrn',
        refundee: neutronSecondUserAddress,
        timeout: {
          local: 100,
          remote: 100,
        },
        owner: account.address,
      },
      'label',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new LidoPump.Client(
      client,
      context.contractAddress,
    );
  });

  it('register ICA w/o funds', async () => {
    const { contractClient, neutronUserAddress } = context;
    await expect(
      contractClient.registerICA(neutronUserAddress, 1.5),
    ).rejects.toThrowError(/No funds sent/);
  });
  it('register ICA w less then needed funds', async () => {
    const { contractClient, neutronUserAddress } = context;
    await expect(
      contractClient.registerICA(neutronUserAddress, 1.5, undefined, [
        {
          amount: '1',
          denom: 'untrn',
        },
      ]),
    ).rejects.toThrowError(/expected at least/);
  });
  it('register ICA', async () => {
    const { contractClient, neutronUserAddress } = context;
    const res = await contractClient.registerICA(
      neutronUserAddress,
      1.5,
      undefined,
      [
        {
          amount: '1000000',
          denom: 'untrn',
        },
      ],
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await contractClient.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 50_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddress = ica;
  });
  it('send some funds to ICA', async () => {
    const { gaiaClient, gaiaUserAddress, icaAddress } = context;
    const res = await gaiaClient.sendTokens(
      gaiaUserAddress,
      icaAddress,
      [
        {
          amount: '1000000',
          denom: 'stake',
        },
      ],
      1.5,
    );
    expect(res.transactionHash).toHaveLength(64);
  });
  it('try to push pump w/o funds', async () => {
    const { contractClient, neutronUserAddress } = context;
    await expect(
      contractClient.push(
        neutronUserAddress,
        {
          coins: [{ amount: '10', denom: 'stake' }],
        },
        1.5,
      ),
    ).rejects.toThrowError(/No funds sent/);
  });
  it('try to push pump w less funds', async () => {
    const { contractClient, neutronUserAddress } = context;
    await expect(
      contractClient.push(
        neutronUserAddress,
        {
          coins: [{ amount: '10', denom: 'stake' }],
        },
        1.5,
        undefined,
        [
          {
            amount: '1',
            denom: 'untrn',
          },
        ],
      ),
    ).rejects.toThrowError(/expected at least/);
  });
  it('push pump', async () => {
    const { contractClient, neutronUserAddress } = context;
    const res = await contractClient.push(
      neutronUserAddress,
      {
        coins: [{ amount: '1000', denom: 'stake' }],
      },
      1.5,
      undefined,
      [
        {
          amount: '20000',
          denom: 'untrn',
        },
      ],
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
  });
  it('verify funds are received', async () => {
    const { neutronClient, neutronSecondUserAddress } = context;
    let ibcBalance = 0;
    await waitFor(async () => {
      const res = await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronSecondUserAddress,
      );
      ibcBalance = parseInt(
        res.data.balances.find((b) => b.denom.startsWith('ibc/'))?.amount ||
          '0',
      );
      return res.data.balances.length > 1;
    }, 40000);
    expect(ibcBalance).toEqual(1000);
  });
  it('check balance on pump', async () => {
    const { neutronClient, contractAddress } = context;
    const res =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        contractAddress,
      );
    expect(res.data.balances).toEqual([
      {
        amount: '10000',
        denom: 'untrn',
      },
    ]);
  });
  it('try to refund tokens from the pump', async () => {
    const {
      contractClient,
      neutronClient,
      neutronUserAddress,
      neutronSecondUserAddress,
    } = context;
    const {
      data: { balance },
    } = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronSecondUserAddress,
      {
        denom: 'untrn',
      },
    );
    const res = await contractClient.refund(neutronUserAddress, 1.5);
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
    const {
      data: { balance: newBalance },
    } = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronSecondUserAddress,
      { denom: 'untrn' },
    );
    expect(parseInt(newBalance.amount) - parseInt(balance.amount)).toEqual(
      10000,
    );
  });
});
