import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropValidatorsStats } from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  setupAuthzExtension,
  setupSlashingExtension,
  setupStakingExtension,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import { stringToPath } from '@cosmjs/crypto';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { ValidatorState } from 'drop-ts-client/lib/src/contractLib/dropValidatorsStats';
import { AuthzExtension } from '@cosmjs/stargate/build/modules/authz/queries';
import { pubkeyToAddress } from '@cosmjs/amino';
import { SlashingExtension } from '@cosmjs/stargate/build/modules';

const StatsClass = DropValidatorsStats.Client;

describe('Validators stats', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof StatsClass>;
    account?: AccountData;
    gaiaAccount?: AccountData;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningCosmWasmClient;
    gaiaQueryClient?: QueryClient &
    StakingExtension &
    SlashingExtension &
    AuthzExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    firstValidatorAddress?: string;
    secondValidatorAddress?: string;
    firstValconsAddress?: string;
    secondValconsAddress?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
      {},
      { neutron: true, hermes: true },
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

    context.gaiaAccount = (await context.gaiaWallet.getAccounts())[0];
    context.gaiaClient = await SigningCosmWasmClient.connectWithSigner(
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
      setupSlashingExtension,
      setupAuthzExtension,
    );

    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
        },
      );
      context.firstValidatorAddress = (await wallet.getAccounts())[0].address;
      const firstValidatorInfo =
        await context.gaiaQueryClient.staking.validator(
          context.firstValidatorAddress,
        );
      context.firstValconsAddress = getValconsAddress(
        firstValidatorInfo.validator.consensusPubkey.value,
      );
    }
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/2'/0/0") as any],
        },
      );
      context.secondValidatorAddress = (await wallet.getAccounts())[0].address;
      const secondValidatorInfo =
        await context.gaiaQueryClient.staking.validator(
          context.secondValidatorAddress,
        );
      context.secondValconsAddress = getValconsAddress(
        secondValidatorInfo.validator.consensusPubkey.value,
      );
    }
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_validators_stats.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropValidatorsStats.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        connection_id: 'connection-0',
        port_id: 'transfer',
        profile_update_period: 10,
        info_update_period: 20,
        avg_block_time: 5,
        owner: account.address,
      },
      'label',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new DropValidatorsStats.Client(
      client,
      context.contractAddress,
    );
  });

  it('register stats queries', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.registerStatsQueries(
      account.address,
      {
        validators: [
          context.firstValidatorAddress,
          context.secondValidatorAddress,
        ],
      },
      1.6,
      undefined,
      [
        {
          amount: '10000000',
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query delegations query', async () => {
    let validators: ValidatorState[];

    const sigingInfos = await context.gaiaQueryClient.slashing.signingInfos();

    const validator1SigningInfo = sigingInfos.info[0];
    const validator2SigningInfo = sigingInfos.info[1];

    await waitFor(async () => {
      validators = await context.contractClient.queryState();

      return (
        validators.length > 0 && !!validators[0].last_processed_remote_height
      );
    }, 60000);

    expect(validators).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          valcons_address: validator1SigningInfo.address,
        }),
        expect.objectContaining({
          valcons_address: validator2SigningInfo.address,
        }),
      ]),
    );

    expect(validators).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          valoper_address: context.firstValidatorAddress,
          last_processed_remote_height: expect.any(Number),
          last_processed_local_height: expect.any(Number),
          last_validated_height: expect.any(Number),
          last_commission_in_range: expect.any(Number),
          uptime: '1',
          tombstone: false,
          prev_jailed_state: false,
          jailed_number: 0,
        }),
        expect.objectContaining({
          valoper_address: context.secondValidatorAddress,
          last_processed_remote_height: expect.any(Number),
          last_processed_local_height: expect.any(Number),
          last_validated_height: expect.any(Number),
          last_commission_in_range: expect.any(Number),
          uptime: '1',
          tombstone: false,
          prev_jailed_state: false,
          jailed_number: 0,
        }),
      ]),
    );

    expect(validators.length).toEqual(2);
  });
});

function getValconsAddress(pubkey: Uint8Array) {
  const buffer = Buffer.from(pubkey.slice(2));
  const base64PubKey = buffer.toString('base64');

  return pubkeyToAddress(
    {
      type: 'tendermint/PubKeyEd25519',
      value: base64PubKey,
    },
    'cosmosvalcons',
  );
}
