import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  LidoProviderProposalsPoc,
  LidoProposalVotesPoc,
} from '../generated/contractLib';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  IndexedTx,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import { stringToPath } from '@cosmjs/crypto';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { ProposalInfo1 } from '../generated/contractLib/lidoProviderProposals';

const LidoProviderProposalsClass = LidoProviderProposalsPoc.Client;
const LidoProposalVotesClass = LidoProposalVotesPoc.Client;

describe('POC Proposal Votes', () => {
  const context: {
    park?: Cosmopark;
    propsContractAddress?: string;
    votesContractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    propsContractClient?: InstanceType<typeof LidoProviderProposalsClass>;
    votesContractClient?: InstanceType<typeof LidoProposalVotesClass>;
    account?: AccountData;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningCosmWasmClient;
    gaiaUserAddress?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    neutronUserAddress?: string;
    validatorAddress?: string;
    secondValidatorAddress?: string;
  } = {};

  beforeAll(async () => {
    context.park = await setupPark(
      'providerprops',
      ['neutron', 'gaia'],
      true,
      true,
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
      setupBankExtension,
    );
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    const propsRes = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/lido_provider_proposals_poc.wasm'),
      ),
      1.5,
    );
    expect(propsRes.codeId).toBeGreaterThan(0);

    const instantiatePropsRes =
      await LidoProviderProposalsPoc.Client.instantiate(
        client,
        account.address,
        propsRes.codeId,
        {
          connection_id: 'connection-0',
          port_id: 'transfer',
          update_period: 10,
          core_address: account.address,
          validators_set_address: account.address,
          init_proposal: 1,
          proposals_prefetch: 10,
          veto_spam_threshold: '0.5',
        },
        'label',
        [
          {
            amount: '10000000',
            denom: 'untrn',
          },
        ],
        'auto',
      );
    expect(instantiatePropsRes.contractAddress).toHaveLength(66);
    context.propsContractAddress = instantiatePropsRes.contractAddress;
    context.propsContractClient = new LidoProviderProposalsPoc.Client(
      client,
      context.propsContractAddress,
    );

    const votesRes = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/lido_proposal_votes_poc.wasm'),
      ),
      1.5,
    );
    expect(votesRes.codeId).toBeGreaterThan(0);

    const instantiateVotesRes = await LidoProposalVotesPoc.Client.instantiate(
      client,
      account.address,
      votesRes.codeId,
      {
        connection_id: 'connection-0',
        port_id: 'transfer',
        update_period: 10,
        core_address: account.address,
        provider_proposals_address: context.propsContractAddress,
      },
      'label',
      [
        {
          amount: '10000000',
          denom: 'untrn',
        },
      ],
      'auto',
    );
    expect(instantiateVotesRes.contractAddress).toHaveLength(66);
    context.votesContractAddress = instantiateVotesRes.contractAddress;
    context.votesContractClient = new LidoProposalVotesPoc.Client(
      client,
      context.votesContractAddress,
    );

    context.gaiaUserAddress = (
      await context.gaiaWallet.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;

    const res = await context.votesContractClient.updateVotersList(
      account.address,
      {
        voters: [context.gaiaUserAddress],
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('delegate tokens on gaia side and create text proposal', async () => {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.master_mnemonic,
      {
        prefix: 'cosmosvaloper',
        hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
      },
    );
    context.validatorAddress = (await wallet.getAccounts())[0].address;
    let res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx staking delegate ${context.validatorAddress} 1000000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
    );
    expect(res.exitCode).toBe(0);
    let out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    let tx: IndexedTx | null = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });
    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);

    res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx gov submit-proposal --type text --title test --description test --from ${context.gaiaUserAddress} --deposit 10000000stake --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
    );
    expect(res.exitCode).toBe(0);
    out = JSON.parse(res.out);

    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    tx = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });
    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);

    res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx gov vote 1 yes --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
    );
    expect(res.exitCode).toBe(0);
    out = JSON.parse(res.out);

    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    tx = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });

    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);

    await waitFor(async () => {
      const proposals = await context.propsContractClient.queryGetProposals();

      return proposals.length > 0;
    }, 60000);

    const resUpdate = await context.propsContractClient.updateConfig(
      context.account.address,
      {
        new_config: {
          proposal_votes_address: context.votesContractAddress,
        },
      },
      1.5,
    );
    expect(resUpdate.transactionHash).toBeTruthy();
  });

  it('query gaiad relayed proposals', async () => {
    let proposals: ProposalInfo1[];

    await waitFor(async () => {
      proposals = await context.propsContractClient.queryGetProposals();

      return proposals.length > 0 && proposals[0].votes !== null;
    }, 60000);

    expect(proposals).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          proposal: expect.objectContaining({
            proposal_id: 1,
            proposal_type: '/cosmos.gov.v1beta1.TextProposal',
            status: 2,
            submit_time: expect.any(Number),
            deposit_end_time: expect.any(Number),
            voting_start_time: expect.any(Number),
            voting_end_time: expect.any(Number),
          }),
          votes: expect.arrayContaining([
            expect.objectContaining({
              proposal_id: 1,
              voter: context.gaiaUserAddress,
              options: [
                {
                  option: 1,
                  weight: '1000000000000000000',
                },
              ],
            }),
          ]),
          is_spam: false,
        }),
      ]),
    );

    expect(proposals.length).toEqual(1);
  });

  it('query contract metrics', async () => {
    const metrics = await context.votesContractClient.queryMetrics();

    expect(metrics).toEqual({
      total_voters: 1,
    });
  });
});
