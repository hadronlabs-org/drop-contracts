import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SigningStargateClient } from "@cosmjs/stargate";
import {
  AccountData,
  DirectSecp256k1HdWallet,
  Registry,
  GeneratedType,
} from "@cosmjs/proto-signing";
import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
import { Coin as BankCoin } from "cosmjs-types/cosmos/base/v1beta1/coin";
import { MsgTransfer } from "cosmjs-types/ibc/applications/transfer/v1/tx";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";
import { Client as DropValidatorsSet } from "../../../integration_tests/src/generated/contractLib/dropValidatorsSet";
import { BinaryReader, BinaryWriter } from "cosmjs-types/binary";
import { DeepPartial, Exact } from "cosmjs-types/helpers";
import { MsgDelegate } from "cosmjs-types/cosmos/staking/v1beta1/tx";
import { sleep } from "../../../integration_tests/src/helpers/sleep";

type MsgTokenizeShares = {
  delegator_address: string;
  validator_address: string;
  amount: BankCoin;
  tokenized_share_owner: string;
};

// We need only to encode such messages
export const MsgTokenizeShares = {
  typeUrl: "/cosmos.staking.v1beta1.MsgTokenizeShares",
  encode(
    message: MsgTokenizeShares,
    writer: BinaryWriter = BinaryWriter.create()
  ): BinaryWriter {
    writer.uint32(10).string(message.delegator_address);
    writer.uint32(18).string(message.validator_address);
    BankCoin.encode(message.amount, writer.uint32(26).fork()).ldelim();
    writer.uint32(34).string(message.tokenized_share_owner);
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): any {
    console.log("decode");
    return {};
  },
  fromPartial<I extends Exact<DeepPartial<MsgTokenizeShares>, I>>(
    object: I
  ): MsgTokenizeShares {
    return {
      delegator_address: object.delegator_address,
      validator_address: object.validator_address,
      amount: BankCoin.fromPartial(object.amount),
      tokenized_share_owner: object.tokenized_share_owner,
    };
  },
};

const CORE_CONTRACT: string = process.env.CORE_CONTRACT;

const NEUTRON_MNEMONIC: string = process.env.NEUTRON_MNEMONIC;
const TARGET_MNEMONIC: string = process.env.TARGET_MNEMONIC;
const NEUTRON_PREFIX: string = process.env.NEUTRON_PREFIX;
const TARGET_PREFIX: string = process.env.TARGET_PREFIX;

const TARGET_NATIVE_DENOM: string = process.env.TARGET_NATIVE_DENOM;
const TARGET_IBC_DENOM: string = process.env.TARGET_IBC_DENOM;
const FACTORY_DENOM: string = process.env.FACTORY_DENOM;

const NEUTRON_NODE_ADDRESS: string = process.env.NEUTRON_NODE_ADDRESS;
const TARGET_NODE_ADDRESS: string = process.env.TARGET_NODE_ADDRESS;

const BOND_PROB: number = Number(process.env.BOND_PROB);
const UNBOND_PROB: number = Number(process.env.UNBOND_PROB);
const WITHDRAW_PROB: number = Number(process.env.WITHDRAW_PROB);
const LSM_SHARE_BOND_PROB: number = Number(process.env.LSM_SHARE_BOND_PROB);

/* Sum of provided probabitions should be equal to 1
 * Each of provided probabitions used to calculate first method to execute
 * If chosen method falls with code != 0 then 1 of other possible methods will be executed
 * With equal probabition for each of remaining methods
 * In short, given probabitions are only used to choose first method to try to execute
 */
if (BOND_PROB + UNBOND_PROB + WITHDRAW_PROB + LSM_SHARE_BOND_PROB !== 1) {
  console.error(
    `BOND_PROB(${BOND_PROB}) + UNBOND_PROB(${UNBOND_PROB}) + WITHDRAW_PROB(${WITHDRAW_PROB}) + LSM_SHARE_BOND_PROB(${LSM_SHARE_BOND_PROB}) != 1`
  );
  process.exit(1);
}

