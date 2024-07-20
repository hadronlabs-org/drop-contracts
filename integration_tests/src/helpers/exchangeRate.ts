import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { connectComet } from '@cosmjs/tendermint-rpc';
import { Decimal } from 'decimal.js';
import { QueryClient, createProtobufRpcClient } from '@cosmjs/stargate';
import {
  setupAuthExtension,
  setupBankExtension,
  setupStakingExtension,
  setupTxExtension,
} from '@cosmjs/stargate';
import { QueryClientImpl } from 'cosmjs-types/cosmos/bank/v1beta1/query';

async function calcExchangeRate(
  clientCW: SigningCosmWasmClient,
  coreContract: string,
  endpoint: string
): Promise<number> {
  const coreConfig = await clientCW.queryContractSmart(coreContract, {
    config: {},
  });
  const FSMState = await clientCW.queryContractSmart(coreContract, {
    contract_state: {},
  });
  if (FSMState !== "idle") {
    // If state isn't idle then this query'll return cached exchange rate
    return Number(
      await clientCW.queryContractSmart(coreContract, {
        exchange_rate: {},
      })
    );
  }
  const queryClient = QueryClient.withExtensions(
    await connectComet(endpoint),
    setupAuthExtension,
    setupBankExtension,
    setupStakingExtension,
    setupTxExtension
  );
  const RPC = createProtobufRpcClient(queryClient);
  const queryService = new QueryClientImpl(RPC);
  const tokenContractConfig = await clientCW.queryContractSmart(
    coreConfig.token_contract,
    {
      config: {},
    }
  );
  const { amount } = await queryService.SupplyOf({
    denom: tokenContractConfig.denom,
  });
  let exchangeRateDenominator: Decimal = new Decimal(amount.amount);
  if (exchangeRateDenominator.isZero()) {
    return new Decimal(1).toNumber();
  }
  const delegationsResponse = await clientCW.queryContractSmart(
    coreConfig.puppeteer_contract,
    {
      extension: {
        msg: {
          delegations: {},
        },
      },
    }
  );
  const delegationsAmount: Decimal =
    delegationsResponse.delegations.delegations.reduce(
      (acc: Decimal, next: any) => acc.plus(new Decimal(next.amount.amount)),
      new Decimal(0)
    );
  const batchID = await clientCW.queryContractSmart(coreContract, {
    current_unbond_batch: {},
  });
  const batch = await clientCW.queryContractSmart(coreContract, {
    unbond_batch: {
      batch_id: String(batchID),
    },
  });
  let unprocessedDassetToUnbond: Decimal = new Decimal("0");
  if (batch.status === "new") {
    unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
      batch.total_dasset_amount_to_withdraw
    );
  }
  if (Number(batchID) > 0) {
    const penultimate = Number(batchID) - 1;
    const penultimateBatch = await clientCW.queryContractSmart(coreContract, {
      unbond_batch: {
        batch_id: String(penultimate),
      },
    });
    if (penultimateBatch.status === "unbond_requested") {
      unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
        batch.total_dasset_amount_to_withdraw
      );
    }
  }
  const failedBatchID = await clientCW.queryContractSmart(coreContract, {
    failed_batch: {},
  });
  if (failedBatchID.response !== null) {
    const failedBatch = await clientCW.queryContractSmart(coreContract, {
      batch_id: failedBatchID.response,
    });
    unprocessedDassetToUnbond = unprocessedDassetToUnbond.plus(
      failedBatch.total_dasset_amount_to_withdraw
    );
  }
  exchangeRateDenominator = exchangeRateDenominator.plus(
    unprocessedDassetToUnbond
  );
  const stakerBalance: Decimal = new Decimal(
    await clientCW.queryContractSmart(coreConfig.staker_contract, {
      all_balance: {},
    })
  );
  const totalLSMShares: Decimal = new Decimal(
    await clientCW.queryContractSmart(coreContract, {
      total_l_s_m_shares: {},
    })
  );
  const exchangeRateNumerator: Decimal = delegationsAmount
    .plus(stakerBalance)
    .plus(totalLSMShares);
  if (exchangeRateNumerator.isZero()) {
    return 1;
  }
  const exchangeRate: Decimal = exchangeRateNumerator.dividedBy(
    exchangeRateDenominator
  );
  return exchangeRate.toNumber();
}

async function compareExchangeRates(
  clientCW: SigningCosmWasmClient,
  coreContract: string,
  endpoint: string,
  decimals: number
): Promise<boolean> {
  return (
    (await calcExchangeRate(clientCW, coreContract, endpoint)).toFixed(
      decimals
    ) ===
    Number(
      await clientCW.queryContractSmart(coreContract, {
        exchange_rate: {},
      })
    ).toFixed(decimals)
  );
}
