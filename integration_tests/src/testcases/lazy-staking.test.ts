import { afterAll, beforeAll, it, describe, expect } from 'vitest';
import Cosmopark from '@neutron-org/cosmopark';
import { setupPark } from '../testSuite';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { join } from 'path';
import fs from 'fs';
import {
  DropLazyStaking,
  DropTemplateCoreContract,
  DropTemplateFactoryContract,
} from 'drop-ts-client';

const DropLazyStakingClient = DropLazyStaking.Client;
const DropTemplateCoreClient = DropTemplateCoreContract.Client;
const DropTemplateFactoryClient = DropTemplateFactoryContract.Client;

describe('Splitter', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    account?: AccountData;
    neutronClient?: InstanceType<typeof NeutronClient>;
    client?: SigningCosmWasmClient;

    lazyStakingClient?: InstanceType<typeof DropLazyStakingClient>;
    coreContractClient?: InstanceType<typeof DropTemplateCoreClient>;
    factoryContractClient?: InstanceType<typeof DropTemplateFactoryClient>;
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
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../../artifacts/drop_template_core_contract.wasm',
          ),
        ),
        1.5,
      );
      const { contractAddress } = await DropTemplateCoreClient.instantiate(
        client,
        account.address,
        codeId,
        {},
        'label',
        'auto',
        [],
      );
      context.coreContractClient = new DropTemplateCoreClient(
        client,
        contractAddress,
      );
    }
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../../artifacts/drop_template_factory_contract.wasm',
          ),
        ),
        1.5,
      );
      const { contractAddress } = await DropTemplateFactoryClient.instantiate(
        client,
        account.address,
        codeId,
        {},
        'label',
        'auto',
        [],
      );
      context.factoryContractClient = new DropTemplateFactoryClient(
        client,
        contractAddress,
      );
    }
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_lazy_staking.wasm'),
        ),
        1.5,
      );
      const { contractAddress } = await DropLazyStakingClient.instantiate(
        client,
        account.address,
        codeId,
        {
          exponent: 6,
          subdenom: 'ldNTRN',
          token_metadata: {
            description: 'lazy derivative for Drop Neutron',
            denom_units: [],
            base: '6',
            display: 'ldNTRN',
            name: 'ldNTRN',
            symbol: 'ldNTRN',
            uri: '',
            uri_hash: '',
          },
          config: {
            factory_addr: context.factoryContractClient.contractAddress,
            base_denom: 'untrn',
            rewards_receiver: account.address,
          },
        },
        'label',
        'auto',
        [],
      );
      context.lazyStakingClient = new DropLazyStakingClient(
        client,
        contractAddress,
      );
    }
  });
});