enum MODE {
  BOND = "BOND",
  UNBOND = "UNBOND",
  WITHDRAW = "SEND_NFT",
  LSM_SHARE_BOND = "LSM_SHARE_BOND",
}

async function calculate_mode(
  bond_p: number,
  unbond_p: number,
  withdraw_p: number,
  lsm_share_bond_p: number
): Promise<MODE> {
  let r: number = Math.random();
  if (r < bond_p) {
    return MODE.BOND;
  } else if (r < bond_p + unbond_p) {
    return MODE.UNBOND;
  } else if (r < bond_p + unbond_p + withdraw_p) {
    return MODE.WITHDRAW;
  } else if (r < bond_p + unbond_p + withdraw_p + lsm_share_bond_p) {
    return MODE.LSM_SHARE_BOND;
  } else {
    console.error(
      `bond_p + unbond_p + withdraw_p = ${bond_p + unbond_p + withdraw_p}, it should be eq to 1`
    );
    process.exit(1);
  }
}

type Wallet = {
  mainWallet?: DirectSecp256k1HdWallet;
  clientCW?: SigningCosmWasmClient;
  clientSG?: SigningStargateClient;
  mainAccounts?: readonly AccountData[];
};

type Wallets = {
  neutronWallet: Wallet;
  targetWallet: Wallet;
};

type BondActionLog = {
  mode: MODE;
  funds: Coin;
  txHash: string;
  details: null;
};

