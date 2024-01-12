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
import { MsgTransfer } from 'cosmjs-types/ibc/applications/transfer/v1/tx';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, setupPark } from '../testSuite';
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
    const { client, account, neutronSecondUserAddress, gaiaUserAddress } =
      context;

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
        dest_address: gaiaUserAddress,
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
          local: 10,
          remote: 10,
        },
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
    ).rejects.toThrowError(/fee should be provided and be one coin/);
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
      const res = await contractClient.queryState();
      ica = res.ica;
      return !!res.ica;
    }, 50_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
  });
});
