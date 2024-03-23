import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropPump } from '../generated/contractLib';
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

const DropPumpClass = DropPump.Client;

describe('Pump-Multi', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClientGaia?: InstanceType<typeof DropPumpClass>;
    contractClientLSM?: InstanceType<typeof DropPumpClass>;
    account?: AccountData;
    icaAddressGaia?: string;
    icaAddressLsm?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    lsmClient?: SigningStargateClient;
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
    lsmIBCDenom?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia', 'lsm'],
      {},
      { hermes: true },
    );
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
    context.lsmClient = await SigningStargateClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.lsm.rpc}`,
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
      fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
      1.5,
    );
    {
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await DropPump.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {
          connection_id: 'connection-0',
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
      context.contractClientGaia = new DropPump.Client(
        client,
        context.contractAddress,
      );
    }
    {
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await DropPump.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {
          connection_id: 'connection-1',
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
      context.contractClientLSM = new DropPump.Client(
        client,
        context.contractAddress,
      );
    }
  });
  it('register ICA gaia', async () => {
    const { contractClientGaia, neutronUserAddress } = context;
    const res = await contractClientGaia.registerICA(
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
      const res = await contractClientGaia.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 210_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddressGaia = ica;
  });
  it('register ICA lsm', async () => {
    const { contractClientLSM, neutronUserAddress } = context;
    const res = await contractClientLSM.registerICA(
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
      const res = await contractClientLSM.queryIca();
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
    context.icaAddressLsm = ica;
  });
  it('update config for gaia pump', async () => {
    const { contractClientGaia, neutronUserAddress, icaAddressLsm } = context;
    const res = await contractClientGaia.updateConfig(
      neutronUserAddress,
      {
        new_config: {
          dest_address: icaAddressLsm,
          dest_channel: 'channel-1',
          dest_port: 'transfer',
        },
      },
      1.5,
      undefined,
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
  });
  it('update config for lsm pump', async () => {
    const { contractClientLSM, neutronUserAddress, neutronSecondUserAddress } =
      context;
    const res = await contractClientLSM.updateConfig(
      neutronUserAddress,
      {
        new_config: {
          dest_address: neutronSecondUserAddress,
          dest_channel: 'channel-0',
          dest_port: 'transfer',
        },
      },
      1.5,
      undefined,
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
  });
  it('send some funds to ICA', async () => {
    const { gaiaClient, gaiaUserAddress, icaAddressGaia } = context;
    const res = await gaiaClient.sendTokens(
      gaiaUserAddress,
      icaAddressGaia,
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
  it('push gaia pump', async () => {
    const { contractClientGaia, neutronUserAddress, lsmClient, icaAddressLsm } =
      context;
    const res = await contractClientGaia.push(
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
    let balance;
    await waitFor(async () => {
      const res = await lsmClient.getAllBalances(icaAddressLsm);
      balance = res[0]?.amount;
      context.lsmIBCDenom = res[0]?.denom;
      return res.length > 0;
    }, 60_000);
    expect(balance).toBe('1000');
  });
  it('push lsm pump', async () => {
    const {
      contractClientLSM,
      neutronUserAddress,
      neutronClient,
      neutronSecondUserAddress,
      lsmIBCDenom,
    } = context;
    const res = await contractClientLSM.push(
      neutronUserAddress,
      {
        coins: [
          {
            amount: '1000',
            denom: lsmIBCDenom,
          },
        ],
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
    let balance;
    await waitFor(async () => {
      const res = await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronSecondUserAddress,
      );
      balance = res.data.balances.find((b) => b.denom.startsWith('ibc/'))
        ?.amount;
      return res.data.balances.length > 1;
    }, 60_000);
    expect(balance).toBe('1000');
  });
});