async function bond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<BondActionLog> {
  return {
    details: null,
    mode: MODE.BOND,
    funds: fund,
    txHash: (
      await drop_instance.bond(address, {}, "auto", "", [
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function bond_random_amount(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<BondActionLog | null> {
  const address: string = neutronWallet.mainAccounts[0].address;

  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  let ibc_denom_balance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    TARGET_IBC_DENOM
  );
  if (Number(ibc_denom_balance.amount) === 0) {
    return null;
  }

  const config = await drop_instance.queryConfig();
  const exchange_rate: number = Math.floor(
    Number(await drop_instance.queryExchangeRate())
  );
  const min_exchange_rate: number = exchange_rate + 1;

  /* Minimum amount of funds that we can send to core contract while bonding
   * Is either lsm_min_bond_amount (which is typically set in 1) or current exchange_rate parameter
   * Here we're choosing the biggest value of these two to avoid further errors
   */
  const min: number =
    Number(config.lsm_min_bond_amount) < min_exchange_rate
      ? min_exchange_rate
      : Number(config.lsm_min_bond_amount);
  if (min > Number(ibc_denom_balance.amount)) {
    return null;
  }

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  let random_amount: number = Math.floor(
    Math.random() * (Number(ibc_denom_balance.amount) - min) + min
  );
  try {
    const res = await bond(drop_instance, address, {
      amount: String(random_amount),
      denom: TARGET_IBC_DENOM,
    });
    if ((await neutronWallet.clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

type UnbondActionLog = {
  mode: MODE;
  funds: Coin;
  txHash: string;
  details: null;
};

async function unbond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<UnbondActionLog> {
  return {
    details: null,
    mode: MODE.UNBOND,
    funds: fund,
    txHash: (
      await drop_instance.unbond(address, "auto", "", [
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function unbond_random_amount(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<UnbondActionLog | null> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  let factory_balance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    FACTORY_DENOM
  );
  if (Number(factory_balance.amount) === 0) {
    return null;
  }

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  let random_amount: number = Math.floor(
    Math.random() * Number(factory_balance.amount) + 1
  );
  try {
    const res = await unbond(drop_instance, address, {
      amount: String(random_amount),
      denom: FACTORY_DENOM,
    });
    if ((await neutronWallet.clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

type WithdrawActionLog = {
  mode: MODE;
  funds: {
    nft_id: string;
  };
  txHash: string;
  details: {
    batch_id: number;
  };
};

async function send_nft(
  withdrawal_voucher: DropWithdrawalVoucher,
  withdrawal_manager: DropWithdrawalManager,
  address: string,
  nft_id: string
): Promise<WithdrawActionLog> {
  const nft_info = await withdrawal_voucher.queryNftInfo({ token_id: nft_id });
  const batch_id: number = Number(nft_info.extension.batch_id);

  /* To withdraw unbonded amount we need to send nft to the withdrawal_manager contract
   * To do that we need to call send method on withdrawal_voucher contract (which is NFT contract underhood)
   * Field msg here is encoded base64 json object { "withdraw": {} }
   */
  return {
    details: {
      batch_id: batch_id,
    },
    mode: MODE.WITHDRAW,
    funds: {
      nft_id: nft_id,
    },
    txHash: (
      await withdrawal_voucher.sendNft(
        address,
        {
          contract: withdrawal_manager.contractAddress,
          token_id: nft_id,
          msg: "eyJ3aXRoZHJhdyI6e319",
        },
        "auto",
        "",
        []
      )
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function withdraw_random_nft(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<WithdrawActionLog | null> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* Get both withdrawal_manager and withdrawal_voucher wrappers based on querying config method
   * We need them to execute send_nft method on withdrawal_voucher with withdrawal_manager as the recepient
   * To withdraw our unbonded tokens we need to work with withdrawal_voucher' send_nft method
   */
  const config = await drop_instance.queryConfig();
  const withdrawal_manager: DropWithdrawalManager = new DropWithdrawalManager(
    neutronWallet.clientCW,
    config.withdrawal_manager_contract
  );
  const withdrawal_voucher: DropWithdrawalVoucher = new DropWithdrawalVoucher(
    neutronWallet.clientCW,
    config.withdrawal_voucher_contract
  );

  /* Get all NFT tokens in 1-st loop possessed by given address
   * Get details for each gotten NFT in 2-nd loop
   * Check batch status for each batch_id, filter who hasn't withdrawed yet in 3-rd loop
   */
  let owned_nfts = (await withdrawal_voucher.queryTokens({ owner: address }))
    .tokens;
  let current_nft_list = owned_nfts;
  while (current_nft_list.length !== 0) {
    current_nft_list = (
      await withdrawal_voucher.queryTokens({
        owner: address,
        start_after: current_nft_list[current_nft_list.length - 1],
      })
    ).tokens;
    owned_nfts = owned_nfts.concat(current_nft_list);
  }
  const owned_nfts_detailed = [];
  for (const nft of owned_nfts) {
    owned_nfts_detailed.push({
      nft_id: nft,
      details: {
        ...(await withdrawal_voucher.queryNftInfo({
          token_id: nft,
        })),
      },
    });
  }
  const withdrawn_nfts = [];
  for (const nft_detailed of owned_nfts_detailed) {
    if (
      (
        await drop_instance.queryUnbondBatch({
          batch_id: nft_detailed.details.extension.batch_id,
        })
      ).status === "withdrawn"
    ) {
      withdrawn_nfts.push(nft_detailed.nft_id);
    }
  }

  /* Nothing to withdraw
   * Return null and try another method in contract
   */
  if (withdrawn_nfts.length === 0) {
    return null;
  }

  /* Pick up random NFT from given NFT list and try to withdraw it
   * If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const nft_id: string =
    withdrawn_nfts[Math.floor(Math.random() * withdrawn_nfts.length)];
  try {
    return await send_nft(
      withdrawal_voucher,
      withdrawal_manager,
      address,
      nft_id
    );
  } catch (e) {
    return null;
  }
}

type LSMShareBondActionLog = {
  mode: MODE;
  funds: {
    ibc: Coin;
    tokenized_share: Coin;
    ibc_tokenized_share: Coin;
  };
  txHash: {
    bridge: string;
    stake: string;
    brdige_back: string;
    stake_tokenized_share: string;
  };
  details: {
    validator_cosmvaloper: string;
  };
};

async function get_last_lsm_denom(wallet: Wallet): Promise<string> {
  return (await wallet.clientSG.getAllBalances(wallet.mainAccounts[0].address))
    .filter((balance) => balance.denom.includes("cosmosvaloper"))
    .map((balance) => {
      return {
        denom: balance.denom,
        token_share_id: Number(balance.denom.split("/")[1]),
      };
    })
    .reduce((maxId, current) => {
      return maxId.token_share_id > current.token_share_id ? maxId : current;
    }).denom;
}

async function lsm_share_bond(
  wallets: Wallets,
  drop_instance: DropCoreClient
): Promise<LSMShareBondActionLog | null> {
  const remote_chain_balance = Number(
    (
      await wallets.targetWallet.clientCW.getBalance(
        wallets.targetWallet.mainAccounts[0].address,
        TARGET_NATIVE_DENOM
      )
    ).amount
  );
  if (remote_chain_balance <= 8002) {
    return null;
  }
  // For some reason it's not working properly with 1 conventional unit in remote chain (https://www.mintscan.io/cosmoshub-testnet/tx/72632F1594285A0D23D878E781B3B8533D44DE87CF3FAAD7C440E5374DF9DEDA?height=22040761)
  // But works well with cu >= 2 (https://www.mintscan.io/cosmoshub-testnet/tx/698B451AA918294009EDBB813EE98B759D8DAAEDCB4E0789A07590B19C3615AC?height=22040981)
  // Idk but it seems like it's kind of bug in client
  const random_amount_delegate = Math.floor(
    Math.random() *
      (remote_chain_balance >= 100_000_000_000
        ? 100_000_000_000
        : remote_chain_balance - 8002) +
      2
  );
  const core_config = await drop_instance.queryConfig();
  const drop_validators_set: DropValidatorsSet = new DropValidatorsSet(
    wallets.neutronWallet.clientCW,
    core_config.validators_set_contract
  );
  const validator_list: Array<string> = (
    await drop_validators_set.queryValidators()
  ).map((element) => element.valoper_address);
  const random_validator =
    validator_list[Math.floor(Math.random() * validator_list.length)];
  const last_lsm_denom = await get_last_lsm_denom(wallets.targetWallet);

  try {
    await wallets.targetWallet.clientCW.delegateTokens(
      wallets.targetWallet.mainAccounts[0].address,
      random_validator,
      {
        denom: TARGET_NATIVE_DENOM,
        amount: random_amount_delegate.toString(),
      },
      {
        amount: [
          {
            denom: TARGET_NATIVE_DENOM,
            amount: "4000",
          },
        ],
        gas: "400000",
      },
      ""
    );
  } catch (e) {}

  try {
    await wallets.targetWallet.clientCW.signAndBroadcastSync(
      wallets.targetWallet.mainAccounts[0].address,
      [
        {
          typeUrl: "/cosmos.staking.v1beta1.MsgTokenizeShares",
          value: {
            delegator_address: wallets.targetWallet.mainAccounts[0].address,
            validator_address: random_validator,
            amount: {
              denom: TARGET_NATIVE_DENOM,
              amount: random_amount_delegate.toString(),
            },
            tokenized_share_owner: wallets.targetWallet.mainAccounts[0].address,
          },
        },
      ],
      {
        gas: "400000",
        amount: [
          {
            denom: TARGET_NATIVE_DENOM,
            amount: "4000",
          },
        ],
      },
      ""
    );
  } catch (e) {}

  while ((await get_last_lsm_denom(wallets.targetWallet)) === last_lsm_denom) {
    await sleep(5000);
  }
  return;
}

async function lsm_share_bond_random_amount(
  wallets: Wallets,
  drop_instance: DropCoreClient
): Promise<LSMShareBondActionLog | null> {
  lsm_share_bond(wallets, drop_instance);
  return;
}

async function main() {
  const neutronWallet: Wallet = {};
  neutronWallet.mainWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    NEUTRON_MNEMONIC,
    {
      prefix: NEUTRON_PREFIX,
    }
  );
  neutronWallet.clientCW = await SigningCosmWasmClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.clientSG = await SigningStargateClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.mainAccounts = await neutronWallet.mainWallet.getAccounts();

  const targetWallet: Wallet = {};
  targetWallet.mainWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    TARGET_MNEMONIC,
    {
      prefix: TARGET_PREFIX,
    }
  );
  targetWallet.clientCW = await SigningCosmWasmClient.connectWithSigner(
    TARGET_NODE_ADDRESS,
    targetWallet.mainWallet,
    {
      registry: new Registry(
        new Map<string, GeneratedType>([
          ["/cosmos.base.v1beta1.Coin", BankCoin],
          ["/cosmos.bank.v1beta1.MsgSend", MsgSend],
          ["/ibc.applications.transfer.v1.MsgTransfer", MsgTransfer],
          ["/cosmos.staking.v1beta1.MsgTokenizeShares", MsgTokenizeShares],
          ["/cosmos.staking.v1beta1.MsgDelegate", MsgDelegate],
        ])
      ),
    }
  );
  targetWallet.clientSG = await SigningStargateClient.connectWithSigner(
    TARGET_NODE_ADDRESS,
    targetWallet.mainWallet
  );
  targetWallet.mainAccounts = await targetWallet.mainWallet.getAccounts();

  const wallets: Wallets = {
    neutronWallet: neutronWallet,
    targetWallet: targetWallet,
  };

  let core_contract = new DropCoreClient(neutronWallet.clientCW, CORE_CONTRACT);

  /* Randombly choose the method we'll try to execute based on provided parameters
   * Variable unused_modes is array of modes we haven't yet executed. We'll try each of them if previous call falled
   * If randomly chosen method'll fall then with equal possibility choose another possible method from unused_modes
   * If each of possible methods fall then our job here's done, print appropriate message and call process.exit() with code 1
   */
  let mode: MODE = await calculate_mode(
    BOND_PROB,
    UNBOND_PROB,
    WITHDRAW_PROB,
    LSM_SHARE_BOND_PROB
  );
  let unused_modes: MODE[] = [
    MODE.BOND,
    MODE.UNBOND,
    MODE.WITHDRAW,
    MODE.LSM_SHARE_BOND,
  ];
  let finished: boolean = false;
  while (!finished && unused_modes.length > 0) {
    switch (mode) {
      // case MODE.WITHDRAW: {
      //   const res = await withdraw_random_nft(neutronWallet, core_contract);
      //   if (res !== null) {
      //     console.log(res);
      //     finished = true;
      //   }
      //   break;
      // }
      // case MODE.UNBOND: {
      //   const res = await unbond_random_amount(neutronWallet, core_contract);
      //   if (res !== null) {
      //     console.log(res);
      //     finished = true;
      //   }
      //   break;
      // }
      // case MODE.BOND: {
      //   const res = await bond_random_amount(neutronWallet, core_contract);
      //   if (res !== null) {
      //     console.log(res);
      //     finished = true;
      //   }
      //   break;
      // }
      case MODE.LSM_SHARE_BOND: {
        const res = await lsm_share_bond_random_amount(wallets, core_contract);
        if (res !== null) {
          console.log(res);
          finished = true;
        }
        break;
      }
    }
    unused_modes = unused_modes.filter((element) => element !== mode);
    mode = unused_modes[Math.floor(Math.random() * unused_modes.length)];
  }
  if (unused_modes.length === 0) {
    console.error("Nothing to bond, unbond or withdraw");
    process.exit(1);
  }
}

main();
