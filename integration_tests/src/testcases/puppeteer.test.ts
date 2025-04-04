import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropPuppeteer } from 'drop-ts-client';
import { SigningStargateClient } from '@cosmjs/stargate';
import { join } from 'path';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';

const DropPuppeteerClass = DropPuppeteer.Client;

describe('Puppeteer', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof DropPuppeteerClass>;
    account?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    neutronUserAddress?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
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
    const { client, account } = context;
    const res = await client.upload(
      account.address,
      Uint8Array.from(
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
        ),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropPuppeteer.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        timeout: 172800,
        owner: account.address,
        connection_id: 'connection-0',
        sdk_version: '0.47.16',
        transfer_channel_id: 'channel-0',
        port_id: 'transfer',
        allowed_senders: [account.address],
        delegations_queries_chunk_size: 8,
        remote_denom: 'stake',
        update_period: 5,
        factory_contract: account.address,
      },
      'label',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new DropPuppeteer.Client(
      client,
      context.contractAddress,
    );
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

  it('validate tokenize shares enabled', async () => {
    const status = JSON.parse(
      (
        await context.park.executeInNetwork(
          'gaia',
          `${context.park.config.networks['gaia'].binary} query staking tokenize-share-lock-info ${context.icaAddress} --output json`,
        )
      ).out,
    ).status;
    expect(status).toEqual('TOKENIZE_SHARE_LOCK_STATUS_UNLOCKED');
  });

  it('disable tokenize shares', async () => {
    const { contractClient, neutronUserAddress } = context;

    const res = await contractClient.disableTokenizeShares(
      neutronUserAddress,
      1.5,
      undefined,
      [
        {
          amount: '100000',
          denom: 'untrn',
        },
      ],
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);

    await waitFor(async () => {
      const res = await contractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);

    const status = JSON.parse(
      (
        await context.park.executeInNetwork(
          'gaia',
          `${context.park.config.networks['gaia'].binary} query staking tokenize-share-lock-info ${context.icaAddress} --output json`,
        )
      ).out,
    ).status;
    expect(status).toEqual('TOKENIZE_SHARE_LOCK_STATUS_LOCKED');
  });

  it('enable tokenize shares', async () => {
    const { contractClient, neutronUserAddress } = context;

    const res = await contractClient.enableTokenizeShares(
      neutronUserAddress,
      1.5,
      undefined,
      [
        {
          amount: '100000',
          denom: 'untrn',
        },
      ],
    );
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);

    await waitFor(async () => {
      const res = await contractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);

    const status = JSON.parse(
      (
        await context.park.executeInNetwork(
          'gaia',
          `${context.park.config.networks['gaia'].binary} query staking tokenize-share-lock-info ${context.icaAddress} --output json`,
        )
      ).out,
    ).status;
    expect(status).toEqual('TOKENIZE_SHARE_LOCK_STATUS_LOCK_EXPIRING');
  });
});
